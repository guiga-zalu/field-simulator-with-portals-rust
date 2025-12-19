use super::point::Point;

use core::ops::{Div, Mul};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ComplexPolar {
    pub radius: f64,
    pub angle: f64,
}

impl Mul<ComplexPolar> for ComplexPolar {
    type Output = ComplexPolar;
    fn mul(self, other: ComplexPolar) -> Self {
        ComplexPolar {
            radius: self.radius * other.radius,
            angle: self.angle + other.angle,
        }
    }
}
impl Div<ComplexPolar> for ComplexPolar {
    type Output = ComplexPolar;
    fn div(self, other: ComplexPolar) -> Self {
        ComplexPolar {
            radius: self.radius / other.radius,
            angle: self.angle - other.angle,
        }
    }
}
impl From<Point> for ComplexPolar {
    fn from(value: Point) -> Self {
        ComplexPolar {
            radius: value.magnitude(),
            angle: value.y.atan2(value.x),
        }
    }
}
impl From<ComplexPolar> for Point {
    fn from(value: ComplexPolar) -> Self {
        Point {
            x: value.radius * value.angle.cos(),
            y: value.radius * value.angle.sin(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Portal {
    pub point_a: Point,
    pub point_b: Point,
}

impl Portal {
    pub fn new(a: Point, b: Point) -> Portal {
        Portal {
            point_a: a,
            point_b: b,
        }
    }

    pub fn size(&self) -> f64 {
        (self.point_b - self.point_a).magnitude()
    }

    pub fn relative_position(&self, point: Point) -> Point {
        (point - self.point_a) / (self.point_b - self.point_a)
    }
    pub fn reverted_relative_position(&self, point: Point) -> Point {
        point * (self.point_b - self.point_a) + self.point_a
    }

    pub fn signed_distance(&self, point: Point) -> f64 {
        // Portal = [Point; 2] -> line
        // line -- T --> Ox
        // T(point).y
        self.relative_position(point).y
    }

    pub(crate) fn into_complex_polar(&self) -> ComplexPolar {
        let delta = self.point_b - self.point_a;
        delta.into()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PortalSet {
    pub a: Portal,
    pub b: Portal,
}

impl PortalSet {
    pub fn new(a: Portal, b: Portal) -> PortalSet {
        PortalSet { a, b }
    }

    pub fn sizes(&self) -> [f64; 2] {
        [self.a.size(), self.b.size()]
    }

    pub fn relative_positions(&self, point: Point) -> [Point; 2] {
        [
            self.a.relative_position(point),
            self.b.relative_position(point),
        ]
    }

    pub fn signed_distances(&self, point: Point) -> [f64; 2] {
        [self.a.signed_distance(point), self.b.signed_distance(point)]
    }

    /// https://en.wikipedia.org/wiki/Line%E2%80%93line_intersection#Given_two_points_on_each_line
    pub fn cross(
        &self,
        point: Point,
        speed: Point,
        movement: Point,
    ) -> Option<(Point, Point, Point)> {
        /*
        (x1, y1) := (0, 0)
        (x2, y2) := (1, 0)
        (x3, y3) := (Bx, By)
        (x4, y4) := (Ax, Ay)
        Py = [
            (x1 y2 - y1 x2) (y3 - y4) - (y1 - y2) (x3 y4 - y3 x4)
        ] / [
            (x1 - x2) (y3 - y4) - (y1 - y2) (x3 - x4)
        ] = 0
        Px = [
            (x1 y2 - y1 x2) (x3 - x4) - (x1 - x2) (x3 y4 - y3 x4)
        ] / [
            (x1 - x2) (y3 - y4) - (y1 - y2)(x3 - x4)
        ]
        = [ x3 y4 - y3 x4 ] / [ y4 - y3 ]
        = [ Bx Ay - By Ax ] / [ Ay - By ]
        */

        let (being_crossed, exiting): (Portal, Portal) = {
            let [before_a, before_b] = self.relative_positions(point);
            let [after_a, after_b] = self.relative_positions(point + movement);
            let cmp_a = self.a.into_complex_polar();
            let cmp_b = self.b.into_complex_polar();
            /*
             * distance_A = if is crossing A { before_a.y * cmp_a.radius } else { None }
             * distance_B = if is crossing B { before_b.y * cmp_b.radius } else { None }
             * if distance_A.abs() > distance_B.abs() { (a, b) } else { (b, a) }
             */
            let distance_a = if before_a.y * after_a.y < 0.0 {
                Some(before_a.y * cmp_a.radius)
            } else {
                None
            };
            let distance_b = if before_b.y * after_b.y < 0.0 {
                Some(before_b.y * cmp_b.radius)
            } else {
                None
            };
            match (distance_a, distance_b) {
                (None, None) => return None,
                (Some(_), None) => (self.a, self.b),
                (None, Some(_)) => (self.b, self.a),
                (Some(distance_a), Some(distance_b)) => {
                    if distance_a.abs() < distance_b.abs() {
                        (self.a, self.b)
                    } else {
                        (self.b, self.a)
                    }
                }
            }
        };

        //* {space = entry portal}
        let before = being_crossed.relative_position(point);
        let after = being_crossed.relative_position(point + movement);
        let x = (before.x * after.y - before.y * after.x) / (after.y - before.y);
        // We want x in [0; 1]
        if x < 0.0 || x > 1.0 {
            // Not being crossed.
            return None;
        }
        let crossing_point = Point::by_x(x);
        let yet_to_move = after - crossing_point;

        //* {space = real space}
        let reprojected = exiting.reverted_relative_position(crossing_point);
        let projection_entry_to_exit =
            (exiting.point_b - exiting.point_a) / (being_crossed.point_b - being_crossed.point_a);
        let speed = speed * projection_entry_to_exit;
        // {space = entry portal}
        // remaining := after - crossing point
        let yet_to_move = exiting.reverted_relative_position(yet_to_move);
        return Some((reprojected, speed, yet_to_move));
    }
}
