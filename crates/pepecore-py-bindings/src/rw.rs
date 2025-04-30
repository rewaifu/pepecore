use std::path::PathBuf;
use pyo3::{pyclass, pyfunction, pymodule, wrap_pyfunction, Bound, IntoPyObjectExt, PyAny, PyResult, Python};
use pyo3::prelude::{PyModule, PyModuleMethods};
use pepecore::enums::{ImgColor, PixelType};
use pepecore::ops::read::read::read_in_path;
use pepecore::ops::save::save::svec_save;
use crate::utility::{downcast_pyany_to_svec, svec_to_pyarray};

#[pyclass]
#[derive(Clone, Copy)]
pub enum ColorMode {
    GRAY,
    RGB,
    RGBA,
    GRAYA,
    DYNAMIC,
}

impl From<ColorMode> for ImgColor {
    fn from(value: ColorMode) -> Self {
        match value {
            ColorMode::GRAY => ImgColor::GRAY,
            ColorMode::RGB => ImgColor::RGB,
            ColorMode::RGBA => ImgColor::RGBA,
            ColorMode::GRAYA => ImgColor::GRAYA,
            ColorMode::DYNAMIC => ImgColor::DYNAMIC,
        }
    }
}

#[pyfunction]
#[pyo3(signature = (path, color_mode = ColorMode::DYNAMIC))]
pub fn read<'py>(py: Python<'py>, path: String, color_mode: ColorMode) -> PyResult<Bound<'py, PyAny>> {
    let path = PathBuf::from(path);

    let img = read_in_path(&path, ImgColor::from(color_mode)).expect("Failed to read image");

    let result = match img.pixel_type() {
        PixelType::F32 => svec_to_pyarray::<f32>(py, &img).into_py_any(py),
        PixelType::U8 => svec_to_pyarray::<u8>(py, &img).into_py_any(py),
        PixelType::U16 => svec_to_pyarray::<u16>(py, &img).into_py_any(py),
    }?;

    Ok(result.into_bound(py))
}

#[pyfunction]
pub fn save<'py>(py: Python<'py>, img: Bound<'py, PyAny>, path: String) -> PyResult<()> {
    let img = downcast_pyany_to_svec(img)?;

    let path = PathBuf::from(path);

    py.allow_threads(|| svec_save(img, &path).expect("Failed to save image"));

    Ok(())
}


#[pymodule]
pub fn rw_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let module = PyModule::new(m.py(), "rw")?;
    module.add_class::<ColorMode>()?;
    module.add_function(wrap_pyfunction!(read, m)?)?;
    module.add_function(wrap_pyfunction!(save, m)?)?;
    m.add_submodule(&module)?;
    Ok(())
}