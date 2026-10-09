use crate::structure::enums::{ResizesAlg, ResizesFilter};
use crate::structure::svec_traits::PySvec;
use pepecore::{OverlayAnchor, PsdComposeOptions, PsdCompression, compose_to_psd};
use pyo3::exceptions::PyValueError;
use pyo3::{Bound, PyAny, PyResult, Python, pyclass, pyfunction};

#[pyclass(name = "OverlayAnchor", eq, eq_int)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum OverlayAnchorPy {
    TopLeft,
    TopCenter,
    TopRight,
    CenterLeft,
    Center,
    CenterRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

impl From<OverlayAnchorPy> for OverlayAnchor {
    fn from(value: OverlayAnchorPy) -> Self {
        match value {
            OverlayAnchorPy::TopLeft => OverlayAnchor::TopLeft,
            OverlayAnchorPy::TopCenter => OverlayAnchor::TopCenter,
            OverlayAnchorPy::TopRight => OverlayAnchor::TopRight,
            OverlayAnchorPy::CenterLeft => OverlayAnchor::CenterLeft,
            OverlayAnchorPy::Center => OverlayAnchor::Center,
            OverlayAnchorPy::CenterRight => OverlayAnchor::CenterRight,
            OverlayAnchorPy::BottomLeft => OverlayAnchor::BottomLeft,
            OverlayAnchorPy::BottomCenter => OverlayAnchor::BottomCenter,
            OverlayAnchorPy::BottomRight => OverlayAnchor::BottomRight,
        }
    }
}

#[pyclass(name = "OverlayFit", eq, eq_int)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum OverlayFitPy {
    Stretch,
    Fit,
    Cover,
    Original,
}

#[pyclass(name = "PsdCompression", eq, eq_int)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PsdCompressionPy {
    Raw,
    Rle,
    Zip,
    ZipPrediction,
}

impl From<PsdCompressionPy> for PsdCompression {
    fn from(value: PsdCompressionPy) -> Self {
        match value {
            PsdCompressionPy::Raw => PsdCompression::Raw,
            PsdCompressionPy::Rle => PsdCompression::Rle,
            PsdCompressionPy::Zip => PsdCompression::Zip,
            PsdCompressionPy::ZipPrediction => PsdCompression::ZipPrediction,
        }
    }
}

/// Compose two images and save a layered PSD.
///
/// `background` defines canvas size and bit depth; `foreground` is placed on
/// top per `fit` box (`fit_width × fit_height`) / `anchor` / offsets.
/// `fit_width`/`fit_height` are ignored for `OverlayFit.Original`.
/// Returns drawn overlay pixels.
#[pyfunction(name = "compose_psd")]
#[pyo3(signature = (background, foreground, path, fit=OverlayFitPy::Original, anchor=OverlayAnchorPy::Center, fit_width=0, fit_height=0, offset_x=0, offset_y=0, resize_alg=ResizesAlg::Conv(ResizesFilter::CatmullRom), compression=PsdCompressionPy::Rle))]
#[allow(clippy::too_many_arguments)]
pub fn py_compose_psd<'py>(
    py: Python<'py>,
    background: Bound<'py, PyAny>,
    foreground: Bound<'py, PyAny>,
    path: String,
    fit: OverlayFitPy,
    anchor: OverlayAnchorPy,
    fit_width: usize,
    fit_height: usize,
    offset_x: i32,
    offset_y: i32,
    resize_alg: ResizesAlg,
    compression: PsdCompressionPy,
) -> PyResult<usize> {
    let bg = background.to_svec(py)?;
    let fg = foreground.to_svec(py)?;
    let fit = match fit {
        OverlayFitPy::Stretch => pepecore::OverlayFit::Stretch { width: fit_width, height: fit_height },
        OverlayFitPy::Fit => pepecore::OverlayFit::Fit { width: fit_width, height: fit_height },
        OverlayFitPy::Cover => pepecore::OverlayFit::Cover { width: fit_width, height: fit_height },
        OverlayFitPy::Original => pepecore::OverlayFit::Original,
    };
    let opts = PsdComposeOptions {
        fit,
        offset: (offset_x, offset_y),
        anchor: anchor.into(),
        resize_alg: resize_alg.into(),
        compression: compression.into(),
        ..PsdComposeOptions::default()
    };
    py.detach(|| compose_to_psd(bg, fg, &opts, &*path))
        .map_err(|e| PyValueError::new_err(format!("compose_psd failed: {:?}", e)))
}
