use std::fmt::Debug;
use crate::array::svec::SVec;
use crate::enums::PixelType;
use crate::errors::HalftoneError;
use crate::ops::svec_ops::halftone::dot::dot_circle;
use crate::ops::svec_ops::halftone::utils::{HalftonePixel, compute_cos_sin, rotate_pixel_coordinates};



/// Apply a standard (non-rotated) halftone to the image.
/// Returns an error if image data or channels are unavailable
/// or if `dot_sizes` does not match the number of channels.
fn apply_halftone<T>(
    img: &mut SVec,
    dot_sizes: &[usize],
) -> Result<(), HalftoneError>
where
    T: HalftonePixel + Debug,
{
    // Retrieve image shape and data buffer
    let (height, width, channels_opt) = img.shape();
    let data = img
        .get_data_mut::<T>()
        .map_err(|e| HalftoneError::GetDataError(format!("{:?}",e)))?;
    let channels = channels_opt.ok_or(HalftoneError::NoChannelsError)?;

    // Ensure that dot_sizes matches number of channels
    if dot_sizes.len() != channels {
        return Err(HalftoneError::DotSizeMismatch(
            dot_sizes.len(),
            channels,
        ));
    }

    // Prepare biases, doubled sizes, and per-channel dot matrices
    let mut biases = Vec::with_capacity(channels);
    let mut double_sizes = Vec::with_capacity(channels);
    let mut dot_matrices = Vec::with_capacity(channels);

    for &size in dot_sizes {
        let bias = size / 2;
        let doubled = size * 2;
        let matrix = if size > 0 {
            let kernel = dot_circle(size);
            let kernel_data = kernel
                .get_data::<f32>()
                .map_err(|e| HalftoneError::GetDataError(format!("{:?}",e)))?;
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
/// Additional `angles` array provides rotation per channel in degrees.
fn apply_rotate_halftone<T>(
    img: &mut SVec,
    dot_sizes: &[usize],
    angles: &[f32],
) -> Result<(), HalftoneError>
where
    T: HalftonePixel + Debug,
{
    // Retrieve image shape and data buffer
    let (height, width, channels_opt) = img.shape();
    let data = img
        .get_data_mut::<T>()
        .map_err(|e| HalftoneError::GetDataError(format!("{:?}",e)))?;
    let channels = channels_opt.ok_or(HalftoneError::NoChannelsError)?;

    // Ensure dot_sizes and angles arrays match number of channels
    if dot_sizes.len() != channels || angles.len() != channels {
        return Err(HalftoneError::DotSizeMismatch(
            dot_sizes.len().max(angles.len()),
            channels,
        ));
    }

    let x_center = width as f32 / 2.0;
    let y_center = height as f32 / 2.0;
    let mut double_sizes = Vec::with_capacity(channels);
    let mut dot_matrices = Vec::with_capacity(channels);
    let mut cos_sin = Vec::with_capacity(channels);

    // Precompute matrices and rotation sin/cos for each channel
    for i in 0..channels {
        let size = dot_sizes[i];
        println!("{}",size);
        let doubled = size * 2;
        let matrix = if size > 0 {
            let kernel = dot_circle(size);
            let kernel_data = kernel
                .get_data::<f32>()
                .map_err(|e| HalftoneError::GetDataError(format!("{:?}",e)))?;
            T::prepare_dot_matrix(kernel_data)
        } else {
            Vec::new()
        };
        double_sizes.push(doubled);
        dot_matrices.push(matrix);
        cos_sin.push( compute_cos_sin(angles[i].to_radians()));
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
                    x as f32,
                    y as f32,
                    x_center,
                    y_center,
                    cos_sin[c][0],
                    cos_sin[c][1],
                );
                let ix = (rx as usize) % ds;
                let iy = (ry as usize) % ds;
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

/// Public API: apply standard halftone
pub fn halftone(
    img: &mut SVec,
    dot_sizes: &[usize],
) -> Result<(), HalftoneError> {
    match img.pixel_type() {
        PixelType::F32 => apply_halftone::<f32>(img, dot_sizes),
        PixelType::U8 => apply_halftone::<u8>(img, dot_sizes),
        PixelType::U16 => apply_halftone::<u16>(img, dot_sizes),
    }
}

/// Public API: apply rotated halftone
pub fn rotate_halftone(
    img: &mut SVec,
    dot_sizes: &[usize],
    angles: &[f32],
) -> Result<(), HalftoneError> {
    match img.pixel_type() {
        PixelType::F32 => apply_rotate_halftone::<f32>(img, dot_sizes, angles),
        PixelType::U8 => apply_rotate_halftone::<u8>(img, dot_sizes, angles),
        PixelType::U16 => apply_rotate_halftone::<u16>(img, dot_sizes, angles),
    }
}
