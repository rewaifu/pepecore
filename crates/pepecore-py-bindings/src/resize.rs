use crate::utility::{downcast_pyany_to_svec, svec_to_pyarray};
use pepecore::enums::PixelType;
use pepecore_resize::interpolation::nearest_neighbour;
use pyo3::{Bound, IntoPyObjectExt, PyAny, PyResult, Python, pyclass, pyfunction};

#[pyclass(eq, eq_int)]
#[derive(Clone, PartialEq)]
pub enum ResizeInterpolation {
    NearestNeighbour,
}

#[pyfunction]
pub fn resize<'py>(
    py: Python<'py>,
    img: Bound<'py, PyAny>,
    height: usize,
    width: usize,
    resize_interpolation: ResizeInterpolation,
) -> PyResult<Bound<'py, PyAny>> {
    let img = downcast_pyany_to_svec(img)?;

    let dst_img = py.allow_threads(|| match resize_interpolation {
        ResizeInterpolation::NearestNeighbour => nearest_neighbour(&img, height, width).expect("Failed to resize"),
    });

    let result = match dst_img.pixel_type() {
        PixelType::F32 => svec_to_pyarray::<f32>(py, &dst_img).into_py_any(py),
        PixelType::U8 => svec_to_pyarray::<u8>(py, &dst_img).into_py_any(py),
        PixelType::U16 => svec_to_pyarray::<u16>(py, &dst_img).into_py_any(py),
    }?;

    Ok(result.into_bound(py))
}
