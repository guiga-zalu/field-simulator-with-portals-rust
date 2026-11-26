use super::{point::Point, util::Scale};

use std::marker::ConstParamTy_;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PortalInteraction {
    Cross,
    Ignore,
    Wall,
}

impl ConstParamTy_ for PortalInteraction {}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Portal {
    pub point_a: Point,
    pub point_b: Point,
}

impl Portal {
    pub const fn new(a: Point, b: Point) -> Portal {
        Portal {
            point_a: a,
            point_b: b,
        }
    }

    pub fn scale(&self) -> Scale {
        Scale(self.delta().magnitude())
    }

    pub const fn relative_position(&self, point: Point) -> Point {
        point.const_sub(self.point_a).const_div(self.delta())
    }
    pub const fn reverted_relative_position(&self, point: Point) -> Point {
        point.const_mul(self.delta()).const_add(self.point_a)
    }

    #[inline]
    pub const fn delta(&self) -> Point {
        self.point_b.const_sub(self.point_a)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PortalSet {
    pub a: Portal,
    pub b: Portal,
}

impl PortalSet {
    pub const fn new(a: Portal, b: Portal) -> PortalSet {
        PortalSet { a, b }
    }

    pub const fn relative_positions(&self, point: Point) -> [Point; 2] {
        [
            self.a.relative_position(point),
            self.b.relative_position(point),
        ]
    }

    pub fn cross_part_1(&self, point: Point, movement: Point) -> Option<[(Portal, Point); 2]> {
        let [before_a, before_b] = self.relative_positions(point);
        let [after_a, after_b] = self.relative_positions(point + movement);
        // ! Non const
        let scale_a = self.a.scale();
        // ! Non const
        let scale_b = self.b.scale();
        /*
         * distance_A = if is crossing A { before_a.y * cmp_a.radius } else { None }
         * distance_B = if is crossing B { before_b.y * cmp_b.radius } else { None }
         * if distance_A.abs() > distance_B.abs() { (a, b) } else { (b, a) }
         */
        let distance_a = if before_a.y * after_a.y < 0.0 {
            Some(before_a.y * scale_a.0)
        } else {
            None
        };
        let distance_b = if before_b.y * after_b.y < 0.0 {
            Some(before_b.y * scale_b.0)
        } else {
            None
        };
        Some(match (distance_a, distance_b) {
            (None, None) => return None,
            (Some(_), None) => [(self.a, before_a), (self.b, after_b)],
            (None, Some(_)) => [(self.b, before_b), (self.a, after_a)],
            (Some(distance_a), Some(distance_b)) => {
                if distance_a.abs() < distance_b.abs() {
                    [(self.a, before_a), (self.b, after_b)]
                } else {
                    [(self.b, before_b), (self.a, after_a)]
                }
            }
        })
    }

    /// https://en.wikipedia.org/wiki/Line%E2%80%93line_intersection#Given_two_points_on_each_line
    pub const fn cross_part_2(
        &self,
        speed: Point,
        crossing: [(Portal, Point); 2],
    ) -> Option<(Point, Point, Point)> {
        let [(being_crossed, before), (exiting, after)] = crossing;
        //* {space = entry portal}
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

        let x = (before.x * after.y - before.y * after.x) / (after.y - before.y);
        // We want x in [0; 1]
        if 0.0 <= x && x <= 1.0 {
            // Not being crossed.
            return None;
        }
        let crossing_point = Point::by_x(x);
        let yet_to_move = after.const_sub(crossing_point);

        //* {space = real space}
        let reprojected = exiting.reverted_relative_position(crossing_point);
        let projection_entry_to_exit = exiting.delta().const_div(being_crossed.delta());
        let speed = speed.const_mul(projection_entry_to_exit);
        // {space = entry portal}
        // remaining := after - crossing point
        let yet_to_move = exiting.reverted_relative_position(yet_to_move);
        Some((reprojected, speed, yet_to_move))
    }
}

impl From<(Portal, Portal)> for PortalSet {
    fn from(value: (Portal, Portal)) -> Self {
        PortalSet::new(value.0, value.1)
    }
}
impl From<[Portal; 2]> for PortalSet {
    fn from(value: [Portal; 2]) -> Self {
        PortalSet::new(value[0], value[1])
    }
}
