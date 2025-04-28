use crate::array::svec::{SVec, Shape};
use crate::enums::ImgData;

const X: f32 = 0.1;
const Y: f32 = 0.15;
fn get_step_and_point(dot_size: usize) -> (f32, (f32, f32)) {
    let point = (dot_size as f32 / 2.0 + X, dot_size as f32 / 2.0 + Y);
    let step = (1.0 - 0.5) / ((dot_size as f32).powi(2) - 1.0);
    (step, point)
}

pub fn dot_circle(dot_size: usize) -> SVec {
    let mut dot = SVec::new(
        Shape::new(dot_size, dot_size, None),
        ImgData::F32(vec![0.0; dot_size * dot_size]),
    );
    let mut inv_dot = SVec::new(
        Shape::new(dot_size, dot_size, None),
        ImgData::F32(vec![0.0; dot_size * dot_size]),
    );
    let mut_dot = dot.get_data_mut::<f32>().unwrap();
    let mut_inv_dot = inv_dot.get_data_mut::<f32>().unwrap();
    let (step, center) = get_step_and_point(dot_size);

    let mut coordinates: Vec<(usize, usize, f32)> = (0..dot_size)
        .flat_map(|i| {
            (0..dot_size).map(move |j| {
                let dist = ((i as f32 - center.0).powi(2) + (j as f32 - center.1).powi(2)).sqrt();
                (i, j, dist)
            })
        })
        .collect();

    coordinates.sort_unstable_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

    for (n, &(i, j, _)) in coordinates.iter().enumerate() {
        let value = step * n as f32;
        mut_dot[i * dot_size + j] = 0.5 + value;
        mut_inv_dot[i * dot_size + j] = 0.503 - value;
    }
    create_mask(&dot, &inv_dot)
}
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
