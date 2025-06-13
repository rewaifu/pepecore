mod ops;
mod structure;

use crate::structure::enums::{ColorCVT, ColorMode, DotTypePy, ImgFormat};

use pyo3::prelude::*;

#[pymodule]
fn pepeline(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(ops::read_write::read, m)?)?;
    m.add_function(wrap_pyfunction!(ops::read_write::save, m)?)?;
    m.add_function(wrap_pyfunction!(ops::colors::py_cvt_color, m)?)?;
    m.add_function(wrap_pyfunction!(ops::crop::py_crop, m)?)?;
    m.add_function(wrap_pyfunction!(ops::colors::py_color_levels, m)?)?;
    m.add_function(wrap_pyfunction!(ops::colors::py_screentone, m)?)?;
    m.add_function(wrap_pyfunction!(ops::colors::py_halftone, m)?)?;
    m.add_class::<ColorMode>()?;
    m.add_class::<ImgFormat>()?;
    m.add_class::<ColorCVT>()?;
    m.add_class::<DotTypePy>()?;
    Ok(())
}
