use pepecore_array::{PixelType, SVec, Shape};

fn rgb_swap<T: Copy>(ptr: *mut T, len: usize) {
    unsafe {
        for i in (0..len).step_by(3) {
            let a = ptr.add(i);
            let c = ptr.add(i + 2);
            std::ptr::swap(a, c);
        }
    }
}

pub fn rgb_to_bgr(img: &mut SVec) {
    let len = img.get_len();
    let img_c = img.shape.get_channels();
    assert_eq!(img_c, Some(3));
    match img.pixel_type() {
        PixelType::F32 => {
            let ptr: *mut f32 = img.get_mut_ptr::<f32>().unwrap();
            rgb_swap(ptr, len)
        }
        PixelType::U8 => {
            let ptr: *mut u8 = img.get_mut_ptr::<u8>().unwrap();
            rgb_swap(ptr, len)
        }
        PixelType::U16 => {
            let ptr: *mut u16 = img.get_mut_ptr::<u16>().unwrap();
            rgb_swap(ptr, len)
        }
    }
}

pub fn gray_to_rgb(img: &mut SVec) {
    let (h, w, c) = img.shape();
    assert_eq!(c, None);
    img.shape = Shape::new(h, w, Some(3));
    match img.pixel_type() {
        PixelType::U8 => {
            let vec_img = img.get_mut_vec::<u8>().unwrap();
            for index in 0..h * w {
                vec_img.extend([vec_img[index], vec_img[index], vec_img[index]]);
            }
        }
        PixelType::F32 => {
            let vec_img = img.get_mut_vec::<f32>().unwrap();
            for index in 0..h * w {
                vec_img.extend([vec_img[index], vec_img[index], vec_img[index]]);
            }
        }
        PixelType::U16 => {
            let vec_img = img.get_mut_vec::<u16>().unwrap();
            for index in 0..h * w {
                vec_img.extend([vec_img[index], vec_img[index], vec_img[index]]);
            }
        }
    }
    img.drain(0..h * w).unwrap()
}

fn process_bayer_line<T: Copy>(ptr: *mut T, w: usize, h: usize, pattern: [usize; 4]) {
    unsafe {
        for y in 0..h {
            for x in 0..w {
                let idx = match (y % 2, x % 2) {
                    (0, 0) => pattern[0],
                    (0, 1) => pattern[1],
                    (1, 0) => pattern[2],
                    (1, 1) => pattern[3],
                    _ => unreachable!(),
                };

                let offset = y * w + x;
                let rgb_offset = offset * 3 + idx;
                *ptr.add(offset) = *ptr.add(rgb_offset);
            }
        }
    }
}

pub fn rgb_to_bayer_2x2(img: &mut SVec, pattern: [usize; 4]) {
    let (h, w, c) = img.shape();
    assert_eq!(c, Some(3));
    img.shape = Shape::new(h, w, None);

    match img.pixel_type() {
        PixelType::U8 => {
            let ptr: *mut u8 = img.get_mut_ptr::<u8>().unwrap();
            process_bayer_line(ptr, w, h, pattern);
        }
        PixelType::F32 => {
            let ptr: *mut f32 = img.get_mut_ptr::<f32>().unwrap();
            process_bayer_line(ptr, w, h, pattern);
        }
        PixelType::U16 => {
            let ptr: *mut u16 = img.get_mut_ptr::<u16>().unwrap();
            process_bayer_line(ptr, w, h, pattern);
        }
    }

    img.truncate(h * w).unwrap()
}
