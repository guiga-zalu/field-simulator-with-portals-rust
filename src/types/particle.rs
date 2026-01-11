use super::{point::Point, universe::Universe};

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Particle {
    pub position: Point,
    pub velocity: Point,
    pub value: f64,
}

impl Particle {
    pub fn move_in_universe_mut(&mut self, universe: &Universe) {
        let (position, speed) = universe.in_universe_move(self.position, self.velocity);
        self.position = position;
        self.velocity = speed;
    }
}
