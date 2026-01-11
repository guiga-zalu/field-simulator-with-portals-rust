pub(crate) mod particle;
pub(crate) mod point;
pub(crate) mod portal;
pub(crate) mod region;
pub(crate) mod universe;

pub use self::{
    point::Point,
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
