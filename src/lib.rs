#![allow(incomplete_features)]
#![feature(unsized_const_params)]
#![feature(adt_const_params)]

mod types;

pub use types::{
    particle::Particle,
    point::Point,
    portal::{Portal, PortalInteraction},
    region::Region,
    universe::{Universe, UniverseData},
};
