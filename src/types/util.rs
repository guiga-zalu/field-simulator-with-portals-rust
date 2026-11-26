use super::point::Point;

pub struct Scale(pub f64);

#[derive(Debug, Clone, Default, Copy, PartialEq)]
pub struct Property {
    pub value: f64,
    pub field: Point,
}

#[derive(Debug, Clone, Default, Copy, PartialEq)]
pub struct Element {
    pub mass: Property,
}
