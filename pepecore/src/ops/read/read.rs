//! Module providing high-level API to read image or PSD files/buffers into `SVec`.
//!
//! This module exports two functions: `read_in_path` and `read_in_buffer`, supporting various image color modes
//! (dynamic, gray, rgb, rgba, gray+alpha) and automatically detecting PSD format via magic bytes.
//!
//! # Examples
//!
//! ```rust
//! use pepecore::read::{read_in_path,read_in_buffer};
//! use pepecore_array::{SVec};
//! use pepecore::enums::ImgColor;
//!
//! // Read a JPEG file as RGB image:
//! let svec: SVec = read_in_path("photo.jpg", ImgColor::RGB)
//!     .expect("Failed to read image");
//! assert_eq!(svec.shape.get_channels(), Some(3));
//!
//! // Read a PSD file from buffer dynamically:
//! let buffer: Vec<u8> = std::fs::read("design.psd").unwrap();
//! let dynamic = read_in_buffer(&buffer, ImgColor::DYNAMIC)
//!     .expect("Failed to decode PSD");
//! println!("PSD size: {:?}", dynamic.shape());
//! ```

use crate::enums::ImgColor;
use crate::errors::DecodeError;
use crate::errors::DecodeError::FileOpenError;
use crate::ops::read::decode::{
    img_din_decode, img_gray_decode, img_graya_decode, img_rgb_decode, img_rgba_decode, psd_din_decode, psd_gray_decode,
    psd_graya_decode, psd_rgb_decode, psd_rgba_decode,
};
use filebuffer::FileBuffer;
use pepecore_array::SVec;
use std::fmt::Debug;
use std::path::Path;
/// Read image from file path into `SVec`, choosing decoder by `ImgColor` and format.
///
/// Automatically detects PSD files by magic bytes `56 66 80 83`.
/// For non-PSD, delegates to common image decoders.
///
/// # Parameters
///
/// - `path`: Path on filesystem to image or PSD file.
/// - `img_color`: Desired output color mode (`DYNAMIC`, `GRAY`, `RGB`, `RGBA`, `GRAYA`).
///
/// # Errors
///
/// Returns `FileOpenError` if file cannot be opened, or relevant decoder errors.
///
/// # Examples
///
/// ```rust
/// use pepecore::read::read_in_path;
/// use pepecore::enums::ImgColor;
///
/// let svec = read_in_path("diagram.psd", ImgColor::RGBA).unwrap();
/// assert_eq!(svec.shape.get_channels(), Some(4));
/// ```

pub fn read_in_path<P: Debug + AsRef<Path> + ?Sized>(path: &P, img_color: ImgColor) -> Result<SVec, DecodeError> {
    let img_buffer = FileBuffer::open(path).map_err(|e| FileOpenError(format!("Path: {:?} FileBuffer error: {:?}", path, e)))?;
    Ok(match &img_buffer[..4] {
        [56, 66, 80, 83] => match img_color {
            ImgColor::DYNAMIC => psd_din_decode(&img_buffer)?,
            ImgColor::GRAY => psd_gray_decode(&img_buffer)?,
            ImgColor::RGB => psd_rgb_decode(&img_buffer)?,
            ImgColor::RGBA => psd_rgba_decode(&img_buffer)?,
            ImgColor::GRAYA => psd_graya_decode(&img_buffer)?,
        },
        _ => match img_color {
            ImgColor::DYNAMIC => img_din_decode(&img_buffer)?,
            ImgColor::GRAY => img_gray_decode(&img_buffer)?,
            ImgColor::RGB => img_rgb_decode(&img_buffer)?,
            ImgColor::RGBA => img_rgba_decode(&img_buffer)?,
            ImgColor::GRAYA => img_graya_decode(&img_buffer)?,
        },
    })
}
/// Decode image from in-memory buffer into `SVec`, choosing decoder by `ImgColor`.
///
/// Uses the same auto-detection logic as `read_in_path`, except without file I/O.
///
/// # Parameters
///
/// - `img_buffer`: Byte slice of image or PSD data.
/// - `img_color`: Desired output color mode.
///
/// # Errors
///
/// Returns relevant decoder `DecodeError`.
///
/// # Examples
///
/// ```rust
/// use pepecore::read::read_in_buffer;
/// use pepecore::enums::ImgColor;
///
/// let data: Vec<u8> = std::fs::read("image.png").unwrap();
/// let gray = read_in_buffer(&data, ImgColor::GRAY).unwrap();
/// assert_eq!(gray.shape.get_channels(), None);
/// ```
pub fn read_in_buffer(img_buffer: &[u8], img_color: ImgColor) -> Result<SVec, DecodeError> {
    Ok(match &img_buffer[..4] {
        [56, 66, 80, 83] => match img_color {
            ImgColor::DYNAMIC => psd_din_decode(img_buffer)?,
            ImgColor::GRAY => psd_gray_decode(img_buffer)?,
            ImgColor::RGB => psd_rgb_decode(img_buffer)?,
            ImgColor::RGBA => psd_rgba_decode(img_buffer)?,
            ImgColor::GRAYA => psd_graya_decode(img_buffer)?,
        },
        _ => match img_color {
            ImgColor::DYNAMIC => img_din_decode(img_buffer)?,
            ImgColor::GRAY => img_gray_decode(img_buffer)?,
            ImgColor::RGB => img_rgb_decode(img_buffer)?,
            ImgColor::RGBA => img_rgba_decode(img_buffer)?,
            ImgColor::GRAYA => img_graya_decode(img_buffer)?,
        },
    })
}
