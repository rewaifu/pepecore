mod ops;
mod structure;

use crate::structure::enums::{ColorCVT, ColorMode, ImgFormat};

use pyo3::prelude::*;

#[pymodule]
fn pepeline(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(ops::read_write::read, m)?)?;
    m.add_function(wrap_pyfunction!(ops::read_write::save, m)?)?;
    m.add_function(wrap_pyfunction!(ops::colors::py_cvt_color, m)?)?;
    m.add_class::<ColorMode>()?;
    m.add_class::<ImgFormat>()?;
    m.add_class::<ColorCVT>()?;
    Ok(())
}
