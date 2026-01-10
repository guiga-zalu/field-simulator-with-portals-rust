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
            gravitons = gravitons::advance(&mut universe, gravitons);
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

static GRAVITON: ParticleParameters = ParticleParameters::new(512, 800);
static SUB_GRAVITON: ParticleParameters = ParticleParameters::new(0, 100);
const FOLDER: &str = "output";

mod gravitons {
    use crate::{SUB_GRAVITON, flood_add_field};

    use super::types::{Particle, Point, Universe};

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
                        direction: *unsafe { directions.get_unchecked(i) },
                        value: mass * inv,
                    });
                }
            }
        }
        particles.into()
    }

    pub fn advance(universe: &mut Universe, mut particles: Box<[Particle]>) -> Box<[Particle]> {
        let mut fill_mask = Universe::field::<bool>(universe);
        particles
            .iter_mut()
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

// /// Floods "pool", starting at "start_pos" with the value "limit", and a lower value at each step, until the value is zero.
// fn flood_until(
//     pool: &mut (Vec<f32>, usize, usize),
//     start_pos: (usize, usize),
//     limit: usize,
//     value_to_plant: f32,
// ) {
//     let data = &mut pool.0;
//     let width = pool.1;
//     // The recipients for the next level
//     let mut stack_next: Vec<(usize, usize)> = vec![start_pos];
//     // Which elements where filled already
//     let mut fill_mask = vec![0; data.len()];

//     for level in 0..limit {
//         // level from "limit" to "1" ("limit" levels in total)
//         let level = limit - level;

//         // The recipients for the current level
//         let stack = stack_next;
//         stack_next = Vec::new();

//         // For each element of the stack
//         for (x, y) in stack {
//             // If already filled, ignore
//             if fill_mask[y * width + x] > 0 {
//                 continue;
//             }
//             // Set fill level
//             fill_mask[y * width + x] = level;
//             data[y * width + x] += value_to_plant;
//             // For each neighbour
//             for dx in -1..=1 {
//                 for dy in -1..=1 {
//                     let nx = x as isize + dx;
//                     let ny = y as isize + dy;
//                     // If it exists
//                     if (0..64).contains(&nx) && (0..64).contains(&ny) {
//                         stack_next.push((nx as usize, ny as usize));
//                     }
//                 }
//             }
//         }
//     }
// }
