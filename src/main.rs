use simulador_de_fluxo::*;

use std::{fs, process::Command, sync::LazyLock};

use pbar::{MultiProgress, ProgressBar, ProgressStyle};

static PBAR_STYLE: LazyLock<ProgressStyle> = LazyLock::new(|| {
    ProgressStyle::default_bar()
        .progress_chars("=>-")
        .template(
            "{prefix:20} [{elapsed_precise} / {eta_precise}] {bar:40.cyan/white} {pos:>7}/{len:7} {msg}",
        )
        .unwrap()
});
static mut PBARS: LazyLock<MultiProgress> = LazyLock::new(|| MultiProgress::new());

fn add_pbar(pbar: ProgressBar) -> ProgressBar {
    unsafe { (*PBARS).add(pbar) }
}

fn main() {
    //* Preparating simulation universe
    println!("Preparating simulation universe...");
    let mut universe = {
        const WIDTH: u32 = 540 - 1;
        const HEIGHT: u32 = 540 - 1;
        const OBJECT_RADIUS: f64 = 1.5;
        let mut universe = Universe::new(WIDTH, HEIGHT);

        let center = Point {
            x: universe.width() as f64 / 2.0,
            y: universe.height() as f64 / 2.0,
        };
        //? Two masses.
        // let t = (WIDTH as f64 * 0.125) as u32;
        // universe[(t, HEIGHT >> 1)].element_mut().unwrap().mass.value = 1.0;
        // universe[(WIDTH - t, HEIGHT >> 1)]
        //     .element_mut()
        //     .unwrap()
        //     .mass
        //     .value = 1.0;
        //? Circle with mass
        for y in (-OBJECT_RADIUS) as i64..=OBJECT_RADIUS as i64 {
            for x in (-OBJECT_RADIUS) as i64..=OBJECT_RADIUS as i64 {
                if (x as f64).hypot(y as f64) <= OBJECT_RADIUS {
                    let x = (center.x as i64 + x) as u32;
                    let y = (center.y as i64 + y) as u32;
                    let (.., element) = universe[(x, y)].element_mut().unwrap();
                    element.mass.value = 1.0;
                }
            }
        }

        let portalset = {
            let delta = universe.width() as f64 / 6.0;
            let portal_height = universe.height() as f64 / 2.0;
            (
                Portal::new(
                    center + (-delta, -portal_height / 2.0),
                    center + (delta, -portal_height / 2.0),
                ),
                Portal::new(
                    center + (-delta, portal_height / 2.0),
                    center + (delta, portal_height / 2.0),
                ),
            )
        };
        // println!(
        //     "Portalset {{ {} - {} <-> {} - {} }}",
        //     portalset.a.point_a, portalset.a.point_b, portalset.b.point_a, portalset.b.point_b
        // );
        universe.add_portal_set(portalset);
        universe
    };

    //* Remove previous images
    println!("Clearing previous images");
    match fs::exists(FOLDER) {
        Ok(true) => {
            for f in fs::read_dir(FOLDER).unwrap() {
                let path = f.unwrap().path();
                if path.extension().unwrap() == "png" {
                    fs::remove_file(path).unwrap();
                }
            }
        }
        _ => fs::create_dir_all(FOLDER).expect("Failed to create directory"),
    };

    //* Run simulation
    println!("Running simulation");
    {
        let mut gravitons = gravitons::spawn(&mut universe, GRAVITON.quantity);
        println!("Spawning {} gravitons", gravitons.len());
        universe
            .to_image()
            .save(format!("{}/{:04}.png", FOLDER, 0))
            .expect("Failed to save image");

        let steps = add_pbar(
            ProgressBar::new(GRAVITON.life_span as u64)
                .with_style(PBAR_STYLE.clone())
                .with_prefix("Gravitons' age"),
        );
        for i in 0..GRAVITON.life_span {
            gravitons = gravitons::advance(&mut universe, gravitons);
            universe
                .to_image()
                .save(format!("{}/{:04}.png", FOLDER, i + 1))
                .expect("Failed to save image");
            steps.inc(1);
        }
        steps.finish();
        {
            let elapsed = steps.elapsed();
            let mut seconds = elapsed.as_secs();
            let mut text = "Simulation took ".to_string();
            if seconds > 3600 {
                text += &format!("{} hour(s) ", seconds / 3600);
                seconds %= 3600;
            }
            if seconds > 60 {
                text += &format!("{} minute(s) and ", seconds / 60);
                seconds %= 60;
            }
            text += &format!("{},{:03} second(s)", seconds, elapsed.subsec_millis());
            println!("{}", text);
        }
    }

    //* Join images into video
    println!("Joining images into video");
    {
        let mut cmd = Command::new("ffmpeg");
        let params = [
            "-framerate",
            "30",
            "-i",
            &format!("{FOLDER}/%04d.png"),
            "-c:v",
            "libx264",
            "-pix_fmt",
            "yuv420p",
            "-vf",
            "\"scale=trunc(iw/2)*2:trunc(ih/2)*2\"",
            "./output.mp4",
        ];
        cmd.args(params);
        let output = cmd.output();

        match output {
            Err(err) => {
                eprintln!("Failed to join images into video because {}", err);
                println!("Run `ffmpeg {:?}` manually", params);
            }
            Ok(output) => println!("{:?}", output.stdout),
        }
    }
}

pub struct ParticleParameters {
    pub quantity: u32,
    pub life_span: u32,
}

impl ParticleParameters {
    pub const fn new(quantity: u32, life_span: u32) -> Self {
        Self {
            quantity,
            life_span,
        }
    }
}

static GRAVITON: ParticleParameters = ParticleParameters::new(128, 400);
static SUB_GRAVITON: ParticleParameters = ParticleParameters::new(0, 50);
const FOLDER: &str = "output";

mod gravitons {
    use pbar::{ProgressBar, ProgressIterator};

    use crate::{PBAR_STYLE, SUB_GRAVITON, add_pbar, flood_add_field};

    use super::{Particle, Point, Universe};

    use core::f64::consts::TAU;

    pub fn spawn(universe: &mut Universe, ammount_per_mass_point: u32) -> Box<[Particle]> {
        let inv = 1.0 / ammount_per_mass_point as f64;
        let k: f64 = TAU * inv;
        let mut particles: Vec<Particle> = Vec::new();
        let directions: Box<[Point]> = (0..ammount_per_mass_point)
            .map(|i| Point::from_angle(i as f64 * k))
            .collect();
        for y in 0..universe.height() {
            for x in 0..universe.width() {
                let (.., element) = universe[(x, y)].element().unwrap();
                let mass = element.mass.value;
                if !mass.is_normal() {
                    continue;
                }
                let position = Point {
                    x: x as f64,
                    y: y as f64,
                };
                for i in 0..ammount_per_mass_point as usize {
                    particles.push(Particle {
                        position,
                        velocity: *unsafe { directions.get_unchecked(i) },
                        value: mass * inv,
                    });
                }
            }
        }
        particles.into()
    }

    pub fn advance(universe: &mut Universe, mut particles: Box<[Particle]>) -> Box<[Particle]> {
        let mut fill_mask = Universe::field::<bool>(universe);
        let len = particles.len();
        let progress = add_pbar(
            ProgressBar::new(len as u64)
                .with_style(PBAR_STYLE.clone())
                .with_prefix("Graviton index"),
        );
        particles
            .iter_mut()
            .progress_with(progress)
            .filter_map(|particle| {
                particle.move_in_universe_mut(universe);
                if !particle.position.is_inside(universe) {
                    return None;
                }
                //* spawn field
                let mass = particle.value;
                flood_add_field(
                    universe,
                    particle.position,
                    mass,
                    SUB_GRAVITON.life_span,
                    &mut fill_mask,
                );
                Some(*particle)
            })
            .collect()
    }
}

#[inline]
fn flood_add_field(
    universe: &mut Universe,
    position: Point,
    value: f64,
    limit: u32,
    fill_mask: &mut UniverseData<bool>,
) {
    // Which elements where filled already
    fill_mask.fill(false);
    // The recipients for the next level
    let mut stack_next: Vec<Point> = vec![position];

    // level in ["1"; "limit"] ("limit" levels in total)
    for _ in 1..=limit {
        // The recipients for the current level
        let stack = stack_next;
        stack_next = Vec::new();

        // For each element of the stack
        for pos in stack {
            // If already filled, ignore
            let (.., mask) = fill_mask.get_from_point_mut(pos).unwrap();
            if *mask {
                continue;
            }
            // Set fill level
            *mask = true;
            let (.., element) = universe.get_from_point_mut(pos).unwrap();
            element.mass.field += (position - pos).direction() * value;

            // For each neighbour
            for neighbour_pos in fill_mask.neighbouring_positions(pos) {
                stack_next.push(neighbour_pos);
            }
        }
    }
}
