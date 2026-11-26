pub(crate) mod particle;
pub(crate) mod point;
pub(crate) mod portal;
pub(crate) mod region;
pub(crate) mod universe;
pub(crate) mod util;

pub use self::{
    point::Point,
    universe::{Universe, UniverseData},
};

pub trait RegionBounds {
    fn contains(&self, point: &Point) -> bool;
}

impl<T: Copy> RegionBounds for UniverseData<T> {
    #[inline(always)]
    fn contains(&self, point: &Point) -> bool {
        !(point.x < 0.0
            || point.y < 0.0
            || point.x >= self.width as f64
            || point.y >= self.height as f64)
    }
}

impl RegionBounds for Universe {
    #[inline(always)]
    fn contains(&self, point: &Point) -> bool {
        !(point.x < 0.0
            || point.y < 0.0
            || point.x >= self.width() as f64
            || point.y >= self.height() as f64)
    }
}

impl Point {
    #[inline(always)]
    pub fn is_inside<Structure>(&self, structure: &Structure) -> bool
    where
        Structure: RegionBounds,
    {
        structure.contains(self)
    }
}
