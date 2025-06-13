use crate::structure::enums::{ColorMode, ImgFormat};
use crate::structure::svec_traits::{PySvec, SvecPyArray};
use pepecore::enums::ImgColor;
use pepecore::read::read_in_path;
use pepecore::save::svec_save;
use pepecore_array::PixelType;
use pyo3::{Bound, PyAny, PyResult, Python, pyfunction};

#[pyfunction]
#[pyo3(signature = (path, color_mode = ColorMode::DYNAMIC, img_format =  ImgFormat::DYNAMIC))]
pub fn read<'py>(py: Python<'py>, path: String, color_mode: ColorMode, img_format: ImgFormat) -> PyResult<Bound<'py, PyAny>> {
    let img = py.allow_threads(|| match img_format {
        ImgFormat::F32 => {
            let mut buff = read_in_path(&*path, ImgColor::from(color_mode)).unwrap();
            buff.as_f32();
            buff
        }
        ImgFormat::U8 => {
            let mut buff = read_in_path(&*path, ImgColor::from(color_mode)).unwrap();
            buff.as_u8();
            buff
        }
        ImgFormat::U16 => {
            let mut buff = read_in_path(&*path, ImgColor::from(color_mode)).unwrap();
            buff.as_u16();
            buff
        }
        ImgFormat::DYNAMIC => read_in_path(&*path, ImgColor::from(color_mode)).unwrap(),
    });

    Ok(match img.pixel_type() {
        PixelType::U8 => img.to_pyany::<u8>(py)?,
        PixelType::F32 => img.to_pyany::<f32>(py)?,
        PixelType::U16 => img.to_pyany::<u16>(py)?,
    })
}

#[pyfunction]
pub fn save<'py>(py: Python<'py>, img: Bound<'py, PyAny>, path: String) -> PyResult<()> {
    let img = img.to_svec(py)?;
    py.allow_threads(|| svec_save(img, &*path).expect("Failed to save image"));
    Ok(())
}
