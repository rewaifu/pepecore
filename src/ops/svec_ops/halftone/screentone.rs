use crate::array::svec::SVec;
use crate::enums::PixelType;
use crate::ops::svec_ops::halftone::dot::dot_circle;
use crate::ops::svec_ops::halftone::utils::{compute_cos_sin, rotate_pixel_coordinates};

trait ScreentonePixel: Sized + Copy + PartialOrd + 'static {
    const MIN_VALUE: Self;
    const MAX_VALUE: Self;

    fn prepare_dot_matrix(matrix: &[f32]) -> Vec<Self>;
}

impl ScreentonePixel for f32 {
    const MIN_VALUE: Self = 0.0;
    const MAX_VALUE: Self = 1.0;

    fn prepare_dot_matrix(matrix: &[f32]) -> Vec<Self> {
        matrix.into()
    }
}

impl ScreentonePixel for u8 {
    const MIN_VALUE: Self = u8::MIN;
    const MAX_VALUE: Self = u8::MAX;

    fn prepare_dot_matrix(matrix: &[f32]) -> Vec<Self> {
        let mut new_dot_matrix_data = Vec::with_capacity(matrix.len());

        unsafe {
            let src_ptr = matrix.as_ptr();
            let dst_ptr: *mut u8 = new_dot_matrix_data.as_mut_ptr();

            for i in 0..matrix.len() {
                *dst_ptr.add(i) = (*src_ptr.add(i) * 255.0).min(255.0) as u8;
            }

            new_dot_matrix_data.set_len(matrix.len());
        }

        new_dot_matrix_data
    }
}

impl ScreentonePixel for u16 {
    const MIN_VALUE: Self = u16::MIN;
    const MAX_VALUE: Self = u16::MAX;

    fn prepare_dot_matrix(matrix: &[f32]) -> Vec<Self> {
        let mut new_dot_matrix_data = Vec::with_capacity(matrix.len());

        unsafe {
            let src_ptr = matrix.as_ptr();
            let dst_ptr: *mut u16 = new_dot_matrix_data.as_mut_ptr();

            for i in 0..matrix.len() {
                *dst_ptr.add(i) = (*src_ptr.add(i) * u16::MAX as f32).min(u16::MAX as f32) as u16;
            }

            new_dot_matrix_data.set_len(matrix.len());
        }

        new_dot_matrix_data
    }
}

fn apply_screentone<T: ScreentonePixel>(img: &mut SVec, dot_size: usize) {
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
fn apply_rotate_screentone<T: ScreentonePixel>(img: &mut SVec, dot_size: usize, angle: f32) {
    let (h, w, _) = img.shape();
    let mut_img = img.get_data_mut::<T>().unwrap();
    let cos_sin = compute_cos_sin(angle.to_radians());
    let dot_matrix = dot_circle(dot_size);
    let dot_matrix_data = dot_matrix.get_data::<f32>().unwrap();
    let mut new_dot_matrix_data = T::prepare_dot_matrix(dot_matrix_data);
    let lx_bias = w / 2;
    let ly_bias = h / 2;
    let dot_size = dot_size * 2;

    for ly in 0..h {
        let ly2 = ly + ly_bias;

        for lx in 0..w {
            let lx2 = lx + lx_bias;
            let mut value = &mut mut_img[ly * w + lx];
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
