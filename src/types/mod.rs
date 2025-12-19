mod point;
mod portal;
mod universe;
pub use self::{
    point::Point,
    portal::{Portal, PortalSet},
    universe::Universe,
};

#[derive(Debug, Clone, Default, Copy, PartialEq)]
pub struct Property {
    pub value: f64,
    pub field: Point,
}

#[derive(Debug, Clone, Default, Copy, PartialEq)]
pub struct Element {
    pub mass: Property,
}

#[derive(Debug, Clone)]
pub enum Region {
    Element(Element),
    // Subdivision(Regions),
}

impl Default for Region {
    fn default() -> Region {
        Region::Element(Default::default())
    }
}

impl Region {
    pub fn element(&self) -> Option<Element> {
        match self {
            Region::Element(e) => Some(*e),
            // _ => None,
        }
    }

    pub fn element_mut(&mut self) -> Option<&mut Element> {
        match self {
            Region::Element(e) => Some(e),
            // _ => None,
        }
    }

    // pub fn subdivision(&self) -> Option<Regions> {
    //     match self {
    //         Region::Subdivision(s) => Some(s),
    //         _ => None,
    //     }
    // }

    // pub fn count(&self) -> usize {
    //     match self {
    //         Region::Element(_) => 1,
    //         // Region::Subdivision(s) => s.map(|e| e.count()).sum(),
    //     }
    // }
}

type Regions = Vec<Region>;

impl Point {
    pub fn is_inside(&self, universe: &Universe) -> bool {
        (self.x >= 0.0 && self.x < universe.width as f64)
            && (self.y >= 0.0 && self.y < universe.height as f64)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct Particle {
    pub position: Point,
    pub speed: Point,
    pub value: f64,
}

impl Particle {
    pub fn move_in_universe_mut(&mut self, universe: &Universe) {
        let (position, speed) = universe.move_in_universe(self.position, self.speed);
        self.position = position;
        self.speed = speed;
    }
}
