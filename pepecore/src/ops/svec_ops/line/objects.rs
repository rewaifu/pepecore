use crate::ops::svec_ops::line::bezier::bezier;
use crate::ops::svec_ops::line::bresenham::bresenham;
use std::collections::HashSet;

pub struct Point {
    pub x: usize,
    pub y: usize,
    pub size: usize,
}
pub enum Line {
    Bresenham(Point, Point),
    Bezier(Point, Point, Point, Point, f64),
}
pub trait Draw {
    fn draw(&self, pixel_sets: &mut HashSet<(usize, usize)>);
}

impl Draw for Line {
    fn draw(&self, pixel_sets: &mut HashSet<(usize, usize)>) {
        match self {
            Line::Bezier(p0, p1, p2, p3, step) => bezier(p0, p1, p2, p3, *step, pixel_sets),
            Line::Bresenham(p0, p1) => bresenham(p0, p1, pixel_sets),
        }
    }
}
