use std::path::Path;
use crate::array::svec::SVec;
use crate::enums::ImgData;
use crate::errors::SaveError;
use crate::errors::SaveError::{GraySaveError, RGBSaveError, UnsupportedChannelSaveError};
use image::{ImageBuffer, Luma, LumaA, Rgb, Rgba};

pub fn svec_save<P: AsRef<Path> + ?Sized>(img: SVec, path: &P) -> Result<(), SaveError> {
    let (height, width, channel) = img.shape();
    
    Ok(match channel {
        Some(1) | None => match img.data {
            ImgData::F32(data) => {
                let img: ImageBuffer<Luma<u8>, Vec<u8>> = ImageBuffer::from_fn(width as u32, height as u32, |x, y| {
                    let idx = (y * width as u32 + x) as usize;
                    let value = (data[idx] * 255.0).clamp(0.0, 255.0) as u8;
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
                    let value = (data[idx] * 255.0).clamp(0.0, 255.0) as u8;
                    let alpha = (data[idx + 1] * 255.0).clamp(0.0, 255.0) as u8;
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
                    let r = (data[idx] * 255.0).clamp(0.0, 255.0) as u8;
                    let g = (data[idx + 1] * 255.0).clamp(0.0, 255.0) as u8;
                    let b = (data[idx + 2] * 255.0).clamp(0.0, 255.0) as u8;
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
                    let r = (data[idx] * 255.0).clamp(0.0, 255.0) as u8;
                    let g = (data[idx + 1] * 255.0).clamp(0.0, 255.0) as u8;
                    let b = (data[idx + 2] * 255.0).clamp(0.0, 255.0) as u8;
                    let a = (data[idx + 3] * 255.0).clamp(0.0, 255.0) as u8;
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
    })
}
