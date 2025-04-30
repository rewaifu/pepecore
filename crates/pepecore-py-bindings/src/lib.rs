#[cfg(feature = "resize")]
mod resize;
mod utility;
mod rw;

use pyo3::prelude::*;

/// A Python module implemented in Rust. The name of this function must match
/// the `lib.name` setting in the `Cargo.toml`, else Python will not be able to
/// import the module.
#[pymodule]
fn pepecore(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.setattr("__version__", env!("CARGO_PKG_VERSION"))?;
    rw::rw_module(&m)?;
    #[cfg(feature = "resize")]
    resize::resize_module(&m)?;
    Ok(())
}
