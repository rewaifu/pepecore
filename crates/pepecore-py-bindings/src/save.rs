use crate::utility::downcast_pyany_to_svec;
use pepecore::ops::save::save::svec_save;
use pyo3::{PyAny, PyResult, Python, pyfunction};
use std::path::PathBuf;

#[pyfunction]
pub fn save<'py>(py: Python<'py>, img: pyo3::Bound<'py, PyAny>, path: String) -> PyResult<()> {
    let img = downcast_pyany_to_svec(img)?;

    let path = PathBuf::from(path);

    py.allow_threads(|| svec_save(img, &path).expect("Failed to save image"));

    Ok(())
}
