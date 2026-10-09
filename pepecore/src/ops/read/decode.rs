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
use photocraft_psd::PsdFile;
/// Decode merged PSD image into planar big-endian samples plus geometry.
///
/// Returns `(channels, height, width, depth, planar_bytes)` where planar data
/// is `channels × height × width` samples in file order (native big-endian
/// for 16-bit). Only 8-bit and 16-bit depths are supported for pixel output.
fn decode_merged_planar(buffer: &[u8]) -> Result<(usize, usize, usize, u16, Vec<u8>), DecodeError> {
    let file = PsdFile::from_bytes(buffer).map_err(|e| PsdDecodingError(format!("{:?}", e)))?;
    let header = &file.header;
    let depth = header.depth;
    if depth != 8 && depth != 16 {
        return Err(PsdDecodingError(format!("Unsupported PSD bit depth = {}", depth)));
    }
    let height = header.height as usize;
    let width = header.width as usize;
    let channels = header.channels as usize;
    if channels == 0 || channels > 56 {
        return Err(PsdDecodingError(format!("Unexpected channel count = {}", channels)));
    }
    let planar = file.decode_merged().map_err(|e| PsdDecodingError(format!("{:?}", e)))?;
    let sample_bytes = (depth as usize) / 8;
    let expected = channels * height * width * sample_bytes;
    if planar.len() != expected {
        return Err(PsdDecodingError(format!(
            "PSD payload size mismatch: got {} bytes, expected {}",
            planar.len(),
            expected
        )));
    }
    Ok((channels, height, width, depth, planar))
}
/// Interleave planar 8-bit channels into pixel order.
fn interleave_u8(planar: &[u8], channels: usize, pixels: usize) -> Vec<u8> {
    let mut out = vec![0u8; pixels * channels];
    for (ch, plane) in planar.chunks(pixels).enumerate() {
        for (i, &v) in plane.iter().enumerate() {
            out[i * channels + ch] = v;
        }
    }
    out
}
/// Interleave planar big-endian 16-bit channels into native u16 pixels.
fn interleave_u16_be(planar: &[u8], channels: usize, pixels: usize) -> Vec<u16> {
    let mut out = vec![0u16; pixels * channels];
    for (ch, plane) in planar.chunks(pixels * 2).enumerate() {
        for (i, pair) in plane.chunks_exact(2).enumerate() {
            out[i * channels + ch] = u16::from_be_bytes([pair[0], pair[1]]);
        }
    }
    out
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
    let (channels, height, width, depth, planar) = decode_merged_planar(buffer)?;
    let shape_channels = if channels > 1 { Some(channels) } else { None };
    let pixels = height * width;
    Ok(if depth == 16 {
        SVec::new(
            Shape::new(height, width, shape_channels),
            ImgData::U16(interleave_u16_be(&planar, channels, pixels)),
        )
    } else {
        SVec::new(
            Shape::new(height, width, shape_channels),
            ImgData::U8(interleave_u8(&planar, channels, pixels)),
        )
    })
}
/// Decode PSD buffer into RGB SVec.
///
/// Converts grayscale PSDs to RGB by replicating channels, preserves alpha if present.
pub fn psd_rgb_decode(buffer: &[u8]) -> Result<SVec, DecodeError> {
    let (channels, height, width, depth, planar) = decode_merged_planar(buffer)?;
    let pixels = height * width;
    let shape = Shape::new(height, width, Some(3));
    if depth == 16 {
        let px = interleave_u16_be(&planar, channels, pixels);
        let data = if channels == 3 {
            px
        } else if channels == 1 {
            let mut rgb = Vec::with_capacity(pixels * 3);
            for gray in &px {
                rgb.extend_from_slice(&[*gray, *gray, *gray]);
            }
            rgb
        } else if channels == 4 {
            let mut rgb = Vec::with_capacity(pixels * 3);
            for rgba in px.chunks_exact(4) {
                rgb.extend_from_slice(&rgba[..3]);
            }
            rgb
        } else {
            return Err(PsdDecodingError(format!("Unexpected channel count = {}", channels)));
        };
        Ok(SVec::new(shape, ImgData::U16(data)))
    } else {
        let px = interleave_u8(&planar, channels, pixels);
        let data = if channels == 3 {
            px
        } else if channels == 1 {
            let mut rgb = Vec::with_capacity(pixels * 3);
            for gray in &px {
                rgb.extend_from_slice(&[*gray, *gray, *gray]);
            }
            rgb
        } else if channels == 4 {
            let mut rgb = Vec::with_capacity(pixels * 3);
            for rgba in px.chunks_exact(4) {
                rgb.extend_from_slice(&rgba[..3]);
            }
            rgb
        } else {
            return Err(PsdDecodingError(format!("Unexpected channel count = {}", channels)));
        };
        Ok(SVec::new(shape, ImgData::U8(data)))
    }
}
/// Decode PSD buffer into RGBA SVec, adding full alpha channel.
///
/// Always outputs 4 channels, setting alpha to max value if missing.
pub fn psd_rgba_decode(buffer: &[u8]) -> Result<SVec, DecodeError> {
    let (channels, height, width, depth, planar) = decode_merged_planar(buffer)?;
    let pixels = height * width;
    let shape = Shape::new(height, width, Some(4));
    if depth == 16 {
        let px = interleave_u16_be(&planar, channels, pixels);
        let data = if channels == 4 {
            px
        } else if channels == 3 {
            let mut rgba = Vec::with_capacity(pixels * 4);
            for rgb in px.chunks_exact(3) {
                rgba.extend_from_slice(rgb);
                rgba.push(u16::MAX);
            }
            rgba
        } else if channels == 1 {
            let mut rgba = Vec::with_capacity(pixels * 4);
            for gray in &px {
                rgba.extend_from_slice(&[*gray, *gray, *gray, u16::MAX]);
            }
            rgba
        } else {
            return Err(PsdDecodingError(format!("Unexpected channel count = {}", channels)));
        };
        Ok(SVec::new(shape, ImgData::U16(data)))
    } else {
        let px = interleave_u8(&planar, channels, pixels);
        let data = if channels == 4 {
            px
        } else if channels == 3 {
            let mut rgba = Vec::with_capacity(pixels * 4);
            for rgb in px.chunks_exact(3) {
                rgba.extend_from_slice(rgb);
                rgba.push(u8::MAX);
            }
            rgba
        } else if channels == 1 {
            let mut rgba = Vec::with_capacity(pixels * 4);
            for gray in &px {
                rgba.extend_from_slice(&[*gray, *gray, *gray, u8::MAX]);
            }
            rgba
        } else {
            return Err(PsdDecodingError(format!("Unexpected channel count = {}", channels)));
        };
        Ok(SVec::new(shape, ImgData::U8(data)))
    }
}
/// BT.709 luma for 8-bit RGB triples.
fn luma_u8(rgb: &[u8]) -> u8 {
    (rgb[0] as f32 * 0.2126 + rgb[1] as f32 * 0.7152 + rgb[2] as f32 * 0.0722) as u8
}
/// BT.709 luma for 16-bit RGB triples.
fn luma_u16(rgb: &[u16]) -> u16 {
    (0.2126 * rgb[0] as f32 + 0.7152 * rgb[1] as f32 + 0.0722 * rgb[2] as f32) as u16
}
/// Decode PSD buffer to grayscale SVec, converting RGB using BT.709.
///
/// Produces a single-channel image.
pub fn psd_gray_decode(buffer: &[u8]) -> Result<SVec, DecodeError> {
    let (channels, height, width, depth, planar) = decode_merged_planar(buffer)?;
    let pixels = height * width;
    let shape = Shape::new(height, width, None);
    if depth == 16 {
        let px = interleave_u16_be(&planar, channels, pixels);
        let data = if channels == 3 {
            px.chunks_exact(3).map(luma_u16).collect()
        } else if channels == 1 {
            px
        } else if channels == 4 {
            px.chunks_exact(4).map(|rgba| luma_u16(&rgba[..3])).collect()
        } else {
            return Err(PsdDecodingError(format!("Unexpected channel count = {}", channels)));
        };
        Ok(SVec::new(shape, ImgData::U16(data)))
    } else {
        let px = interleave_u8(&planar, channels, pixels);
        let data = if channels == 1 {
            px
        } else if channels == 3 {
            px.chunks_exact(3).map(luma_u8).collect()
        } else if channels == 4 {
            px.chunks_exact(4).map(|rgba| luma_u8(&rgba[..3])).collect()
        } else {
            return Err(PsdDecodingError(format!("Unexpected channel count = {}", channels)));
        };
        Ok(SVec::new(shape, ImgData::U8(data)))
    }
}
/// Decode PSD buffer to grayscale with alpha SVec.
///
/// Outputs two channels: brightness and full alpha
pub fn psd_graya_decode(buffer: &[u8]) -> Result<SVec, DecodeError> {
    let (channels, height, width, depth, planar) = decode_merged_planar(buffer)?;
    let pixels = height * width;
    let shape = Shape::new(height, width, Some(2));
    if depth == 16 {
        let px = interleave_u16_be(&planar, channels, pixels);
        let mut data = Vec::with_capacity(pixels * 2);
        if channels == 3 || channels == 4 {
            let step = channels;
            for pix in px.chunks_exact(step) {
                data.push(luma_u16(&pix[..3]));
                data.push(u16::MAX);
            }
        } else if channels == 1 {
            for gray in &px {
                data.push(*gray);
                data.push(u16::MAX);
            }
        } else if channels == 2 {
            for ga in px.chunks_exact(2) {
                data.push(ga[0]);
                data.push(ga[1]);
            }
        } else {
            return Err(PsdDecodingError(format!("Unexpected channel count = {}", channels)));
        }
        Ok(SVec::new(shape, ImgData::U16(data)))
    } else {
        let px = interleave_u8(&planar, channels, pixels);
        let mut data = Vec::with_capacity(pixels * 2);
        if channels == 3 || channels == 4 {
            let step = channels;
            for pix in px.chunks_exact(step) {
                data.push(luma_u8(&pix[..3]));
                data.push(u8::MAX);
            }
        } else if channels == 1 {
            for gray in &px {
                data.push(*gray);
                data.push(u8::MAX);
            }
        } else if channels == 2 {
            for ga in px.chunks_exact(2) {
                data.push(ga[0]);
                data.push(ga[1]);
            }
        } else {
            return Err(PsdDecodingError(format!("Unexpected channel count = {}", channels)));
        }
        Ok(SVec::new(shape, ImgData::U8(data)))
    }
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
