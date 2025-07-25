use crate::array::{PixelType, SVec, Shape};
use fast_image_resize::images::{Image, ImageRef};
use fast_image_resize::{ResizeAlg, ResizeOptions, Resizer};
use image::EncodableLayout;

pub trait ResizeSVec {
    fn resize(&mut self, h: usize, w: usize, resize_alg: ResizeAlg, alpha: bool);
}
fn replace_vec_u8_from_bytes(vec: &mut Vec<u8>, bytes: &[u8]) {
    vec.clear();
    vec.extend_from_slice(bytes);
}
fn replace_vec_u16_from_bytes(vec: &mut Vec<u16>, bytes: &[u8]) {
    assert!(bytes.len() % 2 == 0);

    let len = bytes.len() / 2;
    let ptr = bytes.as_ptr() as *const u16;

    unsafe {
        let slice = std::slice::from_raw_parts(ptr, len);
        vec.clear();
        vec.extend_from_slice(slice);
    }
}
fn replace_vec_f32_from_bytes(vec: &mut Vec<f32>, bytes: &[u8]) {
    assert!(bytes.len() % 4 == 0);

    let len = bytes.len() / 4;
    let ptr = bytes.as_ptr() as *const f32;

    unsafe {
        let slice = std::slice::from_raw_parts(ptr, len);
        vec.clear();
        vec.extend_from_slice(slice);
    }
}
impl ResizeSVec for SVec {
    fn resize(&mut self, h: usize, w: usize, resize_alg: ResizeAlg, alpha: bool) {
        let mut resizer = Resizer::new();
        #[cfg(target_arch = "x86_64")]
        unsafe {
            resizer.set_cpu_extensions(fast_image_resize::CpuExtensions::Avx2);
        }
        match self.pixel_type() {
            PixelType::F32 => {
                let (h_s, w_s, c_s) = self.shape.get_shape();
                let pt = match c_s {
                    Some(1) | None => fast_image_resize::PixelType::F32,
                    Some(2) => fast_image_resize::PixelType::F32x2,
                    Some(3) => fast_image_resize::PixelType::F32x3,
                    Some(4) => fast_image_resize::PixelType::F32x4,
                    _ => panic!(),
                };
                let src = ImageRef::new(w_s as u32, h_s as u32, self.get_data::<f32>().unwrap().as_bytes(), pt).unwrap();
                let mut resized = Image::new(w as u32, h as u32, pt);
                resizer
                    .resize(
                        &src,
                        &mut resized,
                        &ResizeOptions::new().resize_alg(resize_alg).use_alpha(alpha),
                    )
                    .unwrap();
                let data = self.get_mut_vec::<f32>().unwrap();
                replace_vec_f32_from_bytes(data, resized.buffer());
                self.shape = Shape::new(h, w, c_s)
            }
            PixelType::U8 => {
                let (h_s, w_s, c_s) = self.shape.get_shape();
                let pt = match c_s {
                    Some(1) | None => fast_image_resize::PixelType::U8,
                    Some(2) => fast_image_resize::PixelType::U8x2,
                    Some(3) => fast_image_resize::PixelType::U8x3,
                    Some(4) => fast_image_resize::PixelType::U8x4,
                    _ => panic!(),
                };
                let src = ImageRef::new(w_s as u32, h_s as u32, self.get_data::<u8>().unwrap(), pt).unwrap();
                let mut resized = Image::new(w as u32, h as u32, pt);
                resizer
                    .resize(
                        &src,
                        &mut resized,
                        &ResizeOptions::new().resize_alg(resize_alg).use_alpha(alpha),
                    )
                    .unwrap();
                let data = self.get_mut_vec::<u8>().unwrap();
                replace_vec_u8_from_bytes(data, resized.buffer());
                self.shape = Shape::new(h, w, c_s)
            }
            PixelType::U16 => {
                let (h_s, w_s, c_s) = self.shape.get_shape();
                let pt = match c_s {
                    Some(1) | None => fast_image_resize::PixelType::U16,
                    Some(2) => fast_image_resize::PixelType::U16x2,
                    Some(3) => fast_image_resize::PixelType::U16x3,
                    Some(4) => fast_image_resize::PixelType::U16x4,
                    _ => panic!(),
                };
                let src = ImageRef::new(w_s as u32, h_s as u32, self.get_data::<u16>().unwrap().as_bytes(), pt).unwrap();
                let mut resized = Image::new(w as u32, h as u32, pt);
                resizer
                    .resize(
                        &src,
                        &mut resized,
                        &ResizeOptions::new().resize_alg(resize_alg).use_alpha(alpha),
                    )
                    .unwrap();
                let data = self.get_mut_vec::<u16>().unwrap();
                replace_vec_u16_from_bytes(data, resized.buffer());
                self.shape = Shape::new(h, w, c_s)
            }
        }
    }
}
