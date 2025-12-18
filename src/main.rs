mod types;
use types::*;

use std::{f64::consts::TAU, fs::remove_dir_all, process::Command};

fn main() {
    const WIDTH: u32 = 1080 - 1;
    const HEIGHT: u32 = 1080 - 1;
    let mut universe = Universe::new(WIDTH, HEIGHT);

    let center = Point {
        x: universe.width as f64 / 2.0,
        y: universe.height as f64 / 2.0,
    };
    let t = (WIDTH as f64 * 0.125) as u32;
    universe[(t, HEIGHT >> 1)].element_mut().unwrap().mass.value = 1.0;
    universe[(WIDTH - t, HEIGHT >> 1)]
        .element_mut()
        .unwrap()
        .mass
        .value = 1.0;

    let portalset = {
        let delta = universe.width as f64 / 4.0;
        let portal_height = universe.height as f64 / 2.0;
        PortalSet::new(
            Portal::new(
                center + (-delta, -portal_height / 2.0),
                center + (-delta, portal_height / 2.0),
            ),
            Portal::new(
                center + (delta, -portal_height / 2.0),
                center + (delta, portal_height / 2.0),
            ),
        )
    };
    println!(
        "Portalset {{ {} - {} <-> {} - {} }}",
        portalset.a.point_a, portalset.a.point_b, portalset.b.point_a, portalset.b.point_b
    );
    universe.add_portal_set(portalset);

    //* Remove previous images
    remove_dir_all(FOLDER).expect("Failed to remove previous images");

    //* Run simulation
    universe
        .to_image()
        .save(format!("{}/{:04}.png", FOLDER, 0))
        .expect("Failed to save image");
    for i in 0..GRAVITON.quantity {
        let direction_graviton = Point::from_angle(TAU * i as f64 / GRAVITON.quantity as f64);
        process_gravitons(&mut universe, direction_graviton /* , i*/);
        universe
            .to_image()
            .save(format!("{}/{:04}.png", FOLDER, i + 1))
            .expect("Failed to save image");
    }

    //* Join images into video
    let mut cmd = Command::new("ffmpeg");
    cmd.args([
        "-framerate",
        "30",
        "-i",
        &format!("{FOLDER}/%04d.png"),
        "-c:v",
        "libx264",
        "-pix_fmt",
        "yuv420p",
        "-vf",
        "scale=trunc(iw/2)*2:trunc(ih/2)*2",
        "output.mp4",
    ]);
    cmd.output().expect("Failed to join images into video");
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

static GRAVITON: ParticleParameters = ParticleParameters::new(1.8, 256, 800);
static SUB_GRAVITON: ParticleParameters = ParticleParameters::new(0.9, 256, 600);
const FOLDER: &str = "output";

#[inline]
fn process_gravitons(universe: &mut Universe, direction_graviton: Point) {
    for y in 0..universe.height {
        for x in 0..universe.width {
            let element = universe[(x, y)].element().unwrap();
            let mass = element.mass.value;
            if !mass.is_normal() {
                continue;
            }
            for_each_mass_point(universe, (x, y), direction_graviton, mass);
        }
    }
}

#[inline]
/// Summons and processes a graviton
fn for_each_mass_point(
    universe: &mut Universe,
    position: (u32, u32),
    dir_graviton: Point,
    mass: f64,
) {
    let mut dir_graviton = dir_graviton * GRAVITON.step_size;
    let mut position = Point {
        x: position.0 as f64,
        y: position.1 as f64,
    };
    let mass = mass / SUB_GRAVITON.quantity as f64;
    for _age in 0..GRAVITON.life_span {
        if !position.is_inside(universe) {
            return;
        }
        for _subgrv_idx in 0..SUB_GRAVITON.quantity {
            let direction_sub_graviton =
                Point::from_angle(TAU * _subgrv_idx as f64 / SUB_GRAVITON.quantity as f64);
            process_sub_graviton(universe, position, direction_sub_graviton, mass);
        }
        // Advance graviton's position
        (position, dir_graviton) = universe.move_in_universe(position, dir_graviton);
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
        (position, dir_sub_graviton) =
            universe.move_in_universe(position, dir_sub_graviton /* , test*/);
    }
}
