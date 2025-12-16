use super::{Element, Point, Region, Regions, portal::PortalSet};

use std::ops::{Index, IndexMut};

use image::{DynamicImage, ImageBuffer, Rgb};
use rayon::prelude::*;

#[derive(Debug, Clone, Default)]
pub struct Universe {
    pub width: u32,
    pub height: u32,
    portals: Vec<PortalSet>,
    data: Regions,
}

impl Universe {
    pub fn new(width: u32, height: u32) -> Universe {
        let data = vec![Default::default(); (width * height) as usize];
        Universe {
            width,
            height,
            data,
            ..Default::default()
        }
    }

    pub fn get_from_point(&self, point: Point) -> Option<Element> {
        if !point.is_inside(self) {
            return None;
        }
        self[(point.x as u32, point.y as u32)].element()
    }
    pub fn get_from_point_mut(&mut self, point: Point) -> Option<&mut Element> {
        if !point.is_inside(self) {
            return None;
        }
        self[(point.x as u32, point.y as u32)].element_mut()
    }

    pub fn add_portal_set(&mut self, portal: PortalSet) {
        self.portals.push(portal);
    }

    pub fn move_in_universe(&self, point: Point, movement: Point) -> (Point, Point) {
        self.portals[0]
            .cross(point, movement)
            .unwrap_or_else(|| (point + movement, movement))
    }

    pub fn section(&self, x: u32, y: u32, width: u32, height: u32) -> Universe {
        let mut universe = Universe::new(width, height);
        for yi in 0..height {
            for xi in 0..width {
                universe[(xi, yi)] = self[(x + xi, y + yi)].clone();
            }
        }
        universe
    }
}

impl Index<(u32, u32)> for Universe {
    type Output = Region;
    fn index(&self, (x, y): (u32, u32)) -> &Region {
        &self.data[(x + y * self.width) as usize]
    }
}
impl IndexMut<(u32, u32)> for Universe {
    fn index_mut(&mut self, (x, y): (u32, u32)) -> &mut Region {
        &mut self.data[(x + y * self.width) as usize]
    }
}

impl Universe {
    fn normalize(&self) -> Universe {
        let mut mass_min = f64::MAX;
        let mut mass_max = f64::MIN;
        let mut mass_field_max = f64::MIN;
        for y in 0..self.height {
            for x in 0..self.width {
                let element = self[(x, y)].element().unwrap();
                let mass = element.mass.value;
                mass_min = mass_min.min(mass);
                mass_max = mass_max.max(mass);
                let mag = element.mass.field.magnitude();
                mass_field_max = mass_field_max.max(mag);
            }
        }
        let mut new = self.clone();
        for y in 0..new.height {
            for x in 0..new.width {
                let mut element = new[(x, y)].element().unwrap();
                let mass = element.mass.value;
                element.mass.value = (mass - mass_min) / (mass_max - mass_min);
                element.mass.field /= mass_field_max;
                new[(x, y)] = Region::Element(element);
            }
        }
        new
    }
    pub fn to_image(&self) -> DynamicImage {
        let universe = self.normalize();
        const K: f64 = 255.0 / 2.0;
        //* Create image
        let mut img = ImageBuffer::new(universe.width, universe.height);
        //* Draw portals
        let minus = Point { x: -1.0, y: 0.0 };
        let plus = Point { x: 1.0, y: 0.0 };
        const PORTAL_COLOUR: Rgb<u8> = Rgb([192, 32, 32]);
        for portalset in self.portals.iter() {
            let PortalSet { a, b } = portalset;
            draw_line(&mut img, a.point_a, a.point_b, PORTAL_COLOUR);
            draw_line(
                &mut img,
                a.point_a + minus,
                a.point_b + minus,
                PORTAL_COLOUR,
            );
            draw_line(&mut img, a.point_a + plus, a.point_b + plus, PORTAL_COLOUR);
            draw_line(&mut img, b.point_a, b.point_b, PORTAL_COLOUR);
            draw_line(
                &mut img,
                b.point_a + minus,
                b.point_b + minus,
                PORTAL_COLOUR,
            );
            draw_line(&mut img, b.point_a + plus, b.point_b + plus, PORTAL_COLOUR);
        }
        //* Draw field(s)
        img.par_enumerate_pixels_mut().for_each(|(x, y, pixel)| {
            let element = universe[(x, y)].element().unwrap();
            let field = element.mass.field.direction();
            let mag = field.magnitude();
            let r = (field.magnitude() * 255.0) as u8;
            let g = ((field.x + 1.0) * K * mag) as u8;
            let b = ((field.y + 1.0) * K * mag) as u8;
            if *pixel == PORTAL_COLOUR {
                let colour = Rgb([
                    r as f64 + pixel.0[0] as f64,
                    g as f64 + pixel.0[1] as f64,
                    b as f64 + pixel.0[2] as f64,
                ]
                .map(|v| (v / 2.0) as u8));
                *pixel = colour;
            } else {
                *pixel = Rgb([r, g, b]);
            }
        });
        DynamicImage::ImageRgb8(img)
    }
}

use core::ops::{Deref, DerefMut};

/// https://en.wikipedia.org/wiki/Line_drawing_algorithm
/// https://en.wikipedia.org/wiki/Bresenham%27s_line_algorithm
/// https://rosettacode.org/wiki/Bitmap/Bresenham%27s_line_algorithm
fn draw_line<Pixel: image::Pixel, Container: Deref<Target = [Pixel::Subpixel]> + DerefMut>(
    img: &mut ImageBuffer<Pixel, Container>,
    start: Point,
    end: Point,
    color: Pixel,
) {
    // println!("Drawing line from {} to {}", start, end);
    let Point { x: x0, y: y0 } = start;
    let Point { x: x1, y: y1 } = end;
    let dx = x1 - x0;
    let dy = y1 - y0;

    if dy < dx {
        if x0 < x1 {
            draw_line_low(img, start, end, color);
        } else {
            draw_line_low(img, end, start, color);
        }
    } else {
        if y0 < y1 {
            draw_line_high(img, start, end, color);
        } else {
            draw_line_high(img, end, start, color);
        }
    }
}

#[inline]
fn draw_line_low<Pixel: image::Pixel, Container: Deref<Target = [Pixel::Subpixel]> + DerefMut>(
    img: &mut ImageBuffer<Pixel, Container>,
    start: Point,
    end: Point,
    color: Pixel,
) {
    let Point { x: x0, y: y0 } = start;
    let Point { x: x1, y: y1 } = end;
    let dx = x1 - x0;
    let dy = y1 - y0;
    let mut d = 2.0 * dy - dx;
    let mut y = y0.round() as u32;

    if dy > 0.0 {
        for x in x0.round() as u32..=x1.round() as u32 {
            img.get_pixel_mut_checked(x, y).map(|p| *p = color);
            if d > 0.0 {
                y += 1;
                d += 2.0 * (dy - dx);
            }
            d += 2.0 * dy;
        }
    } else {
        let dy = dy.abs();
        for x in x0.round() as u32..=x1.round() as u32 {
            img.get_pixel_mut_checked(x, y).map(|p| *p = color);
            if d > 0.0 {
                y -= 1;
                d += 2.0 * (dy - dx);
            }
            d += 2.0 * dy;
        }
    }
}

#[inline]
fn draw_line_high<Pixel: image::Pixel, Container: Deref<Target = [Pixel::Subpixel]> + DerefMut>(
    img: &mut ImageBuffer<Pixel, Container>,
    start: Point,
    end: Point,
    color: Pixel,
) {
    let Point { x: x0, y: y0 } = start;
    let Point { x: x1, y: y1 } = end;
    let dx = x1 - x0;
    let dy = y1 - y0;
    let mut d = 2.0 * dx - dy;
    let mut x = x0.round() as u32;

    if dx > 0.0 {
        for y in y0.round() as u32..=y1.round() as u32 {
            img.get_pixel_mut_checked(x, y).map(|p| *p = color);
            if d > 0.0 {
                x += 1;
                d += 2.0 * (dx - dy);
            }
            d += 2.0 * dx;
        }
    } else {
        let dx = dx.abs();
        for y in y0.round() as u32..=y1.round() as u32 {
            img.get_pixel_mut_checked(x, y).map(|p| *p = color);
            if d > 0.0 {
                x -= 1;
                d += 2.0 * (dx - dy);
            }
            d += 2.0 * dx;
        }
    }
}
