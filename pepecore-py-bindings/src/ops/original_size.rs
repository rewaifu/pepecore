use crate::structure::svec_traits::PySvec;
use pepecore::real_size::{get_full_original_size, get_original_height_only, get_original_width_only};
use pyo3::{Bound, PyAny, PyResult, Python, pyfunction};

#[pyfunction(name = "real_hw")]
#[pyo3(signature = (img))]
pub fn real_hw<'py>(py: Python<'_>, img: Bound<'py, PyAny>) -> PyResult<(usize, usize)> {
    let img = img.to_svec(py).unwrap();
    let (h, w) = py.detach(|| get_full_original_size(&img));
    Ok((h, w))
}
#[pyfunction(name = "real_h")]
#[pyo3(signature = (img))]
pub fn real_h<'py>(py: Python<'_>, img: Bound<'py, PyAny>) -> PyResult<usize> {
    let img = img.to_svec(py).unwrap();
    let h = py.detach(|| get_original_height_only(&img));
    Ok(h)
}
#[pyfunction(name = "real_w")]
#[pyo3(signature = (img))]
pub fn real_w<'py>(py: Python<'_>, img: Bound<'py, PyAny>) -> PyResult<usize> {
    let img = img.to_svec(py).unwrap();
    let w = py.detach(|| get_original_width_only(&img));
    Ok(w)
}
