use crate::structure::svec_traits::{PySvec, SvecPyArray};
use pepecore::enums::YCbCrRatio;
use pepecore::jpeg_compress;
use pepecore::ops::svec_ops::jpeg::quantize::QuantizationTableType;
use pepecore_array::PixelType;
use pyo3::prelude::*;

#[pyclass(name = "JpegSamplingFactor")]
#[derive(Clone, Copy, Debug)]
#[allow(non_camel_case_types)]
pub enum JpegSamplingFactorPy {
    R444,
    R440,
    R441,
    R422,
    R420,
    R411,
    R410,
}
#[pyclass(name = "QuantizeTable")]
#[derive(Clone, Copy, Debug)]
#[allow(non_camel_case_types)]
pub enum QuantizeTablePy {
    Default,
    Flat,
    CustomMsSsim,
    CustomPsnrHvs,
    ImageMagick,
    KleinSilversteinCarney,
    DentalXRays,
    VisualDetectionModel,
    ImprovedDetectionModel,
}
impl From<QuantizeTablePy> for QuantizationTableType {
    fn from(value: QuantizeTablePy) -> Self {
        match value {
            QuantizeTablePy::Default => QuantizationTableType::Default,
            QuantizeTablePy::Flat => QuantizationTableType::Flat,
            QuantizeTablePy::CustomMsSsim => QuantizationTableType::CustomMsSsim,
            QuantizeTablePy::CustomPsnrHvs => QuantizationTableType::CustomPsnrHvs,
            QuantizeTablePy::ImageMagick => QuantizationTableType::ImageMagick,
            QuantizeTablePy::KleinSilversteinCarney => QuantizationTableType::KleinSilversteinCarney,
            QuantizeTablePy::DentalXRays => QuantizationTableType::DentalXRays,
            QuantizeTablePy::VisualDetectionModel => QuantizationTableType::VisualDetectionModel,
            QuantizeTablePy::ImprovedDetectionModel => QuantizationTableType::ImprovedDetectionModel,
        }
    }
}
impl From<JpegSamplingFactorPy> for YCbCrRatio {
    fn from(value: JpegSamplingFactorPy) -> Self {
        match value {
            JpegSamplingFactorPy::R444 => YCbCrRatio::R444,
            JpegSamplingFactorPy::R440 => YCbCrRatio::R440,
            JpegSamplingFactorPy::R441 => YCbCrRatio::R441,
            JpegSamplingFactorPy::R422 => YCbCrRatio::R422,
            JpegSamplingFactorPy::R420 => YCbCrRatio::R420,
            JpegSamplingFactorPy::R411 => YCbCrRatio::R411,
            JpegSamplingFactorPy::R410 => YCbCrRatio::R410,
        }
    }
}

#[pyfunction(name = "jpeg_encode")]
#[pyo3(signature = (img, quality = 100, qt=QuantizeTablePy::Default, sampling_factor = JpegSamplingFactorPy::R420))]
pub fn py_jpeg_encode<'py>(
    py: Python<'py>,
    img: Bound<'py, PyAny>,
    quality: u8,
    qt: QuantizeTablePy,
    sampling_factor: JpegSamplingFactorPy,
) -> PyResult<Bound<'py, PyAny>> {
    let mut img = img.to_svec(py)?;
    let pixel_type = img.pixel_type();

    py.detach(|| jpeg_compress(&mut img, quality, &qt.into(), &sampling_factor.into()));

    Ok(match pixel_type {
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
