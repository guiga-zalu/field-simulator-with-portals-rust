mod types;
use types::*;

use std::{fs, process::Command};

fn main() {
    //* Preparating simulation universe
    println!("Preparating simulation universe...");
    let mut universe = {
        const WIDTH: u32 = 540 - 1;
        const HEIGHT: u32 = 540 - 1;
        const OBJECT_RADIUS: f64 = 4.0;
        let mut universe = Universe::new(WIDTH, HEIGHT);

        let center = Point {
            x: universe.width as f64 / 2.0,
            y: universe.height as f64 / 2.0,
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
                    universe[(x, y)].element_mut().unwrap().mass.value = 1.0;
                }
            }
        }

        let portalset = {
            let delta = universe.width as f64 / 6.0;
            let portal_height = universe.height as f64 / 2.0;
            PortalSet::new(
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
        let mut gravitons = gravitons::spawn(&mut universe, GRAVITON.quantity, GRAVITON.step_size);
        universe
            .to_image()
            .save(format!("{}/{:04}.png", FOLDER, 0))
            .expect("Failed to save image");
        for i in 0..GRAVITON.life_span {
            println!(
                "Step {} / {} ≃ {}%",
                i + 1,
                GRAVITON.life_span,
                (i + 1) * 100 / GRAVITON.life_span
            );
            gravitons = gravitons::advance(
                &mut universe,
                gravitons,
                SUB_GRAVITON.quantity,
                SUB_GRAVITON.step_size,
            );
            universe
                .to_image()
                .save(format!("{}/{:04}.png", FOLDER, i + 1))
                .expect("Failed to save image");
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
        if cmd.output().is_err() {
            eprintln!("Failed to join images into video");
            println!("Run `ffmpeg {:?}` manually", params);
        }
    }
}

pub struct ParticleParameters {
    pub step_size: f64,
    pub quantity: u32,
    pub life_span: u32,
}

impl ParticleParameters {
    pub const fn new(step_size: f64, quantity: u32, life_span_steps: u32) -> Self {
        Self {
            step_size,
            quantity,
            life_span: (life_span_steps as f64 / step_size).ceil() as u32,
            // life_span: life_span_steps,
        }
    }
}

static GRAVITON: ParticleParameters = ParticleParameters::new(3.0, 512, 800);
static SUB_GRAVITON: ParticleParameters = ParticleParameters::new(0.9, 256, 100);
const FOLDER: &str = "output";

mod gravitons {
    use crate::process_sub_graviton;

    use super::types::{Particle, Point, Universe};

    use core::f64::consts::TAU;

    pub fn spawn(
        universe: &mut Universe,
        ammount_per_mass_point: u32,
        particle_speed: f64,
    ) -> Box<[Particle]> {
        let inv = 1.0 / ammount_per_mass_point as f64;
        let k: f64 = TAU * inv;
        let mut particles: Vec<Particle> = Vec::new();
        let speeds: Box<[Point]> = (0..ammount_per_mass_point)
            .map(|i| Point::from_angle(i as f64 * k) * particle_speed)
            .collect();
        for y in 0..universe.height {
            for x in 0..universe.width {
                let element = universe[(x, y)].element().unwrap();
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
                        speed: *unsafe { speeds.get_unchecked(i) },
                        value: mass * inv,
                    });
                }
            }
        }
        particles.into()
    }

    pub fn advance(
        universe: &mut Universe,
        mut particles: Box<[Particle]>,
        ammount_per_particle: u32,
        sub_particle_speed: f64,
    ) -> Box<[Particle]> {
        let inv = 1.0 / ammount_per_particle as f64;
        let k: f64 = TAU * inv;
        let directions: Box<[Point]> = (0..ammount_per_particle)
            .map(|i| Point::from_angle(i as f64 * k) * sub_particle_speed)
            .collect();

        particles
            .iter_mut()
            .filter_map(|particle| {
                particle.move_in_universe_mut(universe);
                if !particle.position.is_inside(universe) {
                    return None;
                }
                //* spawn field
                let mass = particle.value * inv;
                for i in 0..ammount_per_particle as usize {
                    process_sub_graviton(
                        universe,
                        particle.position,
                        *unsafe { directions.get_unchecked(i) },
                        mass,
                    );
                }
                Some(*particle)
            })
            .collect()
    }
}

#[inline]
fn process_sub_graviton(
    universe: &mut Universe,
    position: Point,
    dir_sub_graviton: Point,
    mass: f64,
) {
    let mut position = position;
    let mut dir_sub_graviton = dir_sub_graviton * SUB_GRAVITON.step_size;
    for _age in 0..SUB_GRAVITON.life_span {
        if !position.is_inside(universe) {
            return;
        }
        let element = universe.get_from_point_mut(position).unwrap();
        element.mass.field -= dir_sub_graviton * mass;
        // Advance sub-graviton's position
        (position, dir_sub_graviton) = universe.move_in_universe(position, dir_sub_graviton);
    }
}
