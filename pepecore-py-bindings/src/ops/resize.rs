use crate::structure::enums::{ResizesAlg, ResizesFilter};
use crate::structure::svec_traits::{PySvec, SvecPyArray};
use pepecore::ops::svec_ops::resize::fir::ResizeSVec;
use pepecore_array::PixelType;
use pyo3::{Bound, PyAny, PyResult, Python, pyfunction};

#[pyfunction(name = "resize")]
#[pyo3(signature = (img, h,w, resize_alg=ResizesAlg::Conv(ResizesFilter::CatmullRom),alpha=true))]
pub fn py_resize<'py>(
    py: Python<'py>,
    img: Bound<'py, PyAny>,
    h: usize,
    w: usize,
    resize_alg: ResizesAlg,
    alpha: bool,
) -> PyResult<Bound<'py, PyAny>> {
    let mut img = img.to_svec(py)?;

    py.allow_threads(|| img.resize(h, w, resize_alg.into(), alpha));
    Ok(match img.pixel_type() {
        PixelType::U8 => img.to_pyany::<u8>(py)?,
        PixelType::F32 => img.to_pyany::<f32>(py)?,
        PixelType::U16 => img.to_pyany::<u16>(py)?,
    })
}
