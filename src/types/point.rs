#![allow(clippy::suspicious_arithmetic_impl)]
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
    pub const fn zero() -> Point {
        Point { x: 0.0, y: 0.0 }
    }

    pub const fn by_x(x: f64) -> Point {
        Point { x, y: 0.0 }
    }

    pub const fn by_y(y: f64) -> Point {
        Point { x: 0.0, y }
    }

    pub fn from_angle(angle: f64) -> Point {
        Point {
            x: angle.cos(),
            y: angle.sin(),
        }
    }

    pub fn into_angle(self) -> f64 {
        self.y.atan2(self.x)
    }

    pub fn into_angle_percentage(self) -> f64 {
        let angle = self.into_angle() / std::f64::consts::TAU;
        if angle.is_sign_negative() {
            angle + 1.0
        } else {
            angle
        }
    }

    pub fn magnitude(&self) -> f64 {
        self.x.hypot(self.y)
    }
    pub const fn magnitude_2(&self) -> f64 {
        (self.x * self.x) + (self.y * self.y)
    }
    pub fn direction(&self) -> Point {
        self / self.magnitude()
    }

    pub const fn inv(&self) -> Point {
        let _mag2 = 1.0 / self.magnitude_2();
        Point {
            x: self.x * _mag2,
            y: -self.y * _mag2,
        }
    }
}

impl Point {
    pub const fn const_add(self, other: Point) -> Point {
        Point {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
    pub const fn const_sub(self, other: Point) -> Point {
        Point {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
    pub const fn const_mul(self, other: Point) -> Point {
        Point {
            x: self.x * other.x - self.y * other.y,
            y: self.x * other.y + self.y * other.x,
        }
    }
    pub const fn const_div(self, other: Point) -> Point {
        self.const_mul(other.inv())
    }

    pub const fn const_add_assign(&mut self, other: Point) {
        self.x += other.x;
        self.y += other.y;
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
        self.const_add(other)
    }
}
impl Sub for Point {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        self.const_sub(other)
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
        self.const_mul(other)
    }
}
impl Div<Point> for Point {
    type Output = Point;

    fn div(self, other: Point) -> Point {
        self.const_div(other)
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
        self.const_mul(other)
    }
}
impl Div<Point> for &Point {
    type Output = Point;

    fn div(self, other: Point) -> Point {
        self.const_div(other)
    }
}
impl AddAssign for Point {
    fn add_assign(&mut self, other: Self) {
        self.const_add_assign(other);
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
        let _other = 1.0 / other;
        *self *= _other;
    }
}

impl Display for Point {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "Point ({}, {})", self.x, self.y)
    }
}
