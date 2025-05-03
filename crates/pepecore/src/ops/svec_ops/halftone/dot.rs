use crate::enums::DotType;
use pepecore_array::{ImgData, SVec, Shape};

fn create_mask(dot: &SVec, dot_inv: &SVec) -> SVec {
    let (h, w, c) = dot.shape();
    let (h2, w2) = (h * 2, w * 2);
    let mut new_dot = SVec::new(Shape::new(h2, w2, c), ImgData::F32(vec![0.0; h2 * w2]));
    let new_dot_data = new_dot.get_data_mut::<f32>().unwrap();
    let dot_data = dot.get_data::<f32>().unwrap();
    let dot_inv_data = dot_inv.get_data::<f32>().unwrap();

    for y in 0..h2 {
        let (y_base, y_src) = if y < h { (0, y) } else { (h, y - h) };
        for x in 0..w2 {
            let (x_base, x_src) = if x < w { (0, x) } else { (w, x - w) };
            let src_idx = y_src * w + x_src;
            let dst_idx = y * w2 + x;
            let src_data = if (x_base ^ y_base) == 0 { dot_data } else { dot_inv_data };
            new_dot_data[dst_idx] = src_data[src_idx];
        }
    }

    new_dot
}
const X: f32 = 0.1;
const Y: f32 = 0.15;
fn cross(x: f32, y: f32, b: f32) -> bool {
    let h = x + b;
    let g = x - b;
    let hh = -x + b;
    let gg = -x - b;
    let c = y > g;
    let hu = y > gg;
    let mut jj = false;

    if hu {
        jj = hh > y;
    }
    if c && !jj {
        jj = h > y;
    }
    jj
}
fn line(x: f32, y: f32, h: f32) -> bool {
    let g = x + h;
    let gg = x - h;
    let mut jj = false;
    if gg < y {
        jj = g > y
    }
    jj
}

fn invline(x: f32, y: f32, h: f32) -> bool {
    let g = -x + h;
    let gg = -x - h;
    let mut jj = false;
    if gg < y {
        jj = g > y
    }
    jj
}

fn ellipse(x: f32, y: f32, h: f32) -> bool {
    let fi = 60.0_f32.to_radians();
    let cos_fi = fi.cos();
    let sin_fi = fi.sin();

    x * x + y * y - 2.0 * x * y * cos_fi < sin_fi * sin_fi * h * h
}
fn no_circle_coordinates(dot_size: usize, function: fn(f32, f32, f32) -> bool) -> Vec<(usize, usize, f32)> {
    let mut buffer_data = vec![0.0; dot_size * dot_size];
    let mut coordinates_and_values: Vec<(usize, usize, f32)> = vec![];
    for ii in 0..dot_size {
        for i in 0..dot_size {
            for b in 0..dot_size {
                if !function(
                    i as f32 - (dot_size / 2) as f32,
                    b as f32 - (dot_size / 2) as f32,
                    ii as f32 + 1.0,
                ) {
                    buffer_data[i + b * dot_size] += 1.0
                }
            }
        }
    }
    for i in 0..dot_size {
        for b in 0..dot_size {
            let value = buffer_data[i + b * dot_size];
            coordinates_and_values.push((i, b, value))
        }
    }
    coordinates_and_values
}
fn circle_cordinates(dot_size: usize) -> Vec<(usize, usize, f32)> {
    let point = (dot_size as f32 / 2.0 + X, dot_size as f32 / 2.0 + Y);
    let coordinates: Vec<(usize, usize, f32)> = (0..dot_size)
        .flat_map(|i| {
            (0..dot_size).map(move |j| {
                let dist = ((i as f32 - point.0).powi(2) + (j as f32 - point.1).powi(2)).sqrt();
                (i, j, dist)
            })
        })
        .collect();
    coordinates
}
pub fn dot_create(dot_size: usize, dot_type: &DotType) -> SVec {
    let mut mut_dot = vec![0.0; dot_size * dot_size];
    let mut mut_inv_dot = vec![0.0; dot_size * dot_size];
    let step = (1.0 - 0.5) / ((dot_size as f32).powi(2) - 1.0);
    let mut coordinates = match dot_type {
        DotType::CIRCLE => circle_cordinates(dot_size),
        DotType::CROSS => no_circle_coordinates(dot_size, cross),
        DotType::ELLIPSE => no_circle_coordinates(dot_size, ellipse),
        DotType::INVLINE => no_circle_coordinates(dot_size, invline),
        DotType::LINE => no_circle_coordinates(dot_size, line),
    };
    coordinates.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

    for (n, &(i, j, _)) in coordinates.iter().enumerate() {
        let value = step * n as f32;
        mut_dot[i * dot_size + j] = 0.5 + value;
        mut_inv_dot[i * dot_size + j] = 0.503 - value;
    }

    let dot = SVec::new(Shape::new(dot_size, dot_size, None), ImgData::F32(mut_dot));
    let inv_dot = SVec::new(Shape::new(dot_size, dot_size, None), ImgData::F32(mut_inv_dot));
    create_mask(&dot, &inv_dot)
}
