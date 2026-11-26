use super::{
    point::Point,
    portal::{PortalInteraction, PortalSet},
    region::{Region, Regions},
    util::{Element, Scale},
};

use std::ops::{Index, IndexMut};

// use colorgrad::Gradient;
use image::{DynamicImage, ImageBuffer, Rgb};
use rayon::prelude::*;

#[derive(Debug, Clone, Default)]
pub struct UniverseData<T: Copy> {
    pub width: u32,
    pub height: u32,
    pub regions: Regions<T>,
}

impl<T: Copy> UniverseData<T> {
    pub fn get_from_point(&self, point: Point) -> Option<(Scale, T)> {
        if !point.is_inside(self) {
            return None;
        }
        let index = (point.y as u32 * self.width + point.x as u32) as usize;
        self.regions[index].element()
    }

    pub fn get_from_point_mut(&mut self, point: Point) -> Option<(Scale, &mut T)> {
        if !point.is_inside(self) {
            return None;
        }
        let index = (point.y as u32 * self.width + point.x as u32) as usize;
        self.regions[index].element_mut()
    }

    pub unsafe fn get_from_point_unchecked(&self, point: Point) -> (Scale, T) {
        let index = (point.y as u32 * self.width + point.x as u32) as usize;
        unsafe { self.regions.get_unchecked(index) }
            .element()
            .unwrap()
    }

    pub unsafe fn get_from_point_unchecked_mut(&mut self, point: Point) -> (Scale, &mut T) {
        let index = (point.y as u32 * self.width + point.x as u32) as usize;
        unsafe { self.regions.get_unchecked_mut(index) }
            .element_mut()
            .unwrap()
    }

    pub fn fill(&mut self, value: T) {
        for region in self.regions.iter_mut() {
            match region {
                Region::Element(e) => *e = value,
                // _ => (),
            }
        }
    }

    #[allow(dead_code)]
    pub fn neighbouring_directions(&self, point: Point) -> Vec<Point> {
        if let Some((scale, _)) = self.get_from_point(point) {
            let mut positions = Vec::new();
            for direction in NEIGHBOURING_DIRECTIONS {
                let direction = direction * scale.0;
                if (point + direction).is_inside(self) {
                    positions.push(direction);
                }
            }
            positions
        } else {
            Vec::new()
        }
    }

    #[allow(dead_code)]
    pub fn neighbouring_positions(&self, point: Point) -> Vec<Point> {
        if let Some((scale, _)) = self.get_from_point(point) {
            let mut positions = Vec::new();
            for direction in NEIGHBOURING_DIRECTIONS {
                let neighbour = point + direction * scale.0;
                if neighbour.is_inside(self) {
                    positions.push(neighbour);
                }
            }
            positions
        } else {
            Vec::new()
        }
    }
}
const NEIGHBOURING_DIRECTIONS: [Point; 4] = [
    // Above
    Point { x: 0.0, y: -1.0 },
    // Sides
    Point { x: -1.0, y: 0.0 },
    Point { x: 1.0, y: 0.0 },
    // Below
    Point { x: 0.0, y: 1.0 },
];

#[derive(Debug, Clone, Default)]
pub struct Universe {
    portalsets: Vec<PortalSet>,
    data: UniverseData<Element>,
}

impl Universe {
    pub fn new(width: u32, height: u32) -> Universe {
        let regions = vec![Default::default(); (width * height) as usize];
        Universe {
            data: UniverseData {
                width,
                height,
                regions,
            },
            ..Default::default()
        }
    }

    #[inline(always)]
    pub const fn width(&self) -> u32 {
        self.data.width
    }

    #[inline(always)]
    pub const fn height(&self) -> u32 {
        self.data.height
    }

    #[allow(dead_code)]
    pub fn get_from_point(&self, point: Point) -> Option<(Scale, Element)> {
        self.data.get_from_point(point)
    }

    #[allow(dead_code)]
    pub fn get_from_point_mut(&mut self, point: Point) -> Option<(Scale, &mut Element)> {
        self.data.get_from_point_mut(point)
    }

    #[allow(dead_code)]
    pub unsafe fn get_from_point_unchecked(&self, point: Point) -> (Scale, Element) {
        unsafe { self.data.get_from_point_unchecked(point) }
    }

    #[allow(dead_code)]
    pub unsafe fn get_from_point_unchecked_mut(&mut self, point: Point) -> (Scale, &mut Element) {
        unsafe { self.data.get_from_point_unchecked_mut(point) }
    }

    #[allow(dead_code)]
    pub fn fill(&mut self, value: Element) {
        self.data.fill(value);
    }

    pub fn add_portal_set<PS: Into<PortalSet>>(&mut self, portalset: PS) {
        self.portalsets.push(portalset.into());
    }

    /// TODO: test multiple portal sets
    pub fn crosses_portal(&self, point: Point, speed: Point) -> bool {
        self.portalsets
            .iter()
            .any(|ps| ps.cross_part_1(point, speed).is_some())
    }

    /// TODO: test multiple portal sets
    pub fn in_universe_move(&self, mut point: Point, mut speed: Point) -> (Point, Point) {
        let mut yet_to_move: Point = speed;
        loop {
            // ! Non const
            let crossing = self.portalsets[0].cross_part_1(point, yet_to_move);
            if crossing.is_none() {
                return (point + speed, speed);
            }
            let cross = self.portalsets[0].cross_part_2(speed, crossing.unwrap());
            if cross.is_none() {
                return (point + speed, speed);
            }
            (point, speed, yet_to_move) = cross.unwrap();
            if yet_to_move.magnitude_2() <= 1.0 {
                point.const_add_assign(yet_to_move);
                break;
            }
        }
        (point, speed)
    }

    #[allow(dead_code)]
    pub fn section(&self, x: u32, y: u32, width: u32, height: u32) -> Universe {
        let mut universe = Universe::new(width, height);
        for yi in 0..height {
            for xi in 0..width {
                universe[(xi, yi)] = self[(x + xi, y + yi)].clone();
            }
        }
        universe
    }

    pub fn neighbouring_positions<const INTERACTION: PortalInteraction>(
        &self,
        point: Point,
    ) -> Vec<Point> {
        match INTERACTION {
            PortalInteraction::Cross => self
                .data
                .neighbouring_directions(point)
                .iter()
                .filter_map(|&neighbour| {
                    let (pos, speed) = self.in_universe_move(point, neighbour);
                    if !pos.is_inside(self) {
                        None
                    } else {
                        let scale = speed.magnitude() / neighbour.magnitude();
                        Some(pos * scale)
                    }
                })
                .collect(),
            PortalInteraction::Wall => self
                .data
                .neighbouring_directions(point)
                .iter()
                .filter_map(|&neighbour| {
                    if self.crosses_portal(point, neighbour) {
                        None
                    } else {
                        let pos = point + neighbour;
                        if !pos.is_inside(self) {
                            None
                        } else {
                            Some(pos)
                        }
                    }
                })
                .collect(),
            PortalInteraction::Ignore => self
                .data
                .neighbouring_directions(point)
                .iter()
                .filter_map(|&neighbour| {
                    let pos = point + neighbour;
                    if !pos.is_inside(self) {
                        None
                    } else {
                        Some(pos)
                    }
                })
                .collect(),
        }
    }
}

impl<T> Index<(u32, u32)> for UniverseData<T>
where
    T: Copy,
{
    type Output = Region<T>;
    fn index(&self, (x, y): (u32, u32)) -> &Region<T> {
        &self.regions[(x + y * self.width) as usize]
    }
}
impl<T> IndexMut<(u32, u32)> for UniverseData<T>
where
    T: Copy,
{
    fn index_mut(&mut self, (x, y): (u32, u32)) -> &mut Region<T> {
        &mut self.regions[(x + y * self.width) as usize]
    }
}
impl Index<(u32, u32)> for Universe {
    type Output = Region<Element>;
    fn index(&self, (x, y): (u32, u32)) -> &Region<Element> {
        let width = self.width();
        &self.data.regions[(x + y * width) as usize]
    }
}
impl IndexMut<(u32, u32)> for Universe {
    fn index_mut(&mut self, (x, y): (u32, u32)) -> &mut Region<Element> {
        let width = self.width();
        &mut self.data.regions[(x + y * width) as usize]
    }
}

impl Universe {
    fn normalize(&self) -> Universe {
        let mut mass_min = f64::MAX;
        let mut mass_max = f64::MIN;
        let mut mass_field_mag_max = f64::MIN;
        for y in 0..self.height() {
            for x in 0..self.width() {
                let (.., element) = self[(x, y)].element().unwrap();
                let mass = element.mass.value;
                mass_min = mass_min.min(mass);
                mass_max = mass_max.max(mass);
                let mag = element.mass.field.magnitude();
                mass_field_mag_max = mass_field_mag_max.max(mag);
            }
        }
        let mut new = self.clone();
        for y in 0..new.height() {
            for x in 0..new.width() {
                let (.., element) = new[(x, y)].element_mut().unwrap();
                let mass = element.mass.value;
                element.mass.value = (mass - mass_min) / (mass_max - mass_min);
                element.mass.field /= mass_field_mag_max;
            }
        }
        new
    }
    pub fn to_image(&self) -> DynamicImage {
        let universe = self.normalize();
        //* Create image
        let mut img = ImageBuffer::new(universe.width(), universe.height());
        //* Draw portals
        let minus = Point { x: -1.0, y: 0.0 };
        let plus = Point { x: 1.0, y: 0.0 };
        const PORTAL_COLOUR: Rgb<u8> = Rgb([192, 32, 32]);
        for portalset in self.portalsets.iter() {
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
        // let gradient = colorgrad::preset::viridis();
        img.par_enumerate_pixels_mut().for_each(|(x, y, pixel)| {
            let (.., element) = universe[(x, y)].element().unwrap();

            // let field = element.mass.field;
            // let mag = field.magnitude();
            // let [r, g, b, ..] = gradient.at(mag as f32).to_rgba8();

            let rgb = polar_point_to_rgb(element.mass.field);

            if *pixel == PORTAL_COLOUR {
                let [r, g, b] = rgb;
                *pixel = Rgb([
                    ((r as f64 + pixel.0[0] as f64) * 0.5) as u8,
                    ((g as f64 + pixel.0[1] as f64) * 0.5) as u8,
                    ((b as f64 + pixel.0[2] as f64) * 0.5) as u8,
                ]);
            } else {
                *pixel = Rgb(rgb);
            }
        });
        DynamicImage::ImageRgb8(img)
    }
}

fn polar_point_to_rgb(point: Point) -> [u8; 3] {
    use palette::{
        FromColor,
        // oklch::Oklch,
        hsluv::Hsluv,
        rgb::{LinSrgb, Srgb},
    };
    // let colour = Oklch::<f64>::new(point.magnitude(), 1.0, point.into_angle_percentage());
    let colour = Hsluv::new(
        point.into_angle() as f32,
        100.0,
        80.0 * point.magnitude() as f32,
    );
    let rgb = LinSrgb::<f32>::from_color(colour);
    let rgb = Srgb::<u8>::from_linear(rgb);
    rgb.into()
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
        draw_line_low(
            img,
            if x0 < x1 { (start, end) } else { (end, start) },
            color,
        );
    } else {
        draw_line_high(
            img,
            if y0 < y1 { (start, end) } else { (end, start) },
            color,
        );
    }
}

#[inline]
fn draw_line_low<Pixel: image::Pixel, Container: Deref<Target = [Pixel::Subpixel]> + DerefMut>(
    img: &mut ImageBuffer<Pixel, Container>,
    path: (Point, Point),
    color: Pixel,
) {
    let Point { x: x0, y: y0 } = path.0;
    let Point { x: x1, y: y1 } = path.1;
    let dx = x1 - x0;
    let dy = y1 - y0;
    let mut d = 2.0 * dy - dx;
    let mut y = y0.round() as u32;

    if dy > 0.0 {
        for x in x0.round() as u32..=x1.round() as u32 {
            if let Some(p) = img.get_pixel_mut_checked(x, y) {
                *p = color;
            }
            if d > 0.0 {
                y += 1;
                d += 2.0 * (dy - dx);
            }
            d += 2.0 * dy;
        }
    } else {
        let dy = dy.abs();
        for x in x0.round() as u32..=x1.round() as u32 {
            if let Some(p) = img.get_pixel_mut_checked(x, y) {
                *p = color;
            }
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
    path: (Point, Point),
    color: Pixel,
) {
    let Point { x: x0, y: y0 } = path.0;
    let Point { x: x1, y: y1 } = path.1;
    let dx = x1 - x0;
    let dy = y1 - y0;
    let mut d = 2.0 * dx - dy;
    let mut x = x0.round() as u32;

    if dx > 0.0 {
        for y in y0.round() as u32..=y1.round() as u32 {
            if let Some(p) = img.get_pixel_mut_checked(x, y) {
                *p = color;
            }
            if d > 0.0 {
                x += 1;
                d += 2.0 * (dx - dy);
            }
            d += 2.0 * dx;
        }
    } else {
        let dx = dx.abs();
        for y in y0.round() as u32..=y1.round() as u32 {
            if let Some(p) = img.get_pixel_mut_checked(x, y) {
                *p = color;
            }
            if d > 0.0 {
                x -= 1;
                d += 2.0 * (dx - dy);
            }
            d += 2.0 * dx;
        }
    }
}

impl Universe {
    pub fn field<T: Copy + Default>(universe: &Self) -> UniverseData<T> {
        let original = &universe.data;
        let mut regions = Vec::new();
        for region in &original.regions {
            match region {
                Region::Element(_) => regions.push(Region::Element(Default::default())),
                // Region::Subdivision(subdivision) => {
                //     regions.push(Region::Subdivision(subdivision.clone()))
                // }
            }
        }
        UniverseData {
            width: original.width,
            height: original.height,
            regions,
        }
    }

    #[inline]
    pub fn iter_elements<'a>(&'a self) -> UniverseDataIterator<'a, Element> {
        self.data.into_iter()
    }
}

#[derive(Debug, Clone)]
pub struct UniverseDataIterator<'a, T: Copy> {
    universe: &'a UniverseData<T>,
    position: Point,
    indexes: Vec<usize>,
}

impl<'a, T> IntoIterator for &'a UniverseData<T>
where
    T: Copy,
{
    type Item = (Point, (Scale, T));
    type IntoIter = UniverseDataIterator<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        UniverseDataIterator {
            universe: self,
            position: Default::default(),
            indexes: vec![0],
        }
    }
}

impl<'a, T> Iterator for UniverseDataIterator<'a, T>
where
    T: Copy,
{
    type Item = (Point, (Scale, T));

    fn next(&mut self) -> Option<Self::Item> {
        let point = self.position;
        // Get element
        let mut local = &self.universe.regions;
        if unsafe { self.indexes.get_unchecked(0) } == &local.len() {
            return None;
        }
        for index in self.indexes.iter() {}
        let element = self.universe[(point.x as u32, point.y as u32)]
            .element()
            .unwrap();
        // Update position
        self.position.x += 1.0;
        if self.position.x.ceil() as u32 == self.universe.width {
            self.position.x = 0.0;
            self.position.y += 1.0;
        }
        // Return
        Some((point, element))
    }
}
