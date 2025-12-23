use std::collections::HashSet;
use crate::ops::svec_ops::line::objects::Point;

fn cubic_bezier(v0: f64, v1: f64, v2: f64, v3: f64, t: f64) -> f64 {
    let u = 1.0 - t;
    u * u * u * v0 + 3.0 * u * u * t * v1 + 3.0 * u * t * t * v2 + t * t * t * v3
}

pub fn bezier(
    p0: &Point,
    p1: &Point,
    p2: &Point,
    p3: &Point,
    step: f64,
    line_hash: &mut HashSet<(usize, usize)>,
) {
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
    while t <= 1.0 {
        let fx = cubic_bezier(x0, x1, x2, x3, t);
        let fy = cubic_bezier(y0, y1, y2, y3, t);
        let fs = cubic_bezier(s0, s1, s2, s3, t);

        let radius_f = (fs / 2.0).max(0.5);
        let cx = fx.round() as isize;
        let cy = fy.round() as isize;

        let r_ceil = radius_f.ceil() as isize;
        for oy in -r_ceil..=r_ceil {
            let yf = oy as f64;
            let inside = radius_f * radius_f - yf * yf;
            if inside < 0.0 {
                continue;
            }
            let x_extent_f = inside.sqrt();
            let x_extent = x_extent_f.floor() as isize;
            let frac = x_extent_f - (x_extent as f64);
            let extra = if frac >= 0.5 {
                (((cx + cy + oy) & 1) == 0) as isize
            } else {
                0
            };

            let x_left = cx - x_extent - extra;
            let x_right = cx + x_extent;
            let yy = cy + oy;
            if yy < 0 {
                continue;
            }

            let mut xx = x_left.max(0);
            while xx <= x_right {
                if xx >= 0 {
                    line_hash.insert((xx as usize, yy as usize));
                }
                xx += 1;
            }
        }

        t += step;
    }
}