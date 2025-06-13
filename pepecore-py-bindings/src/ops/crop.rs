use pyo3::{pyfunction, Bound, PyAny, PyResult, Python};
use crate::structure::svec_traits::{PySvec, SvecPyArray};
use pepecore::crop;
use pepecore_array::PixelType;

#[pyfunction(name = "crop")]
pub fn py_crop<'py>(
    py: Python<'py>,
    img: Bound<'py, PyAny>,
    x: usize,
    y: usize,
    w: usize,
    h: usize,
) -> PyResult<Bound<'py, PyAny>> {
    let mut img = img.to_svec(py)?;
    py.allow_threads(|| crop(&mut img, x, y, w, h).unwrap());
    Ok(match img.pixel_type() {
        PixelType::U8 => img.to_pyany::<u8>(py)?,
        PixelType::F32 => img.to_pyany::<f32>(py)?,
        PixelType::U16 => img.to_pyany::<u16>(py)?,
    })
}