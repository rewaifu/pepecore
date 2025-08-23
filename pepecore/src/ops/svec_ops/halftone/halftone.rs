//! Module providing halftone and rotated halftone operations on `SVec` images.
//!
//! Applies threshold-based dot patterns per channel to simulate printing halftone effects.
//! Supports different pixel types (`u8`, `u16`, `f32`) and customizable dot sizes, shapes, and rotations.
//!
//! # Examples
//!
//! ```rust
//! use pepecore::{halftone, rotate_halftone};
//! use pepecore_array::{SVec,Shape,PixelType,ImgData};
//! use pepecore::enums::{DotType};
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
use crate::array::Shape;
use crate::enums::DotType;
use crate::errors::HalftoneError;
use crate::global_params::rayon_get_mode;
use crate::ops::svec_ops::halftone::dot::dot_create;
use crate::ops::svec_ops::halftone::utils::{HalftonePixel, compute_cos_sin, rotate_pixel_coordinates, wrap_index};
use crate::ops::svec_ops::resize::fir::ResizeSVec;
use fast_image_resize::ResizeAlg;
use pepecore_array::{PixelType, SVec};
use rayon::prelude::*;
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
    let data = img.get_data_mut::<T>()?;
    let channels = channels_opt.ok_or(pepecore_array::error::Error::NoChannelsError)?;

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
            let kernel_data = kernel.get_data::<f32>()?;
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
fn apply_ssaa_halftone<T>(
    img: &mut SVec,
    dot_sizes: &[usize],
    dot_type: &[DotType],
    scale: f32,
    resize_alg: ResizeAlg,
) -> Result<(), HalftoneError>
where
    T: HalftonePixel + Debug + std::marker::Send + std::marker::Sync,
{
    // Retrieve image shape and data buffer
    let (height, width, channels_opt) = img.shape();
    let data = img.get_mut_vec::<T>()?;
    let channels = channels_opt.ok_or(pepecore_array::error::Error::NoChannelsError)?;

    // Ensure that dot_sizes matches number of channels
    if dot_sizes.len() < channels || dot_type.len() < channels {
        return Err(HalftoneError::DotSizeMismatch(dot_sizes.len(), channels));
    }

    // Prepare biases, doubled sizes, and per-channel dot matrices
    let mut biases = Vec::with_capacity(channels);
    let mut double_sizes = Vec::with_capacity(channels);
    let mut dot_matrices = Vec::with_capacity(channels);

    for index in 0..channels {
        let size = (dot_sizes[index] as f32 * scale) as usize;
        let bias = size / 2;
        let doubled = size * 2;
        let matrix = if size > 0 {
            let kernel = dot_create(size, &dot_type[index]);
            let kernel_data = kernel.get_data::<f32>()?;
            T::prepare_dot_matrix(kernel_data)
        } else {
            Vec::new()
        };
        biases.push(bias);
        double_sizes.push(doubled);
        dot_matrices.push(matrix);
    }
    let x_in_tab: Vec<usize> = (0..(width as f32 * scale) as usize)
        .map(|x| ((x as f32 / scale).floor() as usize).min(width - 1))
        .collect();

    let y_in_tab: Vec<usize> = (0..(height as f32 * scale) as usize)
        .map(|y| ((y as f32 / scale).floor() as usize).min(height - 1))
        .collect();
    if rayon_get_mode() {
        let mut new_vec: Vec<T> = (0..(height as f32 * scale) as usize * (width as f32 * scale) as usize * channels)
            .into_par_iter()
            .map(|i| {
                let c = i % channels;
                let lx = (i / channels) % x_in_tab.len();
                let ly = i / (channels * x_in_tab.len());

                let x = x_in_tab[lx];
                let y = y_in_tab[ly];

                let ds = double_sizes[c];
                let idx = (y * width + x) * channels + c;

                if ds == 0 {
                    return data[idx];
                }

                let bias = biases[c];
                let offset_y = (ly + bias) % ds;
                let idx_in_matrix = (lx + bias) % ds + offset_y * ds;

                if data[idx] < dot_matrices[c][idx_in_matrix] {
                    T::MIN_VALUE
                } else {
                    T::MAX_VALUE
                }
            })
            .collect();
        std::mem::swap(data, &mut new_vec);
        img.shape = Shape::new(
            (height as f32 * scale) as usize,
            (width as f32 * scale) as usize,
            channels_opt,
        );
        img.resize(height, width, resize_alg, false);
    } else {
        let mut new_vec: Vec<T> =
            Vec::with_capacity((height as f32 * scale) as usize * (width as f32 * scale) as usize * channels);
        // Apply thresholding per pixel and channel
        for (ly, y) in y_in_tab.iter().enumerate() {
            for (lx, x) in x_in_tab.iter().enumerate() {
                for c in 0..channels {
                    let ds = double_sizes[c];
                    let idx = (y * width + x) * channels + c;
                    // println!("{}",ds);
                    if ds == 0 {
                        new_vec.push(data[idx]);
                        continue;
                    }
                    let bias = biases[c];
                    let offset_y = (ly + bias) % ds;
                    let idx_in_matrix = (lx + bias) % ds + offset_y * ds;

                    new_vec.push(if data[idx] < dot_matrices[c][idx_in_matrix] {
                        T::MIN_VALUE
                    } else {
                        T::MAX_VALUE
                    });
                }
            }
        }
        std::mem::swap(data, &mut new_vec);
        img.shape = Shape::new(
            (height as f32 * scale) as usize,
            (width as f32 * scale) as usize,
            channels_opt,
        );
        img.resize(height, width, resize_alg, false);
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
    let data = img.get_data_mut::<T>()?;
    let channels = channels_opt.ok_or(pepecore_array::error::Error::NoChannelsError)?;

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
            let kernel_data = kernel.get_data::<f32>()?;
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
                let (rx, ry) = rotate_pixel_coordinates(
                    x as f32 + x_center,
                    y as f32 + y_center,
                    width as f32,
                    height as f32,
                    cos_sin[c][0],
                    cos_sin[c][1],
                );
                let dx = wrap_index(rx.round() as i32, ds);
                let dy = wrap_index(ry.round() as i32, ds);

                let idx = (y * width + x) * channels + c;

                data[idx] = if data[idx] < dot_matrices[c][dx + dy * ds] {
                    T::MIN_VALUE
                } else {
                    T::MAX_VALUE
                };
            }
        }
    }

    Ok(())
}
fn apply_ssaa_rotate_halftone<T>(
    img: &mut SVec,
    dot_sizes: &[usize],
    angles: &[f32],
    dot_type: &[DotType],
    scale: f32,
    resize_alg: ResizeAlg,
) -> Result<(), HalftoneError>
where
    T: HalftonePixel + Debug + std::marker::Send + std::marker::Sync,
{
    // Retrieve image shape and data buffer
    let (height, width, channels_opt) = img.shape();
    let data = img.get_mut_vec::<T>()?;
    let channels = channels_opt.ok_or(pepecore_array::error::Error::NoChannelsError)?;
    let (scale_height, scale_width) = ((height as f32 * scale) as usize, (width as f32 * scale) as usize);
    // Ensure dot_sizes and angles arrays match number of channels
    if dot_sizes.len() < channels || angles.len() < channels || dot_type.len() < channels {
        return Err(HalftoneError::DotSizeMismatch(dot_sizes.len().max(angles.len()), channels));
    }

    let x_center = scale_width as f32 / 2.0;
    let y_center = scale_height as f32 / 2.0;
    let mut double_sizes = Vec::with_capacity(channels);
    let mut dot_matrices = Vec::with_capacity(channels);
    let mut cos_sin = Vec::with_capacity(channels);

    // Precompute matrices and rotation sin/cos for each channel
    for i in 0..channels {
        let size = (dot_sizes[i] as f32 * scale) as usize;
        let doubled = size * 2;
        let matrix = if size > 0 {
            let kernel = dot_create(size, &dot_type[i]);
            let kernel_data = kernel.get_data::<f32>()?;
            T::prepare_dot_matrix(kernel_data)
        } else {
            Vec::new()
        };
        double_sizes.push(doubled);
        dot_matrices.push(matrix);
        cos_sin.push(compute_cos_sin(angles[i].to_radians()));
    }
    let x_in_tab: Vec<usize> = (0..(width as f32 * scale) as usize)
        .map(|x| ((x as f32 / scale).floor() as usize).min(width - 1))
        .collect();

    let y_in_tab: Vec<usize> = (0..(height as f32 * scale) as usize)
        .map(|y| ((y as f32 / scale).floor() as usize).min(height - 1))
        .collect();

    if rayon_get_mode() {
        let mut new_vec: Vec<T> = (0..y_in_tab.len() * x_in_tab.len() * channels)
            .into_par_iter()
            .map(|i| {
                let px = i / channels;
                let c = i % channels;

                let ly = px / x_in_tab.len();
                let lx = px % x_in_tab.len();

                let y = y_in_tab[ly];
                let x = x_in_tab[lx];

                let ds = double_sizes[c];
                let idx = (y * width + x) * channels + c;

                if ds == 0 {
                    return data[idx];
                }

                let (rx, ry) = rotate_pixel_coordinates(lx as f32, ly as f32, x_center, y_center, cos_sin[c][0], cos_sin[c][1]);

                let dx = wrap_index(rx.round() as i32, ds);
                let dy = wrap_index(ry.round() as i32, ds);

                if data[idx] < dot_matrices[c][dx + dy * ds] {
                    T::MIN_VALUE
                } else {
                    T::MAX_VALUE
                }
            })
            .collect();

        std::mem::swap(data, &mut new_vec);
        img.shape = Shape::new(scale_height, scale_width, channels_opt);
        img.resize(height, width, resize_alg, false);
    } else {
        let mut new_vec: Vec<T> =
            Vec::with_capacity((height as f32 * scale) as usize * (width as f32 * scale) as usize * channels);
        // Apply rotated halftone per pixel and channel
        for (ly, y) in y_in_tab.iter().enumerate() {
            for (lx, x) in x_in_tab.iter().enumerate() {
                for c in 0..channels {
                    let ds = double_sizes[c];
                    let idx = (y * width + x) * channels + c;
                    if ds == 0 {
                        new_vec.push(data[idx]);
                        continue;
                    }
                    // Rotate current pixel coords around image center
                    let (rx, ry) =
                        rotate_pixel_coordinates(lx as f32, ly as f32, x_center, y_center, cos_sin[c][0], cos_sin[c][1]);
                    let dx = wrap_index(rx.round() as i32, ds);
                    let dy = wrap_index(ry.round() as i32, ds);
                    new_vec.push(if data[idx] < dot_matrices[c][dx + dy * ds] {
                        T::MIN_VALUE
                    } else {
                        T::MAX_VALUE
                    });
                }
            }
        }

        std::mem::swap(data, &mut new_vec);
        img.shape = Shape::new(scale_height, scale_width, channels_opt);
        img.resize(height, width, resize_alg, false);
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
pub fn ssaa_halftone(
    img: &mut SVec,
    dot_sizes: &[usize],
    dot_type: &[DotType],
    scale: f32,
    resize_alg: ResizeAlg,
) -> Result<(), HalftoneError> {
    match img.pixel_type() {
        PixelType::F32 => apply_ssaa_halftone::<f32>(img, dot_sizes, dot_type, scale, resize_alg),
        PixelType::U8 => apply_ssaa_halftone::<u8>(img, dot_sizes, dot_type, scale, resize_alg),
        PixelType::U16 => apply_ssaa_halftone::<u16>(img, dot_sizes, dot_type, scale, resize_alg),
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
pub fn ssaa_rotate_halftone(
    img: &mut SVec,
    dot_sizes: &[usize],
    angles: &[f32],
    dot_type: &[DotType],
    scale: f32,
    resize_alg: ResizeAlg,
) -> Result<(), HalftoneError> {
    match img.pixel_type() {
        PixelType::F32 => apply_ssaa_rotate_halftone::<f32>(img, dot_sizes, angles, dot_type, scale, resize_alg),
        PixelType::U8 => apply_ssaa_rotate_halftone::<u8>(img, dot_sizes, angles, dot_type, scale, resize_alg),
        PixelType::U16 => apply_ssaa_rotate_halftone::<u16>(img, dot_sizes, angles, dot_type, scale, resize_alg),
    }
}
