mod pyimage;
mod read;
mod resize;
mod save;
mod utility;

use pyo3::prelude::*;

/// A Python module implemented in Rust. The name of this function must match
/// the `lib.name` setting in the `Cargo.toml`, else Python will not be able to
/// import the module.
#[pymodule]
fn pepecore(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.setattr("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add_class::<read::ColorMode>()?;
    m.add_class::<resize::ResizeInterpolation>()?;
    m.add_function(wrap_pyfunction!(read::read, m)?)?;
    m.add_function(wrap_pyfunction!(save::save, m)?)?;
    m.add_function(wrap_pyfunction!(resize::resize, m)?)?;
    Ok(())
}
