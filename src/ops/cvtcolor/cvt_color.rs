use crate::array::svec::{SVec, Shape};
use crate::enums::{CVTColor, PixelType};
use crate::ops::cvtcolor::constants::*;
use crate::ops::cvtcolor::cvt_f32::*;
use crate::ops::cvtcolor::cvt_u8::*;
use crate::ops::cvtcolor::cvt_u16::*;
fn rgb_to_bgr(img: &mut SVec) {
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
fn gray_to_rgb(img: &mut SVec) {
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
pub fn cvt_color(img: &mut SVec, cvt_type: CVTColor) {
    match img.pixel_type() {
        PixelType::F32 => match cvt_type {
            CVTColor::RGB2Gray_601 => rgb_to_gray_f32(img, KR_601, KG_601, KB_601),
            CVTColor::RGB2Gray_709 => rgb_to_gray_f32(img, KR_709, KG_709, KB_709),
            CVTColor::RGB2Gray_2020 => rgb_to_gray_f32(img, KR_2020, KG_2020, KB_2020),
            CVTColor::RGB2YCbCR_601 => rgb_to_ycbcr_f32(img, KR_601, KG_601, KB_601),
            CVTColor::RGB2YCbCR_709 => rgb_to_ycbcr_f32(img, KR_709, KG_709, KB_709),
            CVTColor::RGB2YCbCR_2020 => rgb_to_ycbcr_f32(img, KR_2020, KG_2020, KB_2020),
            CVTColor::YCbCR2RGB_601 => ycbcr_to_rgb_f32(img, KR_601, KG_601, KB_601),
            CVTColor::YCbCR2RGB_709 => ycbcr_to_rgb_f32(img, KR_709, KG_709, KB_709),
            CVTColor::YCbCR2RGB_2020 => ycbcr_to_rgb_f32(img, KR_2020, KG_2020, KB_2020),
            CVTColor::RGB2CMYK => rgb_to_cmyk_f32(img),
            CVTColor::CMYK2RGB => cmyk_to_rgb_f32(img),
            CVTColor::BGR2RGB => rgb_to_bgr(img),
            CVTColor::RGB2BGR => rgb_to_bgr(img),
            CVTColor::Gray2RGB => gray_to_rgb(img),
        },
        PixelType::U8 => match cvt_type {
            CVTColor::RGB2Gray_601 => rgb_to_gray_u8(img, KR_601, KG_601, KB_601),
            CVTColor::RGB2Gray_709 => rgb_to_gray_u8(img, KR_709, KG_709, KB_709),
            CVTColor::RGB2Gray_2020 => rgb_to_gray_u8(img, KR_2020, KG_2020, KB_2020),
            CVTColor::RGB2YCbCR_601 => rgb_to_ycbcr_u8(img, KR_601, KG_601, KB_601),
            CVTColor::RGB2YCbCR_709 => rgb_to_ycbcr_u8(img, KR_709, KG_709, KB_709),
            CVTColor::RGB2YCbCR_2020 => rgb_to_ycbcr_u8(img, KR_2020, KG_2020, KB_2020),
            CVTColor::YCbCR2RGB_601 => ycbcr_to_rgb_u8(img, KR_601, KG_601, KB_601),
            CVTColor::YCbCR2RGB_709 => ycbcr_to_rgb_u8(img, KR_709, KG_709, KB_709),
            CVTColor::YCbCR2RGB_2020 => ycbcr_to_rgb_u8(img, KR_2020, KG_2020, KB_2020),
            CVTColor::RGB2CMYK => rgb_to_cmyk_u8(img),
            CVTColor::CMYK2RGB => cmyk_to_rgb_u8(img),
            CVTColor::BGR2RGB => rgb_to_bgr(img),
            CVTColor::RGB2BGR => rgb_to_bgr(img),
            CVTColor::Gray2RGB => gray_to_rgb(img),
        },
        PixelType::U16 => match cvt_type {
            CVTColor::RGB2Gray_601 => rgb_to_gray_u16(img, KR_601, KG_601, KB_601),
            CVTColor::RGB2Gray_709 => rgb_to_gray_u16(img, KR_709, KG_709, KB_709),
            CVTColor::RGB2Gray_2020 => rgb_to_gray_u16(img, KR_2020, KG_2020, KB_2020),
            CVTColor::RGB2YCbCR_601 => rgb_to_ycbcr_u16(img, KR_601, KG_601, KB_601),
            CVTColor::RGB2YCbCR_709 => rgb_to_ycbcr_u16(img, KR_709, KG_709, KB_709),
            CVTColor::RGB2YCbCR_2020 => rgb_to_ycbcr_u16(img, KR_2020, KG_2020, KB_2020),
            CVTColor::YCbCR2RGB_601 => ycbcr_to_rgb_u16(img, KR_601, KG_601, KB_601),
            CVTColor::YCbCR2RGB_709 => ycbcr_to_rgb_u16(img, KR_709, KG_709, KB_709),
            CVTColor::YCbCR2RGB_2020 => ycbcr_to_rgb_u16(img, KR_2020, KG_2020, KB_2020),
            CVTColor::RGB2CMYK => rgb_to_cmyk_u16(img),
            CVTColor::CMYK2RGB => cmyk_to_rgb_u16(img),
            CVTColor::BGR2RGB => rgb_to_bgr(img),
            CVTColor::RGB2BGR => rgb_to_bgr(img),
            CVTColor::Gray2RGB => gray_to_rgb(img),
        },
    }
}
