//! Module providing halftone and rotated halftone operations on `SVec` images.
//!
//! Applies threshold-based dot patterns per channel to simulate printing halftone effects.
//! Supports different pixel types (`u8`, `u16`, `f32`) and customizable dot sizes, shapes, and rotations.
//!
//! # Examples
//!
//! ```rust
//! use pepecore::{halftone, rotate_halftone};
//! use pepecore::array::svec::SVec;
//! use pepecore::enums::{DotType, ImgData, PixelType};
//! use pepecore::svec::Shape;
//! // Create or load a grayscale SVec (single-channel u8)
//! let mut img = SVec::new(Shape::new(100, 100, None), ImgData::U8(vec![128; 10000]));
//! // Apply non-rotated halftone with dot size 5 for the single channel
//! halftone(&mut img, &[5], &[DotType::CIRCLE]).unwrap();
//! // Apply rotated halftone on a color image
//! let mut rgb = SVec::new(Shape::new(50, 50, Some(3)), ImgData::U8(vec![200; 7500]));
//! let sizes = vec![4, 6, 8];
//! let angles = vec![15.0, 45.0, 75.0];
//! let types = vec![DotType::CROSS, DotType::CIRCLE, DotType::ELLIPSE];
//! rotate_halftone(&mut rgb, &sizes, &angles, &types).unwrap();
//! ```
use crate::array::svec::SVec;
use crate::enums::{DotType, PixelType};
use crate::errors::HalftoneError;
use crate::ops::svec_ops::halftone::dot::dot_create;
use crate::ops::svec_ops::halftone::utils::{HalftonePixel, compute_cos_sin, rotate_pixel_coordinates};
use std::fmt::Debug;

/// Apply a standard (non-rotated) halftone to the image.
///
/// Generates per-channel dot matrices using `dot_sizes` and `dot_type`,
/// then applies thresholding to each pixel: if the pixel value is below the
/// dot matrix threshold, sets to `MIN_VALUE`, otherwise to `MAX_VALUE`.
///
/// # Parameters
///
/// - `img`: mutable reference to the `SVec` image.
/// - `dot_sizes`: array of dot sizes per channel (length must match channel count).
/// - `dot_type`: array of `DotType` specifying dot shape per channel.
///
/// # Errors
///
/// Returns `HalftoneError` if:
/// - Unable to access image data or channels.
/// - `dot_sizes` or `dot_type` length is smaller than channel count.
///
fn apply_halftone<T>(img: &mut SVec, dot_sizes: &[usize], dot_type: &[DotType]) -> Result<(), HalftoneError>
where
    T: HalftonePixel + Debug,
{
    // Retrieve image shape and data buffer
    let (height, width, channels_opt) = img.shape();
    let data = img
        .get_data_mut::<T>()
        .map_err(|e| HalftoneError::GetDataError(format!("{:?}", e)))?;
    let channels = channels_opt.ok_or(HalftoneError::NoChannelsError)?;

    // Ensure that dot_sizes matches number of channels
    if dot_sizes.len() < channels || dot_type.len() < channels {
        return Err(HalftoneError::DotSizeMismatch(dot_sizes.len(), channels));
    }

    // Prepare biases, doubled sizes, and per-channel dot matrices
    let mut biases = Vec::with_capacity(channels);
    let mut double_sizes = Vec::with_capacity(channels);
    let mut dot_matrices = Vec::with_capacity(channels);

    for index in 0..channels {
        let size = dot_sizes[index];
        let bias = size / 2;
        let doubled = size * 2;
        let matrix = if size > 0 {
            let kernel = dot_create(size, &dot_type[index]);
            let kernel_data = kernel
                .get_data::<f32>()
                .map_err(|e| HalftoneError::GetDataError(format!("{:?}", e)))?;
            T::prepare_dot_matrix(kernel_data)
        } else {
            Vec::new()
        };
        biases.push(bias);
        double_sizes.push(doubled);
        dot_matrices.push(matrix);
    }

    // Apply thresholding per pixel and channel
    for y in 0..height {
        for x in 0..width {
            for c in 0..channels {
                let ds = double_sizes[c];
                if ds == 0 {
                    continue;
                }
                let bias = biases[c];
                let offset_y = (y + bias) % ds;
                let idx_in_matrix = (x + bias) % ds + offset_y * ds;
                let idx = (y * width + x) * channels + c;

                data[idx] = if data[idx] < dot_matrices[c][idx_in_matrix] {
                    T::MIN_VALUE
                } else {
                    T::MAX_VALUE
                };
            }
        }
    }

    Ok(())
}

/// Apply a rotated halftone to the image.
///
/// Similar to `apply_halftone`, but first rotates each pixel coordinate by
/// the angle specified in `angles` (degrees) around image center, producing
/// rotated dot alignment.
///
/// # Parameters
///
/// - `img`: mutable reference to the `SVec`.
/// - `dot_sizes`: array of dot sizes per channel.
/// - `angles`: array of rotation angles in degrees per channel.
/// - `dot_type`: array of `DotType` per channel.
///
/// # Errors
///
/// Returns `HalftoneError` on data access failures or mismatched array lengths.
///
fn apply_rotate_halftone<T>(
    img: &mut SVec,
    dot_sizes: &[usize],
    angles: &[f32],
    dot_type: &[DotType],
) -> Result<(), HalftoneError>
where
    T: HalftonePixel + Debug,
{
    // Retrieve image shape and data buffer
    let (height, width, channels_opt) = img.shape();
    let data = img
        .get_data_mut::<T>()
        .map_err(|e| HalftoneError::GetDataError(format!("{:?}", e)))?;
    let channels = channels_opt.ok_or(HalftoneError::NoChannelsError)?;

    // Ensure dot_sizes and angles arrays match number of channels
    if dot_sizes.len() < channels || angles.len() < channels || dot_type.len() < channels {
        return Err(HalftoneError::DotSizeMismatch(dot_sizes.len().max(angles.len()), channels));
    }

    let x_center = width as f32 / 2.0;
    let y_center = height as f32 / 2.0;
    let mut double_sizes = Vec::with_capacity(channels);
    let mut dot_matrices = Vec::with_capacity(channels);
    let mut cos_sin = Vec::with_capacity(channels);

    // Precompute matrices and rotation sin/cos for each channel
    for i in 0..channels {
        let size = dot_sizes[i];
        let doubled = size * 2;
        let matrix = if size > 0 {
            let kernel = dot_create(size, &dot_type[i]);
            let kernel_data = kernel
                .get_data::<f32>()
                .map_err(|e| HalftoneError::GetDataError(format!("{:?}", e)))?;
            T::prepare_dot_matrix(kernel_data)
        } else {
            Vec::new()
        };
        double_sizes.push(doubled);
        dot_matrices.push(matrix);
        cos_sin.push(compute_cos_sin(angles[i].to_radians()));
    }

    // Apply rotated halftone per pixel and channel
    for y in 0..height {
        for x in 0..width {
            for c in 0..channels {
                let ds = double_sizes[c];
                if ds == 0 {
                    continue;
                }
                // Rotate current pixel coords around image center
                let (rx, ry) = rotate_pixel_coordinates(x as f32, y as f32, x_center, y_center, cos_sin[c][0], cos_sin[c][1]);
                let ix = rx % ds;
                let iy = ry % ds;
                let idx_in_matrix = ix + iy * ds;
                let idx = (y * width + x) * channels + c;

                data[idx] = if data[idx] < dot_matrices[c][idx_in_matrix] {
                    T::MIN_VALUE
                } else {
                    T::MAX_VALUE
                };
            }
        }
    }

    Ok(())
}

/// Apply non-rotated halftone to `img` dispatching by pixel type.
///
/// # See
/// - `apply_halftone` for detailed behavior.
pub fn halftone(img: &mut SVec, dot_sizes: &[usize], dot_type: &[DotType]) -> Result<(), HalftoneError> {
    match img.pixel_type() {
        PixelType::F32 => apply_halftone::<f32>(img, dot_sizes, dot_type),
        PixelType::U8 => apply_halftone::<u8>(img, dot_sizes, dot_type),
        PixelType::U16 => apply_halftone::<u16>(img, dot_sizes, dot_type),
    }
}

/// Apply rotated halftone to `img` dispatching by pixel type.
///
/// # See
/// - `apply_rotate_halftone` for detailed behavior.
pub fn rotate_halftone(img: &mut SVec, dot_sizes: &[usize], angles: &[f32], dot_type: &[DotType]) -> Result<(), HalftoneError> {
    match img.pixel_type() {
        PixelType::F32 => apply_rotate_halftone::<f32>(img, dot_sizes, angles, dot_type),
        PixelType::U8 => apply_rotate_halftone::<u8>(img, dot_sizes, angles, dot_type),
        PixelType::U16 => apply_rotate_halftone::<u16>(img, dot_sizes, angles, dot_type),
    }
}
