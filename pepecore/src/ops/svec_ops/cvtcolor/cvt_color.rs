//! Module for color space conversions on `SVec` images.
//!
//! Provides a single function `cvt_color` that applies various color transformations
//! (e.g., RGB⇄Gray, RGB⇄YCbCr, RGB⇄CMYK, channel swaps) on an `SVec` in-place, supporting
//! different pixel types (u8, u16, f32).
//!
//! # Examples
//!
//! ```rust
//! use pepecore::cvt_color::cvt_color;
//! use pepecore_array::{SVec,Shape,PixelType,ImgData};
//! use pepecore::enums::{CVTColor};
//!
//! // Create an example SVec with 3-channel u8 data
//! let mut svec = SVec::new(Shape::new(2, 2, Some(3)), ImgData::U8(vec![255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 0]));
//! // Convert RGB to grayscale using the BT.601 standard
//! cvt_color(&mut svec, CVTColor::RGB2Gray_601);
//! assert_eq!(svec.pixel_type(), PixelType::U8);
//! assert!(svec.shape.get_channels().is_none()); // now single-channel
//! ```
use crate::enums::CVTColor;
use crate::ops::svec_ops::cvtcolor::constants::*;
use crate::ops::svec_ops::cvtcolor::cvt::{gray_to_rgb, rgb_to_bayer_2x2, rgb_to_bgr};
use crate::ops::svec_ops::cvtcolor::cvt_f32::*;
use crate::ops::svec_ops::cvtcolor::cvt_u8::*;
use crate::ops::svec_ops::cvtcolor::cvt_u16::*;
use pepecore_array::{PixelType, SVec};

/// Convert color space of `SVec` in-place according to `cvt_type`.
///
/// Dispatches based on pixel type (`PixelType::F32`, `U8`, `U16`) and applies
/// the selected conversion:
///
/// - **RGB → Gray** (BT.601, BT.709, BT.2020)
/// - **RGB → YCbCr** (BT.601, BT.709, BT.2020)
/// - **YCbCr → RGB** (BT.601, BT.709, BT.2020)
/// - **RGB ⇄ CMYK**
/// - **RGB ⇄ BGR** (channel swap)
/// - **Gray → RGB** (replicate channel)
///
/// # Parameters
///
/// - `img`: mutable reference to an `SVec` containing image data.
/// - `cvt_type`: variant of `CVTColor` specifying the desired conversion.
///
/// # Panics
///
/// Panics if the `SVec` channel count is not compatible with the chosen conversion
/// (e.g., `RGB2Gray` on non-3-channel image).
///
/// # Examples
///
/// ```rust
/// use pepecore::cvt_color::cvt_color;
/// use pepecore_array::{SVec,Shape,PixelType,ImgData};
/// use pepecore::enums::{CVTColor};
///
/// // 1x1 pixel RGB f32
/// let mut svec = SVec::new(Shape::new(1, 1, Some(3)), ImgData::F32(vec![0.5, 0.2, 0.7]));
/// cvt_color(&mut svec, CVTColor::RGB2YCbCR_709);
/// assert_eq!(svec.pixel_type(), PixelType::F32);
/// assert_eq!(svec.shape.get_ndims(), 3);
/// ```
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
            CVTColor::RGB2Bayer_BGGR => rgb_to_bayer_2x2(img, [0, 1, 1, 2]),
            CVTColor::RGB2Bayer_RGGB => rgb_to_bayer_2x2(img, [2, 1, 1, 0]),
            CVTColor::RGB2Bayer_GBRG => rgb_to_bayer_2x2(img, [1, 0, 2, 1]),
            CVTColor::RGB2Bayer_GRBG => rgb_to_bayer_2x2(img, [1, 2, 0, 1]),
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
            CVTColor::RGB2Bayer_BGGR => rgb_to_bayer_2x2(img, [0, 1, 1, 2]),
            CVTColor::RGB2Bayer_RGGB => rgb_to_bayer_2x2(img, [2, 1, 1, 0]),
            CVTColor::RGB2Bayer_GBRG => rgb_to_bayer_2x2(img, [1, 0, 2, 1]),
            CVTColor::RGB2Bayer_GRBG => rgb_to_bayer_2x2(img, [1, 2, 0, 1]),
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
            CVTColor::RGB2Bayer_BGGR => rgb_to_bayer_2x2(img, [0, 1, 1, 2]),
            CVTColor::RGB2Bayer_RGGB => rgb_to_bayer_2x2(img, [2, 1, 1, 0]),
            CVTColor::RGB2Bayer_GBRG => rgb_to_bayer_2x2(img, [1, 0, 2, 1]),
            CVTColor::RGB2Bayer_GRBG => rgb_to_bayer_2x2(img, [1, 2, 0, 1]),
        },
    }
}
