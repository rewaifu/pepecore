use crate::array::svec::SVec;
use crate::ops::svec_ops::halftone::dot::dot_circle;
use crate::ops::svec_ops::halftone::utils::{compute_cos_sin, rotate_pixel_coordinates};

pub fn f32_screentone(img: &mut SVec, dot_size: usize) {
    let (h, w, _) = img.shape();
    let mut_img = img.get_data_mut::<f32>().unwrap();

    let dot_matrix = dot_circle(dot_size);
    let dot_matrix_data = dot_matrix.get_data::<f32>().unwrap();
    let lx_bias = dot_size / 2;
    let ly_bias = dot_size / 2;
    let dot_size = dot_size * 2;

    for ly in 0..h {
        let ly2 = (ly + ly_bias) % dot_size;

        for lx in 0..w {
            let mut value = &mut mut_img[ly * w + lx];

            *value = if *value < dot_matrix_data[(lx + lx_bias) % dot_size + ly2 * dot_size] {
                0.0f32
            } else {
                1.0f32
            };
        }
    }
}
pub fn f32_rotate_screentone(img: &mut SVec, dot_size: usize, angle: f32) {
    let (h, w, _) = img.shape();
    let mut_img = img.get_data_mut::<f32>().unwrap();
    let cos_sin = compute_cos_sin(angle.to_radians());
    let dot_matrix = dot_circle(dot_size);
    let dot_matrix_data = dot_matrix.get_data::<f32>().unwrap();
    let lx_bias = w / 2;
    let ly_bias = h / 2;
    let dot_size = dot_size * 2;

    for ly in 0..h {
        let ly2 = ly + ly_bias;

        for lx in 0..w {
            let lx2 = lx + lx_bias;
            let mut value = &mut mut_img[ly * w + lx];
            let rot = rotate_pixel_coordinates(lx2 as f32, ly2 as f32, w as f32, h as f32, cos_sin[0], cos_sin[1]);
            *value = if *value < dot_matrix_data[rot.0 % dot_size + (rot.1 % dot_size) * dot_size] {
                0.0f32
            } else {
                1.0f32
            };
        }
    }
}
pub fn u8_rotate_screentone(img: &mut SVec, dot_size: usize, angle: f32) {
    let (h, w, _) = img.shape();
    let mut_img = img.get_data_mut::<u8>().unwrap();
    let cos_sin = compute_cos_sin(angle.to_radians());
    let dot_matrix = dot_circle(dot_size);
    let dot_matrix_data = dot_matrix.get_data::<f32>().unwrap();
    let mut new_dot_matrix_data = Vec::with_capacity(dot_matrix.get_len());
    unsafe {
        let src_ptr = dot_matrix_data.as_ptr();
        let dst_ptr: *mut u8 = new_dot_matrix_data.as_mut_ptr();

        for i in 0..dot_matrix_data.len() {
            *dst_ptr.add(i) = (*src_ptr.add(i) * 255.0).min(255.0) as u8;
        }

        new_dot_matrix_data.set_len(dot_matrix_data.len());
    }
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
                u8::MIN
            } else {
                u8::MAX
            };
        }
    }
}
pub fn u8_screentone(img: &mut SVec, dot_size: usize) {
    let (h, w, _) = img.shape();
    let mut_img = img.get_data_mut::<u8>().unwrap();

    let dot_matrix = dot_circle(dot_size);
    let dot_matrix_data = dot_matrix.get_data::<f32>().unwrap();
    let mut new_dot_matrix_data = Vec::with_capacity(dot_matrix.get_len());
    unsafe {
        let src_ptr = dot_matrix_data.as_ptr();
        let dst_ptr: *mut u8 = new_dot_matrix_data.as_mut_ptr();

        for i in 0..dot_matrix_data.len() {
            *dst_ptr.add(i) = (*src_ptr.add(i) * 255.0).min(255.0) as u8;
        }

        new_dot_matrix_data.set_len(dot_matrix_data.len());
    }
    let lx_bias = dot_size / 2;
    let ly_bias = dot_size / 2;
    let dot_size = dot_size * 2;

    for ly in 0..h {
        let ly2 = (ly + ly_bias) % dot_size;

        for lx in 0..w {
            let mut value = &mut mut_img[ly * w + lx];

            *value = if *value < new_dot_matrix_data[(lx + lx_bias) % dot_size + ly2 * dot_size] {
                u8::MIN
            } else {
                u8::MAX
            };
        }
    }
}
pub fn u16_rotate_screentone(img: &mut SVec, dot_size: usize, angle: f32) {
    let (h, w, _) = img.shape();
    let mut_img = img.get_data_mut::<u16>().unwrap();
    let cos_sin = compute_cos_sin(angle.to_radians());
    let dot_matrix = dot_circle(dot_size);
    let dot_matrix_data = dot_matrix.get_data::<f32>().unwrap();
    let mut new_dot_matrix_data = Vec::with_capacity(dot_matrix.get_len());
    unsafe {
        let src_ptr = dot_matrix_data.as_ptr();
        let dst_ptr: *mut u16 = new_dot_matrix_data.as_mut_ptr();

        for i in 0..dot_matrix_data.len() {
            *dst_ptr.add(i) = (*src_ptr.add(i) * u16::MAX as f32).min(u16::MAX as f32) as u16;
        }
        new_dot_matrix_data.set_len(dot_matrix_data.len());
    }
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
                u16::MIN
            } else {
                u16::MAX
            };
        }
    }
}
pub fn u16_screentone(img: &mut SVec, dot_size: usize) {
    let (h, w, _) = img.shape();
    let mut_img = img.get_data_mut::<u16>().unwrap();

    let dot_matrix = dot_circle(dot_size);
    let dot_matrix_data = dot_matrix.get_data::<f32>().unwrap();
    let mut new_dot_matrix_data = Vec::with_capacity(dot_matrix.get_len());
    unsafe {
        let src_ptr = dot_matrix_data.as_ptr();
        let dst_ptr: *mut u16 = new_dot_matrix_data.as_mut_ptr();

        for i in 0..dot_matrix_data.len() {
            *dst_ptr.add(i) = (*src_ptr.add(i) * u16::MAX as f32).min(u16::MAX as f32) as u16;
        }

        new_dot_matrix_data.set_len(dot_matrix_data.len());
    }
    let lx_bias = dot_size / 2;
    let ly_bias = dot_size / 2;
    let dot_size = dot_size * 2;

    for ly in 0..h {
        let ly2 = (ly + ly_bias) % dot_size;

        for lx in 0..w {
            let mut value = &mut mut_img[ly * w + lx];

            *value = if *value < new_dot_matrix_data[(lx + lx_bias) % dot_size + ly2 * dot_size] {
                u16::MIN
            } else {
                u16::MAX
            };
        }
    }
}
