use std::collections::HashSet;
use crate::ops::svec_ops::line::objects::Point;

pub fn bresenham(p0: &Point, p1: &Point, line_hash: &mut HashSet<(usize, usize)>) {
    let (x0, y0, s0) = (p0.x as isize, p0.y as isize, p0.size as f64);
    let (x1, y1, s1) = (p1.x as isize, p1.y as isize, p1.size as f64);
    if (s0 - s1).abs() < 1e-9 {
        bresenham_fixed_circle(x0, y0, x1, y1, s0 as f64, line_hash);
        return;
    }

    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx - dy;

    let total_steps = std::cmp::max(dx, dy) as isize + 1;
    let denom = if total_steps > 1 { (total_steps - 1) as f64 } else { 1.0 };

    let mut x = x0;
    let mut y = y0;
    let mut step: isize = 0;

    while x != x1 || y != y1 {
        let t = (denom - step as f64) / denom;
        let size_f = s0 * t + s1 * (1.0 - t);
        let radius_f = (size_f / 2.0).max(0.5);
        let r_ceil = radius_f.ceil() as isize;
        for oy in -r_ceil..=r_ceil {
            let yf = oy as f64;
            let inside = radius_f * radius_f - yf * yf;
            if inside < 0.0 {
                continue;
            }
            let x_extent = (inside.sqrt()).floor() as isize;
            let frac = (inside.sqrt()) - (x_extent as f64);

            let extra = if frac >= 0.5 {
                if ((x + y) & 1) == 0 { 1 } else { 0 }
            } else { 0 };

            let x_left = x - x_extent - extra as isize;
            let x_right = x + x_extent + (if extra==1 {0} else {0}); // symmetric except extra on left

            let yy = y + oy;
            if yy < 0 { continue; }
            let xs = if x_left < 0 { 0 } else { x_left } ;
            for xx in xs..=x_right {
                if xx < 0 { continue; }
                line_hash.insert((xx as usize, yy as usize));
            }
        }
        let e2 = err * 2;
        if e2 > -dy {
            err -= dy;
            x += sx;
        }
        if e2 < dx {
            err += dx;
            y += sy;
        }
        step += 1;
    }
}

fn bresenham_fixed_circle(x0: isize, y0: isize, x1: isize, y1: isize, size_f: f64, line_hash: &mut HashSet<(usize, usize)>) {
    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx - dy;

    let radius_f = (size_f / 2.0).max(0.5);
    let r_ceil = radius_f.ceil() as isize;

    let mut x = x0;
    let mut y = y0;
    while x != x1 || y != y1 {
        for oy in -r_ceil..=r_ceil {
            let yf = oy as f64;
            let inside = radius_f * radius_f - yf * yf;
            if inside < 0.0 { continue; }
            let x_extent = (inside.sqrt()).floor() as isize;
            let frac = (inside.sqrt()) - (x_extent as f64);
            let extra = if frac >= 0.5 {
                if ((x + y) & 1) == 0 { 1 } else { 0 }
            } else { 0 };
            let x_left = x - x_extent - extra as isize;
            let x_right = x + x_extent;
            let yy = y + oy;
            if yy < 0 { continue; }
            let xs = if x_left < 0 { 0 } else { x_left } ;
            for xx in xs..=x_right {
                if xx < 0 { continue; }
                line_hash.insert((xx as usize, yy as usize));
            }
        }

        let e2 = err * 2;
        if e2 > -dy {
            err -= dy;
            x += sx;
        }
        if e2 < dx {
            err += dx;
            y += sy;
        }
    }

}