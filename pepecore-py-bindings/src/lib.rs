mod ops;
mod structure;

use crate::structure::enums::{ColorCVT, ColorMode, DotTypePy, ImgFormat, TypeNoise};

use pyo3::prelude::*;
use crate::ops::encode::JpegSamplingFactorPy;

#[pymodule]
fn pepeline(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(ops::read_write::read, m)?)?;
    m.add_function(wrap_pyfunction!(ops::read_write::save, m)?)?;
    m.add_function(wrap_pyfunction!(ops::colors::py_cvt_color, m)?)?;
    m.add_function(wrap_pyfunction!(ops::crop::py_crop, m)?)?;
    m.add_function(wrap_pyfunction!(ops::colors::py_color_levels, m)?)?;
    m.add_function(wrap_pyfunction!(ops::colors::py_screentone, m)?)?;
    m.add_function(wrap_pyfunction!(ops::colors::py_halftone, m)?)?;
    m.add_function(wrap_pyfunction!(ops::old_rebind::best_tile, m)?)?;
    m.add_function(wrap_pyfunction!(ops::old_rebind::noise_generate, m)?)?;
    m.add_function(wrap_pyfunction!(ops::encode::py_jpeg_encode, m)?)?;
    m.add_class::<ColorMode>()?;
    m.add_class::<ImgFormat>()?;
    m.add_class::<ColorCVT>()?;
    m.add_class::<DotTypePy>()?;
    m.add_class::<TypeNoise>()?;
    m.add_class::<JpegSamplingFactorPy>()?;
    Ok(())
}
