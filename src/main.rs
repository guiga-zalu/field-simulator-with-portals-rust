mod types;
use types::*;

use std::f64::consts::TAU;

// use image::{GenericImage, Rgba};

fn main() {
    const WIDTH: u32 = 1080 - 1;
    const HEIGHT: u32 = 1080 - 1;
    let mut universe = Universe::new(WIDTH, HEIGHT);

    let center = (universe.width >> 1, universe.height >> 1);
    universe[center].element_mut().unwrap().mass.value = 1.0;

    let portalset = {
        let delta = universe.width as f64 / 4.0;
        let portal_height = universe.height as f64 / 2.0;
        let center = Point {
            x: universe.width as f64 / 2.0,
            y: universe.height as f64 / 2.0,
        };
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

    universe
        .to_image()
        .save(format!("{}/{:04}.png", FOLDER, 0))
        .unwrap();
    for i in 0..GRAVITON.quantity {
        let direction_graviton = Point::from_angle(TAU * i as f64 / GRAVITON.quantity as f64);
        process_gravitons(&mut universe, direction_graviton /* , i*/);
        // return;
        universe
            .to_image()
            .save(format!("{}/{:04}.png", FOLDER, i + 1))
            .unwrap();
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

static GRAVITON: ParticleParameters = ParticleParameters::new(1.1, 256, 800);
static SUB_GRAVITON: ParticleParameters = ParticleParameters::new(0.9, 64, 200);
const FOLDER: &str = "output";

#[inline]
fn process_gravitons(
    universe: &mut Universe,
    direction_graviton: Point,
    // i: u32,
) {
    for y in 0..universe.height {
        for x in 0..universe.width {
            let element = universe[(x, y)].element().unwrap();
            let mass = element.mass.value;
            // println!("Mass: {}", mass);
            if !mass.is_normal() {
                continue;
            }
            for_each_mass_point(
                universe,
                (x, y),
                direction_graviton,
                mass,
                // i,
            );
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
    // i: u32,
) {
    let mut dir_graviton = dir_graviton * GRAVITON.step_size;
    // println!("Position: {}/{}", position.0, position.1);
    let mut position = Point {
        x: position.0 as f64,
        y: position.1 as f64,
    };
    let mass = mass / SUB_GRAVITON.quantity as f64;
    for _age in 0..GRAVITON.life_span {
        // println!("Graviton #{}, age {}", i, _age);
        if !position.is_inside(universe) {
            // eprintln!("ERROR: Graviton #{}, age {} is out of bounds", i, _age);
            // dbg!(position);
            // dbg!(dir_graviton);
            return;
        }
        for _subgrv_idx in 0..SUB_GRAVITON.quantity {
            // if _age == 100 && _subgrv_idx == 15 {
            //     println!("\tSub-graviton #{}", _subgrv_idx);
            //     // universe_to_image(&universe.section(
            //     //     universe.width * 3 / 5,
            //     //     universe.height * 1 / 4,
            //     //     universe.width * 2 / 5,
            //     //     universe.height * 2 / 4,
            //     // ))
            //     // .save(format!(
            //     //     "output/tmp/Grv {:04} - age {:04} = SubGrv {:04}.png",
            //     //     i, _age, _subgrv_idx
            //     // ))
            //     // .unwrap();
            // }
            let direction_sub_graviton =
                Point::from_angle(TAU * _subgrv_idx as f64 / SUB_GRAVITON.quantity as f64);
            process_sub_graviton(
                universe,
                position,
                direction_sub_graviton,
                mass,
                // i,
                // _age,
                // _subgrv_idx,
            );
        }
        // Advance graviton's position
        (position, dir_graviton) =
            universe.move_in_universe(position, dir_graviton /* , false*/);
        // if _age == 100 {
        //     universe_to_image(universe)
        //         .save(format!("output/tmp/Grv {:04} - age {:04}.png", i, _age))
        //         .unwrap();
        // }
    }
}

#[inline]
fn process_sub_graviton(
    universe: &mut Universe,
    position: Point,
    dir_sub_graviton: Point,
    mass: f64,
    // grv_idx: u32,
    // grv_age: u32,
    // subgrv_idx: u32,
) {
    let mut position = position;
    let mut dir_sub_graviton = dir_sub_graviton * SUB_GRAVITON.step_size;
    for _age in 0..SUB_GRAVITON.life_span {
        // let test = grv_idx == 0 && grv_age == 3 && subgrv_idx == 0 && _age > 165 && _age < 170;
        if !position.is_inside(universe) {
            // eprintln!(
            //     "ERROR: Sub-graviton #{}, age {} is out of bounds",
            //     subgrv_idx, _age
            // );
            // dbg!(position);
            // dbg!(dir_sub_graviton);
            return;
        }
        let element = universe.get_from_point_mut(position).unwrap();
        // element.mass.field -= dir_sub_graviton * mass * if test { 120.0 } else { 1.0 };
        element.mass.field -= dir_sub_graviton * mass;
        // if test {
        //     println!(
        //         "Grv {:04} - age {:04} = SubGrv {:04} - age {:04}",
        //         grv_idx, grv_age, subgrv_idx, _age
        //     );
        //     let img = &universe.to_image().crop_imm(
        //         universe.width * 7 / 10,
        //         universe.height * 2 / 5,
        //         universe.width * 1 / 10,
        //         universe.height * 1 / 10,
        //     );
        //     img.resize(
        //         img.width() << 2,
        //         img.height() << 2,
        //         image::imageops::FilterType::Nearest,
        //     )
        //     .save(format!(
        //         "output/tmp/Grv {:04} - age {:04} = SubGrv {:04} - age {:04}.png",
        //         grv_idx, grv_age, subgrv_idx, _age
        //     ))
        //     .unwrap();
        // }
        // Advance sub-graviton's position
        (position, dir_sub_graviton) =
            universe.move_in_universe(position, dir_sub_graviton /* , test*/);
    }
}
