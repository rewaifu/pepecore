use crate::ops::svec_ops::line::bresenham::bresenham;
use crate::ops::svec_ops::line::objects::Point;
use std::collections::HashSet;

fn cubic_bezier(v0: f64, v1: f64, v2: f64, v3: f64, t: f64) -> f64 {
    let u = 1.0 - t;
    u * u * u * v0 + 3.0 * u * u * t * v1 + 3.0 * u * t * t * v2 + t * t * t * v3
}

pub fn bezier(p0: &Point, p1: &Point, p2: &Point, p3: &Point, step: f64, line_hash: &mut HashSet<(usize, usize)>) {
    let x0 = p0.x as f64;
    let y0 = p0.y as f64;
    let s0 = p0.size as f64;
    let x1 = p1.x as f64;
    let y1 = p1.y as f64;
    let s1 = p1.size as f64;
    let x2 = p2.x as f64;
    let y2 = p2.y as f64;
    let s2 = p2.size as f64;
    let x3 = p3.x as f64;
    let y3 = p3.y as f64;
    let s3 = p3.size as f64;

    let mut t = 0.0;
    let mut prev_point: Option<Point> = None;

    while t <= 1.0 {
        let fx = cubic_bezier(x0, x1, x2, x3, t);
        let fy = cubic_bezier(y0, y1, y2, y3, t);
        let fs = cubic_bezier(s0, s1, s2, s3, t);

        let current_point = Point {
            x: fx.round().max(0.0) as usize,
            y: fy.round().max(0.0) as usize,
            size: fs.round().max(1.0) as usize,
        };

        if let Some(prev) = prev_point {
            bresenham(&prev, &current_point, line_hash);
        }

        prev_point = Some(current_point);
        t += step;
    }

    // Убедимся что дошли до конечной точки
    if let Some(prev) = prev_point {
        let final_point = Point {
            x: x3.round().max(0.0) as usize,
            y: y3.round().max(0.0) as usize,
            size: s3.round().max(1.0) as usize,
        };
        bresenham(&prev, &final_point, line_hash);
    }
}
