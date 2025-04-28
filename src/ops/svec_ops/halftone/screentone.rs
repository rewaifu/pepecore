use crate::array::svec::SVec;
use crate::enums::PixelType;
use crate::ops::svec_ops::halftone::dot::dot_circle;
use crate::ops::svec_ops::halftone::utils::{HalftonePixel, compute_cos_sin, rotate_pixel_coordinates};

fn apply_screentone<T: HalftonePixel>(img: &mut SVec, dot_size: usize) {
    let (h, w, _) = img.shape();
    let mut_img = img.get_data_mut::<T>().unwrap();

    let dot_matrix = dot_circle(dot_size);
    let dot_matrix_data = dot_matrix.get_data::<f32>().unwrap();
    let lx_bias = dot_size / 2;
    let ly_bias = dot_size / 2;
    let dot_size = dot_size * 2;

    let dot_matrix_converted = T::prepare_dot_matrix(dot_matrix_data);

    for ly in 0..h {
        let ly2 = (ly + ly_bias) % dot_size;

        for lx in 0..w {
            let idx = ly * w + lx;
            let dot_idx = (lx + lx_bias) % dot_size + ly2 * dot_size;

            mut_img[idx] = if mut_img[idx] < dot_matrix_converted[dot_idx] {
                T::MIN_VALUE
            } else {
                T::MAX_VALUE
            };
        }
    }
}
fn apply_rotate_screentone<T: HalftonePixel>(img: &mut SVec, dot_size: usize, angle: f32) {
    let (h, w, _) = img.shape();
    let mut_img = img.get_data_mut::<T>().unwrap();
    let cos_sin = compute_cos_sin(angle.to_radians());
    let dot_matrix = dot_circle(dot_size);
    let dot_matrix_data = dot_matrix.get_data::<f32>().unwrap();
    let new_dot_matrix_data = T::prepare_dot_matrix(dot_matrix_data);
    let lx_bias = w / 2;
    let ly_bias = h / 2;
    let dot_size = dot_size * 2;

    for ly in 0..h {
        let ly2 = ly + ly_bias;

        for lx in 0..w {
            let lx2 = lx + lx_bias;
            let value = &mut mut_img[ly * w + lx];
            let rot = rotate_pixel_coordinates(lx2 as f32, ly2 as f32, w as f32, h as f32, cos_sin[0], cos_sin[1]);
            *value = if *value < new_dot_matrix_data[rot.0 % dot_size + (rot.1 % dot_size) * dot_size] {
                T::MIN_VALUE
            } else {
                T::MAX_VALUE
            };
        }
    }
}
pub fn screentone(img: &mut SVec, dot_size: usize) {
    match img.pixel_type() {
        PixelType::F32 => apply_screentone::<f32>(img, dot_size),
        PixelType::U8 => apply_screentone::<u8>(img, dot_size),
        PixelType::U16 => apply_screentone::<u16>(img, dot_size),
    }
}

pub fn rotate_screentone(img: &mut SVec, dot_size: usize, angle: f32) {
    match img.pixel_type() {
        PixelType::F32 => apply_rotate_screentone::<f32>(img, dot_size, angle),
        PixelType::U8 => apply_rotate_screentone::<u8>(img, dot_size, angle),
        PixelType::U16 => apply_rotate_screentone::<u16>(img, dot_size, angle),
    }
}
