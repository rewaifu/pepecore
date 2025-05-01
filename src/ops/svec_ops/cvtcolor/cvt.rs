use pepecore_array::{PixelType, SVec, Shape};

pub fn rgb_to_bgr(img: &mut SVec) {
    let len = img.get_len();
    let img_c = img.shape.get_channels();
    assert_eq!(img_c, Some(3));
    match img.pixel_type() {
        PixelType::F32 => {
            let ptr: *mut f32 = img.get_mut_ptr::<f32>().unwrap();
            unsafe {
                for i in (0..len).step_by(3) {
                    let a = ptr.add(i);
                    let c = ptr.add(i + 2);
                    std::ptr::swap(a, c);
                }
            }
        }
        PixelType::U8 => {
            let ptr: *mut u8 = img.get_mut_ptr::<u8>().unwrap();
            unsafe {
                for i in (0..len).step_by(3) {
                    let a = ptr.add(i);
                    let c = ptr.add(i + 2);
                    std::ptr::swap(a, c);
                }
            }
        }
        PixelType::U16 => {
            let ptr: *mut u16 = img.get_mut_ptr::<u16>().unwrap();
            unsafe {
                for i in (0..len).step_by(3) {
                    let a = ptr.add(i);
                    let c = ptr.add(i + 2);
                    std::ptr::swap(a, c);
                }
            }
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
