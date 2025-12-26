mod ops;
mod structure;

use crate::ops::encode::QuantizeTablePy;
use crate::structure::enums::{ColorCVT, ColorMode, DotTypePy, ImgFormat, ResizesAlg, ResizesFilter, TypeNoise};

use crate::ops::encode::JpegSamplingFactorPy;
use crate::ops::get_palette::PyPaletteAlg;
use crate::ops::lines::{PyBezier, PyBresenham, PyPoint};
use pepecore::rayon_mode;
use pyo3::prelude::*;

#[pyfunction(name = "rayon_mode")]
#[pyo3(signature = (on=true))]
pub fn rm(on: bool) {
    rayon_mode(on)
}
#[pymodule]
fn pepeline(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(ops::read_write::read, m)?)?;
    m.add_function(wrap_pyfunction!(ops::read_write::buff_read, m)?)?;
    m.add_function(wrap_pyfunction!(ops::read_write::save, m)?)?;
    m.add_function(wrap_pyfunction!(ops::colors::py_cvt_color, m)?)?;
    m.add_function(wrap_pyfunction!(ops::crop::py_crop, m)?)?;
    m.add_function(wrap_pyfunction!(ops::colors::py_color_levels, m)?)?;
    m.add_function(wrap_pyfunction!(ops::colors::py_screentone, m)?)?;
    m.add_function(wrap_pyfunction!(ops::colors::py_halftone, m)?)?;
    m.add_function(wrap_pyfunction!(ops::old_rebind::best_tile, m)?)?;
    m.add_function(wrap_pyfunction!(ops::noise::py_noise, m)?)?;
    m.add_function(wrap_pyfunction!(ops::encode::py_jpeg_encode, m)?)?;
    m.add_function(wrap_pyfunction!(ops::resize::py_resize, m)?)?;
    m.add_function(wrap_pyfunction!(rm, m)?)?;
    m.add_function(wrap_pyfunction!(ops::normalize::normalize, m)?)?;
    m.add_function(wrap_pyfunction!(ops::original_size::real_hw, m)?)?;
    m.add_function(wrap_pyfunction!(ops::original_size::real_h, m)?)?;
    m.add_function(wrap_pyfunction!(ops::original_size::real_w, m)?)?;
    m.add_function(wrap_pyfunction!(ops::lines::py_line, m)?)?;
    m.add_function(wrap_pyfunction!(ops::read_write::read_tiler, m)?)?;
    m.add_function(wrap_pyfunction!(ops::get_palette::py_palette, m)?)?;
    m.add_class::<PyPaletteAlg>()?;
    m.add_class::<PyPoint>()?;
    m.add_class::<PyBresenham>()?;
    m.add_class::<PyBezier>()?;
    m.add_class::<ColorMode>()?;
    m.add_class::<ImgFormat>()?;
    m.add_class::<ColorCVT>()?;
    m.add_class::<DotTypePy>()?;
    m.add_class::<TypeNoise>()?;
    m.add_class::<JpegSamplingFactorPy>()?;
    m.add_class::<QuantizeTablePy>()?;
    m.add_class::<ResizesFilter>()?;
    m.add_class::<ResizesAlg>()?;
    Ok(())
}
