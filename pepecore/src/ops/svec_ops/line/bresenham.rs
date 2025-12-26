use crate::ops::svec_ops::line::objects::Point;
use std::collections::HashSet;

pub fn bresenham(p0: &Point, p1: &Point, line_hash: &mut HashSet<(usize, usize)>) {
    let (x0, y0, s0) = (p0.x as isize, p0.y as isize, p0.size as f64);
    let (x1, y1, s1) = (p1.x as isize, p1.y as isize, p1.size as f64);

    if (s0 - s1).abs() < 1e-9 {
        bresenham_fixed_circle(x0, y0, x1, y1, s0, line_hash);
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

    loop {
        let t = (step as f64) / denom;
        let size_f = s0 * (1.0 - t) + s1 * t;
        draw_circle(x, y, size_f, line_hash);

        if x == x1 && y == y1 {
            break;
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

    let mut x = x0;
    let mut y = y0;

    loop {
        draw_circle(x, y, size_f, line_hash);

        if x == x1 && y == y1 {
            break;
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

fn draw_circle(cx: isize, cy: isize, size_f: f64, line_hash: &mut HashSet<(usize, usize)>) {
    let radius_f = (size_f / 2.0).max(0.5);
    let r_ceil = radius_f.ceil() as isize;

    for oy in -r_ceil..=r_ceil {
        let yf = oy as f64;
        let inside = radius_f * radius_f - yf * yf;
        if inside < 0.0 {
            continue;
        }

        let x_extent_f = inside.sqrt();
        let x_extent = x_extent_f.round() as isize;

        let x_left = cx - x_extent;
        let x_right = cx + x_extent;
        let yy = cy + oy;

        // Проверяем границы перед конвертацией в usize
        if yy < 0 {
            continue;
        }

        for xx in x_left..=x_right {
            if xx >= 0 {
                line_hash.insert((xx as usize, yy as usize));
            }
        }
    }
}
