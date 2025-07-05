//! Module providing functionality to save `SVec` images to disk using the `image` crate.
//!
//! Supports saving single-channel (gray), two-channel (gray+alpha), three-channel (RGB), and four-channel (RGBA)
//! images of various bit depths (u8, u16, f32), with proper error handling for unsupported formats.
//!
//! # Examples
//!
//! ```rust
//! use pepecore::save::svec_save;
//! use pepecore_array::{SVec,Shape,PixelType,ImgData};
//! use pepecore::enums::{ImgColor};
//! use std::fs;
//!
//! // Assume `svec` is obtained from decoding an image or PSD
//! let svec: SVec = SVec::new(Shape::new(3,3,None),ImgData::U8(vec![128,0,100,
//!                                                                  32,64,88,
//!                                                                  77,255,9]));
//!
//! // Save as PNG with automatic channel mapping
//! svec_save(svec, "output.png").expect("Failed to save image");
//! ```

use crate::errors::SaveError;
use crate::errors::SaveError::{GraySaveError, JxlSaveError, RGBSaveError, UnsupportedChannelSaveError};
use image::{ImageBuffer, Luma, LumaA, Rgb, Rgba};
use pepecore_array::{ImgData, SVec};
use std::path::Path;
use zune_core::bit_depth::BitDepth;
use zune_core::colorspace::ColorSpace;
use zune_core::options::EncoderOptions;
use zune_jpegxl::JxlSimpleEncoder;

/// Save an `SVec` image to the filesystem at the given `path`.
///
/// Automatically selects the appropriate pixel buffer based on the SVec's channel count and data type:
///
/// - **1 channel** (grayscale) or no channel: saves as Luma
/// - **2 channels** (grayscale + alpha): saves as LumaA
/// - **3 channels** (RGB): saves as Rgb
/// - **4 channels** (RGBA): saves as Rgba
///
/// # Parameters
///
/// - `img`: The `SVec` containing image data.
/// - `path`: File path to save the image (extension determines format).
///
/// # Errors
///
/// Returns:
/// - `GraySaveError` for errors saving grayscale or gray+alpha images.
/// - `RGBSaveError` for errors saving RGB or RGBA images.
/// - `UnsupportedChannelSaveError` if the channel count is not 0–4.
///
/// # Examples
///
/// ```rust
/// use pepecore::save::svec_save;
/// use pepecore_array::{SVec,Shape,PixelType,ImgData};
/// // Create or decode an SVec with 3 channels, u8 data
/// let svec: SVec = SVec::new(Shape::new(1,1,Some(3)),ImgData::U8(vec![0,128,255]));
/// svec_save(svec, "photo_out.jpg").unwrap();
/// ```
pub fn svec_save<P: AsRef<Path> + ?Sized>(img: SVec, path: &P) -> Result<(), SaveError> {
    let ext = path
        .as_ref()
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if ext == "jxl" {
        return save_jxl(img, path);
    }

    let (height, width, channel) = img.shape();
    match channel {
        Some(1) | None => match img.data {
            ImgData::F32(data) => {
                let img: ImageBuffer<Luma<u8>, Vec<u8>> = ImageBuffer::from_fn(width as u32, height as u32, |x, y| {
                    let idx = (y * width as u32 + x) as usize;
                    let value = (data[idx] * 255.0) as u8;
                    Luma([value])
                });
                img.save(path).map_err(|e| GraySaveError(format!("{:?}", e)))?
            }
            ImgData::U16(data) => {
                let img: ImageBuffer<Luma<u16>, Vec<u16>> = ImageBuffer::from_raw(width as u32, height as u32, data).unwrap();
                img.save(path).map_err(|e| GraySaveError(format!("{:?}", e)))?
            }
            ImgData::U8(data) => {
                let img: ImageBuffer<Luma<u8>, Vec<u8>> = ImageBuffer::from_raw(width as u32, height as u32, data).unwrap();
                img.save(path).map_err(|e| GraySaveError(format!("{:?}", e)))?
            }
        },
        Some(2) => match img.data {
            ImgData::F32(data) => {
                let img: ImageBuffer<LumaA<u8>, Vec<u8>> = ImageBuffer::from_fn(width as u32, height as u32, |x, y| {
                    let idx = ((y * width as u32 + x) * 2) as usize;
                    let value = (data[idx] * 255.0) as u8;
                    let alpha = (data[idx + 1] * 255.0) as u8;
                    LumaA([value, alpha])
                });
                img.save(path).map_err(|e| GraySaveError(format!("{:?}", e)))?
            }
            ImgData::U16(data) => {
                let img: ImageBuffer<LumaA<u16>, Vec<u16>> = ImageBuffer::from_raw(width as u32, height as u32, data).unwrap();
                img.save(path).map_err(|e| GraySaveError(format!("{:?}", e)))?
            }
            ImgData::U8(data) => {
                let img: ImageBuffer<LumaA<u8>, Vec<u8>> = ImageBuffer::from_raw(width as u32, height as u32, data).unwrap();
                img.save(path).map_err(|e| GraySaveError(format!("{:?}", e)))?
            }
        },
        Some(3) => match img.data {
            ImgData::F32(data) => {
                let img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_fn(width as u32, height as u32, |x, y| {
                    let idx = ((y * width as u32 + x) * 3) as usize;
                    let r = (data[idx] * 255.0) as u8;
                    let g = (data[idx + 1] * 255.0) as u8;
                    let b = (data[idx + 2] * 255.0) as u8;
                    Rgb([r, g, b])
                });
                img.save(path).map_err(|e| RGBSaveError(format!("{:?}", e)))?
            }
            ImgData::U16(data) => {
                let img: ImageBuffer<Rgb<u16>, Vec<u16>> = ImageBuffer::from_raw(width as u32, height as u32, data).unwrap();
                img.save(path).map_err(|e| RGBSaveError(format!("{:?}", e)))?
            }
            ImgData::U8(data) => {
                let img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_raw(width as u32, height as u32, data).unwrap();
                img.save(path).map_err(|e| RGBSaveError(format!("{:?}", e)))?
            }
        },
        Some(4) => match img.data {
            ImgData::F32(data) => {
                let img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_fn(width as u32, height as u32, |x, y| {
                    let idx = ((y * width as u32 + x) * 4) as usize;
                    let r = (data[idx] * 255.0) as u8;
                    let g = (data[idx + 1] * 255.0) as u8;
                    let b = (data[idx + 2] * 255.0) as u8;
                    let a = (data[idx + 3] * 255.0) as u8;
                    Rgba([r, g, b, a])
                });
                img.save(path).map_err(|e| RGBSaveError(format!("{:?}", e)))?
            }
            ImgData::U16(data) => {
                let img: ImageBuffer<Rgba<u16>, Vec<u16>> = ImageBuffer::from_raw(width as u32, height as u32, data).unwrap();
                img.save(path).map_err(|e| RGBSaveError(format!("{:?}", e)))?
            }
            ImgData::U8(data) => {
                let img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_raw(width as u32, height as u32, data).unwrap();
                img.save(path).map_err(|e| RGBSaveError(format!("{:?}", e)))?
            }
        },

        _ => return Err(UnsupportedChannelSaveError(format!("{:?}", channel))),
    };
    Ok(())
}

fn save_jxl<P: AsRef<Path> + ?Sized>(img: SVec, path: &P) -> Result<(), SaveError> {
    let (height, width, channel) = img.shape();
    let colorspace = match channel {
        Some(1) | None => ColorSpace::Luma,
        Some(2) => ColorSpace::LumaA,
        Some(3) => ColorSpace::RGB,
        Some(4) => ColorSpace::RGBA,
        _ => return Err(UnsupportedChannelSaveError(format!("{:?}", channel))),
    };
    // fixme: open file stream and write to stream
    match img.data {
        ImgData::U8(data) => {
            let options = EncoderOptions::new(width, height, colorspace, BitDepth::Eight);
            let encoder = JxlSimpleEncoder::new(&data, options);
            let mut out = Vec::new();
            encoder.encode(&mut out).map_err(|e| JxlSaveError(format!("{:?}", e)))?;
            std::fs::write(path, out).map_err(|e| JxlSaveError(format!("{:?}", e)))?;
            Ok(())
        }
        ImgData::U16(data) => {
            let mut bytes = Vec::with_capacity(data.len() * 2);
            for v in data {
                bytes.extend_from_slice(&v.to_ne_bytes());
            }
            let options = EncoderOptions::new(width, height, colorspace, BitDepth::Sixteen);
            let encoder = JxlSimpleEncoder::new(&bytes, options);
            let mut out = Vec::new();
            encoder.encode(&mut out).map_err(|e| JxlSaveError(format!("{:?}", e)))?;
            std::fs::write(path, out).map_err(|e| JxlSaveError(format!("{:?}", e)))?;
            Ok(())
        }
        ImgData::F32(data) => {
            let mut bytes = Vec::with_capacity(data.len());
            for v in data {
                let val = (v.clamp(0.0, 1.0) * 255.0) as u8; // fixme: slow
                bytes.push(val);
            }
            let options = EncoderOptions::new(width, height, colorspace, BitDepth::Eight);
            let encoder = JxlSimpleEncoder::new(&bytes, options);
            let mut out = Vec::new();
            encoder.encode(&mut out).map_err(|e| JxlSaveError(format!("{:?}", e)))?;
            std::fs::write(path, out).map_err(|e| JxlSaveError(format!("{:?}", e)))?;
            Ok(())
        }
    }
}
