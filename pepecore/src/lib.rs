//! Top-level library module re-exporting core functionality for image decoding, encoding, and processing.
//!
//! This crate provides:
//! - **Array and SVec** structures for image data representation (`array::svec`).
//! - **Image decoding** from file paths or byte buffers (`read::read_in_path`).
//! - **Image saving** to various formats (`save::save`).
//! - **Color conversions** (grayscale, YCbCr, CMYK, channel swaps) via `cvt_color`.
//! - **Halftone effects** (`halftone`, `rotate_halftone`).
//! - **Screentone effects** (`screentone`, `rotate_screentone`).
//!
//! # Usage Example
//!
//! ```rust
//! use pepecore_array::SVec;
//! use pepecore::{
//!     read,
//!     save,
//!     cvt_color,
//!     halftone,
//!     screentone,
//!     color_levels,
//!     enums::{ImgColor, CVTColor}
//! };
//! use pepecore::cvt_color::cvt_color;
//! use pepecore::enums::DotType;
//! use pepecore::read::read_in_path;
//!
//! // Decode an image file as RGB:
//! let mut img: SVec = read_in_path("input.png", ImgColor::RGB).unwrap();
//!
//! // Convert RGB to grayscale using BT.709:
//! cvt_color(&mut img, CVTColor::RGB2Gray_709);
//!
//! // Apply a halftone effect:
//! screentone(&mut img, 5, &DotType::CIRCLE);
//!
//!
//! // Save result as PNG:
//! save::svec_save(img, "output.png").unwrap();
//! ```

pub use pepecore_array as array;

pub mod enums;
pub mod errors;
mod global_params;
pub mod ops;

// Re-export common types and functions
pub use ops::read::read;
pub use ops::save::save;

pub use global_params::rayon_mode;
#[cfg(feature = "encode")]
pub use ops::encode;
pub use ops::svec_ops::color_levels;
pub use ops::svec_ops::crop::crop;
pub use ops::svec_ops::cvtcolor::cvt_color;
pub use ops::svec_ops::halftone::halftone::{halftone, rotate_halftone, ssaa_halftone, ssaa_rotate_halftone};
pub use ops::svec_ops::halftone::screentone::{rotate_screentone, screentone, ssaa_rotate_screentone, ssaa_screentone};
pub use ops::svec_ops::normalize::NormalizeSVec;
