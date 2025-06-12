use crate::structure::enums::ColorCVT;
use crate::structure::svec_traits::{PySvec, SvecPyArray};
use pepecore::cvt_color::cvt_color;
use pepecore::enums::CVTColor;
use pepecore_array::PixelType;
use pyo3::{Bound, PyAny, PyResult, Python, pyfunction};

#[pyfunction(name = "cvt_color")]
pub fn py_cvt_color<'py>(py: Python<'py>, img: Bound<'py, PyAny>, cvt_mode: ColorCVT) -> PyResult<Bound<'py, PyAny>> {
    let mut img = img.to_svec(py)?;
    py.allow_threads(|| cvt_color(&mut img, CVTColor::from(cvt_mode)));
    Ok(match img.pixel_type() {
        PixelType::U8 => img.to_pyany::<u8>(py)?,
        PixelType::F32 => img.to_pyany::<f32>(py)?,
        PixelType::U16 => img.to_pyany::<u16>(py)?,
    })
}
