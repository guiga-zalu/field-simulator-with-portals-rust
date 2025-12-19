use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign},
};

#[derive(Debug, Clone, Default, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn by_x(x: f64) -> Point {
        Point { x, y: 0.0 }
    }

    pub fn by_y(y: f64) -> Point {
        Point { x: 0.0, y }
    }

    pub fn from_angle(angle: f64) -> Point {
        Point {
            x: angle.cos(),
            y: angle.sin(),
        }
    }

    pub fn magnitude(&self) -> f64 {
        self.x.hypot(self.y)
    }
    pub fn magnitude_2(&self) -> f64 {
        self.x.powi(2) + self.y.powi(2)
    }
    pub fn direction(&self) -> Point {
        self / self.magnitude()
    }

    pub fn inv(&self) -> Point {
        Point {
            x: self.x,
            y: -self.y,
        } / self.magnitude_2()
    }
}

impl Add<(f64, f64)> for Point {
    type Output = Self;

    fn add(self, other: (f64, f64)) -> Self {
        Point {
            x: self.x + other.0,
            y: self.y + other.1,
        }
    }
}
impl Sub<(f64, f64)> for Point {
    type Output = Self;

    fn sub(self, other: (f64, f64)) -> Self {
        Point {
            x: self.x - other.0,
            y: self.y - other.1,
        }
    }
}
impl Add for Point {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Point {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}
impl Sub for Point {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Point {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}
impl Mul<f64> for Point {
    type Output = Self;

    fn mul(self, other: f64) -> Self {
        Point {
            x: self.x * other,
            y: self.y * other,
        }
    }
}
impl Div<f64> for Point {
    type Output = Self;

    fn div(self, other: f64) -> Self {
        Point {
            x: self.x / other,
            y: self.y / other,
        }
    }
}
impl Mul<Point> for Point {
    type Output = Point;

    fn mul(self, other: Point) -> Point {
        Point {
            x: self.x * other.x - self.y * other.y,
            y: self.x * other.y + self.y * other.x,
        }
    }
}
impl Div<Point> for Point {
    type Output = Point;

    fn div(self, other: Point) -> Point {
        self * other.inv()
    }
}
impl Add for &Point {
    type Output = Point;

    fn add(self, other: Self) -> Point {
        Point {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}
impl Sub for &Point {
    type Output = Point;

    fn sub(self, other: Self) -> Point {
        Point {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}
impl Mul<f64> for &Point {
    type Output = Point;

    fn mul(self, other: f64) -> Point {
        Point {
            x: self.x * other,
            y: self.y * other,
        }
    }
}
impl Div<f64> for &Point {
    type Output = Point;

    fn div(self, other: f64) -> Point {
        Point {
            x: self.x / other,
            y: self.y / other,
        }
    }
}
impl Mul<Point> for &Point {
    type Output = Point;

    fn mul(self, other: Point) -> Point {
        Point {
            x: self.x * other.x - self.y * other.y,
            y: self.x * other.y + self.y * other.x,
        }
    }
}
impl Div<Point> for &Point {
    type Output = Point;

    fn div(self, other: Point) -> Point {
        self * other.inv()
    }
}
impl AddAssign for Point {
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
    }
}
impl SubAssign for Point {
    fn sub_assign(&mut self, other: Self) {
        self.x -= other.x;
        self.y -= other.y;
    }
}
impl MulAssign<f64> for Point {
    fn mul_assign(&mut self, other: f64) {
        self.x *= other;
        self.y *= other;
    }
}
impl DivAssign<f64> for Point {
    fn div_assign(&mut self, other: f64) {
        self.x /= other;
        self.y /= other;
    }
}

impl Display for Point {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "Point ({}, {})", self.x, self.y)
    }
}
