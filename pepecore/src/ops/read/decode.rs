//! Module for decoding image buffers (PSD and common formats) into SVec structures.
//!
//! This module provides functions to decode raw bytes of PSD files as well as other image formats
//! into `SVec`, using different channel configurations (gray, rgb, rgba, gray+a) and dynamic
//! data types (U8, U16, F32).
use std::io::Cursor;

use crate::errors::DecodeError;
use crate::errors::DecodeError::{ImgDecodingError, PsdDecodingError};
use image::DynamicImage;
use pepecore_array::{ImgData, SVec, Shape};
use zune_core::bytestream::ZCursor;
use zune_psd::PSDDecoder;
/// Decode raw size information from PSD header bytes.
///
/// Reads a slice of 8 bytes (offset within PSD header) and returns
/// the (height, width) as u32 values.
///
/// # Parameters
///
/// - `bytes`: 8-byte slice from PSD header containing size info.
///
/// # Returns
///
/// A tuple `(height, width)` of the PSD canvas.
fn decode_size_psd(bytes: &[u8]) -> (u32, u32) {
    let mut height: u32 = 0;
    let mut width: u32 = 0;
    height += bytes[3] as u32;
    height += if bytes[2] > 0 { bytes[2] as u32 * 256 } else { 0 };
    height += if bytes[1] > 0 { bytes[1] as u32 * 256 * 256 } else { 0 };
    width += bytes[7] as u32;
    width += if bytes[6] > 0 { bytes[6] as u32 * 256 } else { 0 };
    width += if bytes[5] > 0 { bytes[5] as u32 * 256 * 256 } else { 0 };
    (height, width)
}
/// Decode PSD buffer into a dynamic integer SVec (packed channels).
///
/// Produces `ImgData::U8` or `ImgData::U16` based on bit depth, preserving original
/// channel layout.
///
/// # Parameters
///
/// - `buffer`: PSD file content as byte slice.
///
/// # Errors
///
/// Returns `PsdDecodingError` if PSD decoding fails or channel count is unexpected.
pub fn psd_din_decode(buffer: &[u8]) -> Result<SVec, DecodeError> {
    let size_bites: &[u8] = &buffer[14..22];
    let channels = if buffer[13] > 1 { Some(buffer[13] as usize) } else { None };

    let mut decoder = PSDDecoder::new(ZCursor::new(buffer));
    let px = decoder.decode_raw().map_err(|e| PsdDecodingError(format!("{:?}", e)))?;

    let (height, width) = decode_size_psd(size_bites);
    Ok(if buffer[23] == 16 {
        SVec::new(
            Shape::new(height as usize, width as usize, channels),
            ImgData::U16(unsafe {
                let len = px.len() / 2;
                let ptr = px.as_ptr();
                let mut vec = Vec::with_capacity(len);

                for i in 0..len {
                    let lo = *ptr.add(i * 2) as u16;
                    let hi = *ptr.add(i * 2 + 1) as u16;
                    vec.push((hi << 8) | lo);
                }
                vec
            }),
        )
    } else {
        SVec::new(Shape::new(height as usize, width as usize, channels), ImgData::U8(px))
    })
}
/// Decode PSD buffer into RGB SVec.
///
/// Converts grayscale PSDs to RGB by replicating channels, preserves alpha if present.
pub fn psd_rgb_decode(buffer: &[u8]) -> Result<SVec, DecodeError> {
    let size_bites: &[u8] = &buffer[14..22];
    let channels = buffer[13];

    let mut decoder = PSDDecoder::new(ZCursor::new(buffer));
    let px = decoder.decode_raw().map_err(|e| PsdDecodingError(format!("{:?}", e)))?;

    let (height, width) = decode_size_psd(size_bites);
    Ok(SVec::new(
        Shape::new(height as usize, width as usize, Some(3)),
        if buffer[23] == 16 {
            ImgData::U16(if channels == 3 {
                unsafe {
                    let len = px.len() / 2;
                    let ptr = px.as_ptr();
                    let mut vec = Vec::with_capacity(len);

                    for i in 0..len {
                        let lo = *ptr.add(i * 2) as u16;
                        let hi = *ptr.add(i * 2 + 1) as u16;
                        vec.push((hi << 8) | lo);
                    }
                    vec
                }
            } else if channels == 1 {
                unsafe {
                    let len = px.len() / 2;
                    let ptr = px.as_ptr();
                    let mut vec = Vec::with_capacity(len);

                    for i in 0..len {
                        let lo = *ptr.add(i * 2) as u16;
                        let hi = *ptr.add(i * 2 + 1) as u16;
                        vec.push((hi << 8) | lo);
                        vec.push((hi << 8) | lo);
                        vec.push((hi << 8) | lo);
                    }
                    vec
                }
            } else {
                return Err(PsdDecodingError(format!("Unexpected channel count = {}", channels)));
            })
        } else if channels == 3 {
            ImgData::U8(px)
        } else if channels == 1 {
            let mut rgb_values = Vec::with_capacity(px.len() * 3);

            for gray in &px {
                rgb_values.extend([*gray, *gray, *gray].iter().copied());
            }
            ImgData::U8(rgb_values)
        } else {
            return Err(PsdDecodingError(format!("Unexpected channel count = {}", channels)));
        },
    ))
}
/// Decode PSD buffer into RGBA SVec, adding full alpha channel.
///
/// Always outputs 4 channels, setting alpha to max value if missing.
pub fn psd_rgba_decode(buffer: &[u8]) -> Result<SVec, DecodeError> {
    let size_bites: &[u8] = &buffer[14..22];
    let channels = buffer[13];

    let mut decoder = PSDDecoder::new(ZCursor::new(buffer));
    let px = decoder.decode_raw().map_err(|e| PsdDecodingError(format!("{:?}", e)))?;

    let (height, width) = decode_size_psd(size_bites);
    Ok(SVec::new(
        Shape::new(height as usize, width as usize, Some(4)),
        if buffer[23] == 16 {
            ImgData::U16(if channels == 3 {
                unsafe {
                    let len = px.len() / 2;
                    let ptr = px.as_ptr();
                    let mut vec = Vec::with_capacity((height * width * 4) as usize);

                    for i in 0..len {
                        let lo = *ptr.add(i * 2) as u16;
                        let hi = *ptr.add(i * 2 + 1) as u16;
                        vec.push((hi << 8) | lo);
                        if i % 3 == 2 {
                            vec.push(u16::MAX); // Добавляем новое значение в конец каждого блока
                        }
                    }
                    vec
                }
            } else if channels == 1 {
                unsafe {
                    let len = px.len() / 2;
                    let ptr = px.as_ptr();
                    let mut vec = Vec::with_capacity((height * width * 4) as usize);

                    for i in 0..len {
                        let lo = *ptr.add(i * 2) as u16;
                        let hi = *ptr.add(i * 2 + 1) as u16;
                        vec.push((hi << 8) | lo);
                        vec.push((hi << 8) | lo);
                        vec.push((hi << 8) | lo);
                        vec.push(u16::MAX);
                    }
                    vec
                }
            } else {
                return Err(PsdDecodingError(format!("Unexpected channel count = {}", channels)));
            })
        } else if channels == 3 {
            let mut vec = Vec::with_capacity((height * width * 4) as usize);
            for i in 0..px.len() / 3 {
                // Добавляем три элемента из старого вектора
                vec.push(vec[i * 3]);
                vec.push(vec[i * 3 + 1]);
                vec.push(vec[i * 3 + 2]);
                vec.push(u8::MAX);
            }
            ImgData::U8(vec)
        } else if channels == 1 {
            let mut rgb_values = Vec::with_capacity(px.len() * 4);

            for gray in &px {
                rgb_values.extend([*gray, *gray, *gray, u8::MAX].iter().copied());
            }
            ImgData::U8(rgb_values)
        } else {
            return Err(PsdDecodingError(format!("Unexpected channel count = {}", channels)));
        },
    ))
}
/// Decode PSD buffer to grayscale SVec, converting RGB using BT.709.
///
/// Produces a single-channel image.
pub fn psd_gray_decode(buffer: &[u8]) -> Result<SVec, DecodeError> {
    let size_bites: &[u8] = &buffer[14..22];
    let channels = buffer[13];

    let mut decoder = PSDDecoder::new(ZCursor::new(buffer));
    let px = decoder.decode_raw().map_err(|e| PsdDecodingError(format!("{:?}", e)))?;

    let (height, width) = decode_size_psd(size_bites);
    Ok(SVec::new(
        Shape::new(height as usize, width as usize, None),
        if buffer[23] == 16 {
            ImgData::U16(if channels == 3 {
                unsafe {
                    let len = px.len() / 6; // Так как каждый пиксель состоит из 3 компонентов, каждый по 2 байта
                    let ptr = px.as_ptr();
                    let mut vec = Vec::with_capacity(len);

                    for i in 0..len {
                        // Чтение по 2 байта для каждого компонента (R, G, B)
                        let r_lo = *ptr.add(i * 6) as u16; // младший байт для R
                        let r_hi = *ptr.add(i * 6 + 1) as u16; // старший байт для R
                        let g_lo = *ptr.add(i * 6 + 2) as u16; // младший байт для G
                        let g_hi = *ptr.add(i * 6 + 3) as u16; // старший байт для G
                        let b_lo = *ptr.add(i * 6 + 4) as u16; // младший байт для B
                        let b_hi = *ptr.add(i * 6 + 5) as u16; // старший байт для B

                        // Собираем компоненты RGB из двух байт
                        let r = (r_hi << 8) | r_lo; // R (16 бит)
                        let g = (g_hi << 8) | g_lo; // G (16 бит)
                        let b = (b_hi << 8) | b_lo; // B (16 бит)

                        // Преобразование в яркость по стандарту BT.709
                        let gray = (0.2126 * r as f32 + 0.7152 * g as f32 + 0.0722 * b as f32) as u16;

                        // Добавляем преобразованный серый цвет
                        vec.push(gray);
                    }

                    vec
                }
            } else if channels == 1 {
                unsafe {
                    let len = px.len() / 2;
                    let ptr = px.as_ptr();
                    let mut vec = Vec::with_capacity(len);

                    for i in 0..len {
                        let lo = *ptr.add(i * 2) as u16;
                        let hi = *ptr.add(i * 2 + 1) as u16;
                        vec.push((hi << 8) | lo);
                    }
                    vec
                }
            } else {
                return Err(PsdDecodingError(format!("Unexpected channel count = {}", channels)));
            })
        } else if channels == 1 {
            ImgData::U8(px)
        } else if channels == 3 {
            let mut values = Vec::with_capacity(px.len() / 3);

            for rgb in px.chunks(3) {
                values.push((rgb[0] as f32 * 0.2126 + rgb[1] as f32 * 0.7152 + rgb[2] as f32 * 0.0722) as u8);
            }
            ImgData::U8(values)
        } else {
            return Err(PsdDecodingError(format!("Unexpected channel count = {}", channels)));
        },
    ))
}
/// Decode PSD buffer to grayscale with alpha SVec.
///
/// Outputs two channels: brightness and full alpha
pub fn psd_graya_decode(buffer: &[u8]) -> Result<SVec, DecodeError> {
    let size_bites: &[u8] = &buffer[14..22];
    let channels = buffer[13];

    let mut decoder = PSDDecoder::new(ZCursor::new(buffer));
    let px = decoder.decode_raw().map_err(|e| PsdDecodingError(format!("{:?}", e)))?;

    let (height, width) = decode_size_psd(size_bites);
    Ok(SVec::new(
        Shape::new(height as usize, width as usize, Some(2)),
        if buffer[23] == 16 {
            ImgData::U16(if channels == 3 {
                unsafe {
                    let len = px.len() / 3; // Так как каждый пиксель состоит из 3 компонентов, каждый по 2 байта
                    let ptr = px.as_ptr();
                    let mut vec = Vec::with_capacity(len);

                    for i in 0..len {
                        // Чтение по 2 байта для каждого компонента (R, G, B)
                        let r_lo = *ptr.add(i * 6) as u16; // младший байт для R
                        let r_hi = *ptr.add(i * 6 + 1) as u16; // старший байт для R
                        let g_lo = *ptr.add(i * 6 + 2) as u16; // младший байт для G
                        let g_hi = *ptr.add(i * 6 + 3) as u16; // старший байт для G
                        let b_lo = *ptr.add(i * 6 + 4) as u16; // младший байт для B
                        let b_hi = *ptr.add(i * 6 + 5) as u16; // старший байт для B

                        // Собираем компоненты RGB из двух байт
                        let r = (r_hi << 8) | r_lo; // R (16 бит)
                        let g = (g_hi << 8) | g_lo; // G (16 бит)
                        let b = (b_hi << 8) | b_lo; // B (16 бит)

                        // Преобразование в яркость по стандарту BT.709
                        let gray = (0.2126 * r as f32 + 0.7152 * g as f32 + 0.0722 * b as f32) as u16;

                        // Добавляем преобразованный серый цвет
                        vec.push(gray);
                        vec.push(u16::MAX)
                    }

                    vec
                }
            } else if channels == 1 {
                unsafe {
                    let len = px.len() / 2;
                    let ptr = px.as_ptr();
                    let mut vec = Vec::with_capacity(len);

                    for i in 0..len {
                        let lo = *ptr.add(i * 2) as u16;
                        let hi = *ptr.add(i * 2 + 1) as u16;
                        vec.push((hi << 8) | lo);
                        vec.push(u16::MAX)
                    }
                    vec
                }
            } else {
                return Err(PsdDecodingError(format!("Unexpected channel count = {}", channels)));
            })
        } else if channels == 1 {
            ImgData::U8(px.iter().flat_map(|&x| vec![x, u8::MAX]).collect())
        } else if channels == 3 {
            let mut values = Vec::with_capacity(px.len() / 3);

            for rgb in px.chunks(3) {
                values.push((rgb[0] as f32 * 0.2126 + rgb[1] as f32 * 0.7152 + rgb[2] as f32 * 0.0722) as u8);
                values.push(u8::MAX)
            }
            ImgData::U8(values)
        } else {
            return Err(PsdDecodingError(format!("Unexpected channel count = {}", channels)));
        },
    ))
}

/// Decode common image buffer into dynamic SVec (all color modes).
///
/// Uses `image` crate to detect format and return proper channel count.
pub fn img_din_decode(buffer: &[u8]) -> Result<SVec, DecodeError> {
    let img = image::ImageReader::new(Cursor::new(buffer))
        .with_guessed_format()
        .map_err(|e| ImgDecodingError(format!("{:?}", e)))?
        .decode()
        .map_err(|e| ImgDecodingError(format!("{:?}", e)))?;
    let width = img.width() as usize;
    let height = img.height() as usize;
    Ok(match &img {
        DynamicImage::ImageLuma8(img) => SVec::new(Shape::new(height, width, None), ImgData::U8(img.as_raw().clone())),
        DynamicImage::ImageLumaA8(img) => SVec::new(Shape::new(height, width, Some(2)), ImgData::U8(img.as_raw().clone())),
        DynamicImage::ImageRgb8(img) => SVec::new(Shape::new(height, width, Some(3)), ImgData::U8(img.as_raw().clone())),
        DynamicImage::ImageRgba8(img) => SVec::new(Shape::new(height, width, Some(4)), ImgData::U8(img.as_raw().clone())),
        DynamicImage::ImageLuma16(img) => SVec::new(Shape::new(height, width, None), ImgData::U16(img.as_raw().clone())),
        DynamicImage::ImageLumaA16(img) => SVec::new(Shape::new(height, width, Some(2)), ImgData::U16(img.as_raw().clone())),
        DynamicImage::ImageRgb16(img) => SVec::new(Shape::new(height, width, Some(3)), ImgData::U16(img.as_raw().clone())),
        DynamicImage::ImageRgba16(img) => SVec::new(Shape::new(height, width, Some(4)), ImgData::U16(img.as_raw().clone())),
        DynamicImage::ImageRgb32F(img) => SVec::new(Shape::new(height, width, Some(3)), ImgData::F32(img.as_raw().clone())),
        DynamicImage::ImageRgba32F(img) => SVec::new(Shape::new(height, width, Some(4)), ImgData::F32(img.as_raw().clone())),
        _ => return Err(ImgDecodingError("Unsupported image color mod".to_string())),
    })
}
pub fn img_rgb_decode(buffer: &[u8]) -> Result<SVec, DecodeError> {
    let img = image::ImageReader::new(Cursor::new(buffer))
        .with_guessed_format()
        .map_err(|e| ImgDecodingError(format!("{:?}", e)))?
        .decode()
        .map_err(|e| ImgDecodingError(format!("{:?}", e)))?;
    let width = img.width() as usize;
    let height = img.height() as usize;
    Ok(SVec::new(
        Shape::new(height, width, Some(3)),
        match &img {
            DynamicImage::ImageLuma8(_) | DynamicImage::ImageLumaA8(_) | DynamicImage::ImageRgba8(_) => {
                ImgData::U8(img.to_rgb8().as_raw().clone())
            }

            DynamicImage::ImageRgb8(img) => ImgData::U8(img.as_raw().clone()),

            DynamicImage::ImageLuma16(_) | DynamicImage::ImageLumaA16(_) | DynamicImage::ImageRgba16(_) => {
                ImgData::U16(img.to_rgb16().as_raw().clone())
            }

            DynamicImage::ImageRgb16(img) => ImgData::U16(img.as_raw().clone()),

            DynamicImage::ImageRgb32F(img) => ImgData::F32(img.as_raw().clone()),

            DynamicImage::ImageRgba32F(_) => ImgData::F32(img.to_rgb32f().as_raw().clone()),

            _ => return Err(ImgDecodingError("Unsupported image color mod".to_string())),
        },
    ))
}
pub fn img_rgba_decode(buffer: &[u8]) -> Result<SVec, DecodeError> {
    let img = image::ImageReader::new(Cursor::new(buffer))
        .with_guessed_format()
        .map_err(|e| ImgDecodingError(format!("{:?}", e)))?
        .decode()
        .map_err(|e| ImgDecodingError(format!("{:?}", e)))?;
    let width = img.width() as usize;
    let height = img.height() as usize;
    Ok(SVec::new(
        Shape::new(height, width, Some(4)),
        match &img {
            DynamicImage::ImageLuma8(_) | DynamicImage::ImageLumaA8(_) | DynamicImage::ImageRgb8(_) => {
                ImgData::U8(img.to_rgba8().as_raw().clone())
            }

            DynamicImage::ImageRgba8(img) => ImgData::U8(img.as_raw().clone()),

            DynamicImage::ImageLuma16(_) | DynamicImage::ImageLumaA16(_) | DynamicImage::ImageRgb16(_) => {
                ImgData::U16(img.to_rgba16().as_raw().clone())
            }

            DynamicImage::ImageRgba16(img) => ImgData::U16(img.as_raw().clone()),

            DynamicImage::ImageRgb32F(_) => ImgData::F32(img.to_rgba32f().as_raw().clone()),
            DynamicImage::ImageRgba32F(img) => ImgData::F32(img.as_raw().clone()),

            _ => return Err(ImgDecodingError("Unsupported image color mod".to_string())),
        },
    ))
}
pub fn img_gray_decode(buffer: &[u8]) -> Result<SVec, DecodeError> {
    let img = image::ImageReader::new(Cursor::new(buffer))
        .with_guessed_format()
        .map_err(|e| ImgDecodingError(format!("{:?}", e)))?
        .decode()
        .map_err(|e| ImgDecodingError(format!("{:?}", e)))?;
    let width = img.width() as usize;
    let height = img.height() as usize;
    Ok(SVec::new(
        Shape::new(height, width, None),
        match &img {
            DynamicImage::ImageRgba8(_) | DynamicImage::ImageLumaA8(_) | DynamicImage::ImageRgb8(_) => {
                ImgData::U8(img.to_luma8().as_raw().clone())
            }
            DynamicImage::ImageLuma8(img) => ImgData::U8(img.as_raw().clone()),
            DynamicImage::ImageRgba16(_) | DynamicImage::ImageLumaA16(_) | DynamicImage::ImageRgb16(_) => {
                ImgData::U16(img.to_luma16().as_raw().clone())
            }
            DynamicImage::ImageLuma16(img) => ImgData::U16(img.as_raw().clone()),
            DynamicImage::ImageRgb32F(_) | DynamicImage::ImageRgba32F(_) => ImgData::F32(img.to_luma32f().as_raw().clone()),
            _ => return Err(ImgDecodingError("Unsupported image color mod".to_string())),
        },
    ))
}
pub fn img_graya_decode(buffer: &[u8]) -> Result<SVec, DecodeError> {
    let img = image::ImageReader::new(Cursor::new(buffer))
        .with_guessed_format()
        .map_err(|e| ImgDecodingError(format!("{:?}", e)))?
        .decode()
        .map_err(|e| ImgDecodingError(format!("{:?}", e)))?;
    let width = img.width() as usize;
    let height = img.height() as usize;
    Ok(SVec::new(
        Shape::new(height, width, None),
        match &img {
            DynamicImage::ImageRgba8(_) | DynamicImage::ImageLuma8(_) | DynamicImage::ImageRgb8(_) => {
                ImgData::U8(img.to_luma_alpha8().as_raw().clone())
            }
            DynamicImage::ImageLumaA8(img) => ImgData::U8(img.as_raw().clone()),
            DynamicImage::ImageRgba16(_) | DynamicImage::ImageLuma16(_) | DynamicImage::ImageRgb16(_) => {
                ImgData::U16(img.to_luma_alpha16().as_raw().clone())
            }
            DynamicImage::ImageLumaA16(img) => ImgData::U16(img.as_raw().clone()),
            DynamicImage::ImageRgb32F(_) | DynamicImage::ImageRgba32F(_) => ImgData::F32(img.to_luma_alpha32f().as_raw().clone()),
            _ => return Err(ImgDecodingError("Unsupported image color mod".to_string())),
        },
    ))
}
