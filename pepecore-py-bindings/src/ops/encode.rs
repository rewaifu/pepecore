use crate::structure::svec_traits::{PySvec, SvecPyArray};
use pepecore::encode::{Encoder, JpegEncodeOptions, JpegSamplingFactor};
use pepecore_array::PixelType;
use pyo3::prelude::*;

#[pyclass(name = "JpegSamplingFactor")]
#[derive(Clone, Copy, Debug)]
pub enum JpegSamplingFactorPy {
    R_4_4_4,
    R_4_4_0,
    R_4_4_1,
    R_4_2_2,
    R_4_2_1,
    R_4_2_0,
    R_4_1_1,
    R_4_1_0,
}

impl From<JpegSamplingFactorPy> for JpegSamplingFactor {
    fn from(value: JpegSamplingFactorPy) -> Self {
        match value {
            JpegSamplingFactorPy::R_4_4_4 => JpegSamplingFactor::R_4_4_4,
            JpegSamplingFactorPy::R_4_4_0 => JpegSamplingFactor::R_4_4_0,
            JpegSamplingFactorPy::R_4_4_1 => JpegSamplingFactor::R_4_4_1,
            JpegSamplingFactorPy::R_4_2_2 => JpegSamplingFactor::R_4_2_2,
            JpegSamplingFactorPy::R_4_2_1 => JpegSamplingFactor::R_4_2_1,
            JpegSamplingFactorPy::R_4_2_0 => JpegSamplingFactor::R_4_2_0,
            JpegSamplingFactorPy::R_4_1_1 => JpegSamplingFactor::R_4_1_1,
            JpegSamplingFactorPy::R_4_1_0 => JpegSamplingFactor::R_4_1_0,
        }
    }
}

#[pyfunction(name = "jpeg_encode")]
#[pyo3(signature = (img, quality = 100, progressive =  true, sampling_factor = JpegSamplingFactorPy::R_4_2_0))]
pub fn py_jpeg_encode<'py>(
    py: Python<'py>,
    img: Bound<'py, PyAny>,
    quality: u8,
    progressive: bool,
    sampling_factor: JpegSamplingFactorPy,
) -> PyResult<Bound<'py, PyAny>> {
    let mut img = img.to_svec(py)?;

    let options = JpegEncodeOptions {
        quality,
        progressive,
        sampling_factor: sampling_factor.into(),
    };

    let mut img = py
        .allow_threads(|| Encoder::encode_jpeg(&mut img, options))
        .expect("Failed to save image");

    Ok(match img.pixel_type() {
        PixelType::U8 => img.to_pyany::<u8>(py)?,
        PixelType::F32 => {
            img.as_f32();
            img.to_pyany::<f32>(py)?
        }
        PixelType::U16 => {
            img.as_u16();
            img.to_pyany::<u16>(py)?
        }
    })
}
