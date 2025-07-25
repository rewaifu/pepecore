use crate::structure::svec_traits::{PySvec, SvecPyArray};
use pepecore::ops::svec_ops::resize::fir::ResizeSVec;
use pepecore::{NormalizeSVec, rayon_mode};
use pepecore_array::PixelType;
use pyo3::{Bound, PyAny, PyResult, Python, pyfunction};

#[pyfunction(name = "rayon_mode")]
#[pyo3(signature = (img, scale = 1.0))]
pub fn normalize<'py>(py: Python<'py>, img: Bound<'py, PyAny>, scale: f32) -> PyResult<Bound<'py, PyAny>> {
    let mut img = img.to_svec(py)?;
    py.allow_threads(|| img.normalize(scale));
    Ok(match img.pixel_type() {
        PixelType::U8 => img.to_pyany::<u8>(py)?,
        PixelType::F32 => img.to_pyany::<f32>(py)?,
        PixelType::U16 => img.to_pyany::<u16>(py)?,
    })
}
