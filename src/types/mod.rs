mod point;
mod portal;
mod universe;
pub use self::{
    point::Point,
    portal::Portal,
    universe::{Universe, UniverseData},
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

pub struct Scale(pub f64);

#[derive(Debug, Clone)]
pub enum Region<T>
where
    T: Copy,
{
    Element(T),
    // Subdivision {
    //     scale: Scale,
    //     regions: Regions,
    // },
}

impl<T> Default for Region<T>
where
    T: Copy + Default,
{
    fn default() -> Region<T> {
        Region::Element(Default::default())
    }
}

impl<T> Region<T>
where
    T: Copy,
{
    pub fn element(&self) -> Option<(Scale, T)> {
        match self {
            Region::Element(e) => Some((Scale(1.0), *e)),
            // _ => None,
        }
    }

    pub fn element_mut(&mut self) -> Option<(Scale, &mut T)> {
        match self {
            Region::Element(e) => Some((Scale(1.0), e)),
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

type Regions<T> = Vec<Region<T>>;

pub trait RegionBounds {
    fn max_point(&self) -> Point;
    fn contains(&self, point: &Point) -> bool {
        let max_point = self.max_point();
        (0.0..max_point.x).contains(&point.x) && (0.0..max_point.y).contains(&point.y)
    }
}

impl<T> RegionBounds for UniverseData<T>
where
    T: Copy,
{
    fn max_point(&self) -> Point {
        Point {
            x: self.width as f64,
            y: self.height as f64,
        }
    }
}

impl RegionBounds for Universe {
    fn max_point(&self) -> Point {
        Point {
            x: self.width() as f64,
            y: self.height() as f64,
        }
    }
}

impl Point {
    pub fn is_inside<Structure>(&self, structure: &Structure) -> bool
    where
        Structure: RegionBounds,
    {
        structure.contains(self)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Particle {
    pub position: Point,
    pub direction: Point,
    pub value: f64,
}

impl Particle {
    pub fn move_in_universe_mut(&mut self, universe: &Universe) {
        let (position, speed) = universe.in_universe_move(self.position, self.direction);
        self.position = position;
        self.direction = speed;
    }
}
