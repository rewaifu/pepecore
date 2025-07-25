use crate::structure::enums::{ColorCVT, DotTypePy, ResizesAlg, ResizesFilter};
use crate::structure::svec_traits::{PySvec, SvecPyArray};
use pepecore::cvt_color::cvt_color;
use pepecore::enums::CVTColor;
use pepecore::{
    color_levels, halftone, rotate_halftone, rotate_screentone, screentone, ssaa_halftone, ssaa_rotate_halftone,
    ssaa_rotate_screentone, ssaa_screentone,
};
use pepecore_array::PixelType;
use pyo3::exceptions::PyValueError;
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

#[pyfunction(name = "color_levels")]
#[pyo3(signature = (img, in_low = 0, in_high =  255, out_low = 0, out_high = 255, gamma = 1.0))]
pub fn py_color_levels<'py>(
    py: Python<'py>,
    img: Bound<'py, PyAny>,
    in_low: u8,
    in_high: u8,
    out_low: u8,
    out_high: u8,
    gamma: f32,
) -> PyResult<Bound<'py, PyAny>> {
    let mut img = img.to_svec(py)?;
    match img.pixel_type() {
        PixelType::U8 => {
            img.as_f32();
            let vec = img.get_data_mut::<f32>().unwrap(); //temporary solution, since u8 and u16 give incorrect result
            color_levels::f32_color_level(
                vec,
                in_low as f32 / 255.0,
                in_high as f32 / 255.0,
                out_low as f32 / 255.0,
                out_high as f32 / 255.0,
                gamma,
            );
            img.as_u8();
            img.to_pyany::<u8>(py)
        }
        PixelType::U16 => {
            img.as_f32();
            let vec = img.get_data_mut::<f32>().unwrap(); //temporary solution, since u8 and u16 give incorrect result
            color_levels::f32_color_level(
                vec,
                in_low as f32 / 255.0,
                in_high as f32 / 255.0,
                out_low as f32 / 255.0,
                out_high as f32 / 255.0,
                gamma,
            );
            img.as_u16();
            img.to_pyany::<u16>(py)
        }
        PixelType::F32 => {
            let vec = img.get_mut_vec::<f32>().unwrap();
            color_levels::f32_color_level(
                vec,
                in_low as f32 / 255.0,
                in_high as f32 / 255.0,
                out_low as f32 / 255.0,
                out_high as f32 / 255.0,
                gamma,
            );
            img.to_pyany::<f32>(py)
        }
    }
}

#[pyfunction(name = "screentone")]
#[pyo3(signature = (img, dot_size, angle = None, dot_type = DotTypePy::CIRCLE,scale = None, resize_alg=ResizesAlg::Conv(ResizesFilter::CatmullRom)))]
pub fn py_screentone<'py>(
    py: Python<'py>,
    img: Bound<'py, PyAny>,
    dot_size: usize,
    angle: Option<f32>,
    dot_type: DotTypePy,
    scale: Option<f32>,
    resize_alg: ResizesAlg,
) -> PyResult<Bound<'py, PyAny>> {
    let mut img = img.to_svec(py)?;
    let channels = img.shape.get_channels().unwrap_or(1);
    if channels != 1 {
        return Err(PyValueError::new_err(
            "The screentone filter only accepts grayscale images (single channel).",
        ));
    }

    if let Some(angle) = angle {
        if let Some(scale) = scale {
            py.allow_threads(|| ssaa_rotate_screentone(&mut img, dot_size, angle, &dot_type.into(), scale, resize_alg.into()));
        } else {
            py.allow_threads(|| rotate_screentone(&mut img, dot_size, angle, &dot_type.into()));
        }
    } else if let Some(scale) = scale {
        py.allow_threads(|| ssaa_screentone(&mut img, dot_size, &dot_type.into(), scale, resize_alg.into()));
    } else {
        py.allow_threads(|| screentone(&mut img, dot_size, &dot_type.into()));
    }

    Ok(match img.pixel_type() {
        PixelType::U8 => img.to_pyany::<u8>(py)?,
        PixelType::F32 => img.to_pyany::<f32>(py)?,
        PixelType::U16 => img.to_pyany::<u16>(py)?,
    })
}

#[pyfunction(name = "halftone")]
#[pyo3(signature = (img, dot_sizes, angles = None, dot_types = None,scale = None, resize_alg=ResizesAlg::Conv(ResizesFilter::CatmullRom)))]
pub fn py_halftone<'py>(
    py: Python<'py>,
    img: Bound<'py, PyAny>,
    dot_sizes: Vec<usize>,
    angles: Option<Vec<f32>>,
    dot_types: Option<Vec<DotTypePy>>,
    scale: Option<f32>,
    resize_alg: ResizesAlg,
) -> PyResult<Bound<'py, PyAny>> {
    let mut img = img.to_svec(py)?;
    let dot_types = dot_types.unwrap_or_else(|| vec![DotTypePy::CIRCLE; dot_sizes.len()]);
    let dot_types: Vec<_> = dot_types.into_iter().map(|d| d.into()).collect();

    if let Some(angles) = angles {
        if let Some(scale) = scale {
            py.allow_threads(|| {
                ssaa_rotate_halftone(&mut img, &dot_sizes, &angles, &dot_types, scale, resize_alg.into()).unwrap()
            });
        } else {
            py.allow_threads(|| rotate_halftone(&mut img, &dot_sizes, &angles, &dot_types).unwrap());
        }
    } else if let Some(scale) = scale {
        py.allow_threads(|| ssaa_halftone(&mut img, &dot_sizes, &dot_types, scale, resize_alg.into()).unwrap());
    } else {
        py.allow_threads(|| halftone(&mut img, &dot_sizes, &dot_types).unwrap());
    }

    Ok(match img.pixel_type() {
        PixelType::U8 => img.to_pyany::<u8>(py)?,
        PixelType::F32 => img.to_pyany::<f32>(py)?,
        PixelType::U16 => img.to_pyany::<u16>(py)?,
    })
}
