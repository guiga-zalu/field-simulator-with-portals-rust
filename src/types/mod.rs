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

// #[derive(Debug, Clone, Copy, PartialEq)]
// /// [ R T ] × [ r t ] = [ Rr (Rr + Ta) ]
// /// [ 0 1 ]   [ 0 1 ]   [ 0      1     ]
// ///
// ///
// pub struct Transform {
//     pub matrix: [[f64; 3]; 3],
// }

// impl Transform {
//     pub fn zero() -> Transform {
//         Transform {
//             matrix: [[0.0; 3], [0.0; 3], [0.0, 0.0, 1.0]],
//         }
//     }
//     pub fn identity() -> Transform {
//         Transform {
//             matrix: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
//         }
//     }

//     pub fn determinant(&self) -> f64 {
//         // /*
//         // 00 01 02
//         // 10 11 12
//         // 20 21 22
//         // */
//         // self.matrix[0][0]
//         //     * (self.matrix[1][1] * self.matrix[2][2] - self.matrix[1][2] * self.matrix[2][1])
//         //     - self.matrix[0][1]
//         //         * (self.matrix[1][0] * self.matrix[2][2] - self.matrix[1][2] * self.matrix[2][0])
//         //     + self.matrix[0][2]
//         //         * (self.matrix[1][0] * self.matrix[2][1] - self.matrix[1][1] * self.matrix[2][0])
//         // [2] = [0.0, 0.0, 1.0] =>
//         self.matrix[0][0] * self.matrix[1][1] - self.matrix[0][1] * self.matrix[1][0]
//     }
//     // Inverts the 3x3 matrix
//     pub fn inverse(&self) -> Option<Transform> {
//         let det = self.determinant();
//         if det == 0.0 {
//             return None;
//         }
//         // let [[a, b, c], [d, e, f], [g, h, i]] = self.matrix;
//         // let matrix = [
//         //     [
//         //         (e * i - f * h) / det,
//         //         (c * h - b * i) / det,
//         //         (b * f - c * e) / det,
//         //     ],
//         //     [
//         //         (f * g - d * i) / det,
//         //         (a * i - c * g) / det,
//         //         (c * d - a * f) / det,
//         //     ],
//         //     [
//         //         (d * h - e * g) / det,
//         //         (b * g - a * h) / det,
//         //         (a * e - b * d) / det,
//         //     ],
//         // ];
//         // [2] = [0.0, 0.0, 1.0] =>
//         let [[a, b, c], [d, e, f], ..] = self.matrix;
//         let _det = 1.0 / det;
//         let matrix = [
//             [e * _det, -b * _det, (b * f - c * e) * _det],
//             [-d * _det, a * _det, (c * d - a * f) * _det],
//             [0.0, 0.0, 1.0],
//         ];
//         Some(Transform { matrix })
//     }

//     pub fn from_rotation(angle: f64) -> Transform {
//         Transform {
//             matrix: [
//                 [angle.cos(), angle.sin(), 0.0],
//                 [-angle.sin(), angle.cos(), 0.0],
//                 [0.0, 0.0, 1.0],
//             ],
//         }
//     }
//     pub fn from_translation(point: Point) -> Transform {
//         Transform {
//             matrix: [[1.0, 0.0, point.x], [0.0, 1.0, point.y], [0.0, 0.0, 1.0]],
//         }
//     }

//     pub fn from_basis(a: Point, b: Point) -> Transform {
//         Transform {
//             matrix: [[a.x, b.x, 0.0], [a.y, b.y, 0.0], [0.0, 0.0, 1.0]],
//         }
//     }
// }

// impl Default for Transform {
//     fn default() -> Self {
//         Transform::identity()
//     }
// }

// impl Mul<Transform> for Transform {
//     type Output = Self;
//     fn mul(self, other: Transform) -> Transform {
//         let mut out = Transform::identity();
//         // for i in 0..3 {
//         // [2] = [0.0, 0.0, 1.0] =>
//         for i in 0..2 {
//             for j in 0..3 {
//                 out.matrix[i][j] = self.matrix[i][0] * other.matrix[0][j]
//                     + self.matrix[i][1] * other.matrix[1][j]
//                     + self.matrix[i][2] * other.matrix[2][j];
//             }
//         }
//         out
//     }
// }
// impl Mul<Point> for Transform {
//     type Output = Point;
//     fn mul(self, other: Point) -> Point {
//         let x = self.matrix[0][0] * other.x + self.matrix[0][1] * other.y + self.matrix[0][2];
//         let y = self.matrix[1][0] * other.x + self.matrix[1][1] * other.y + self.matrix[1][2];
//         Point { x, y }
//     }
// }

impl Point {
    pub fn is_inside(&self, universe: &Universe) -> bool {
        (self.x >= 0.0 && self.x < universe.width as f64)
            && (self.y >= 0.0 && self.y < universe.height as f64)
    }
}
