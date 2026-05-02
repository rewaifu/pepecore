use crate::structure::enums::TypeNoise;
use crate::structure::svec_traits::SvecPyArray;
use fastnoise_lite::NoiseType;
// use numpy::PyArray2;
use pepecore::ops::svec_ops::noise::fast_noise_lite::{create_noise_2d, create_noise_3d};
use pepecore::ops::svec_ops::noise::fiber::{FiberParams, apply_fiber_noise};
use pepecore_array::{ImgData, SVec, Shape};
use pyo3::exceptions::PyValueError;
use pyo3::{Bound, PyAny, PyResult, Python, pyfunction};

#[pyfunction(name = "noise")]
pub fn py_noise<'py>(
    py: Python<'py>,
    shape: Vec<usize>,
    octaves: usize,
    amplitudes: Vec<f32>,
    frequency: Vec<f32>,
    noise_type: Vec<TypeNoise>,
) -> PyResult<Bound<'py, PyAny>> {
    let len_shape = shape.len();
    let noise_type: Vec<NoiseType> = noise_type.iter().map(|value| value.clone().into()).collect();
    if len_shape == 2 {
        let vec = py.detach(|| {
            create_noise_2d(
                Shape::new(shape[0], shape[1], None),
                octaves,
                &amplitudes,
                &frequency,
                &noise_type,
            )
        });
        Ok(vec.to_pyany::<f32>(py)?)
    } else if len_shape == 3 {
        let vec = py.detach(|| {
            create_noise_3d(
                Shape::new(shape[0], shape[1], Some(shape[2])),
                octaves,
                &amplitudes,
                &frequency,
                &noise_type,
            )
        });
        Ok(vec.to_pyany::<f32>(py)?)
    } else {
        return Err(PyValueError::new_err("Unsuported Shape"));
    }
}
//          fibers: 1024 * 1024,
// seed: 42,
// angle_kappa: 8.0,
// angle_mu: 0.0,
// length_range: (6, 20),
// thickness_range: (1, 3),
// paralel: false 

#[pyfunction(name = "fiber_noise")]
pub fn py_fiber_noise<'py>(
    py: Python<'py>,
    h:usize,
    w:usize,
    fibers:usize,
    angle_kappa:f64,
    angle_mu:f64,
    length_range:(i32,i32),
    thickness_range:(i32,i32),
    parallel: bool,
    seed:u64,
) -> PyResult<Bound<'py, PyAny>> {
    let vec = py.detach(|| {
        apply_fiber_noise(
            w,h,
            &FiberParams{
                fibers,
                seed,
                angle_kappa,
                angle_mu,
                length_range,
                thickness_range,
                paralel:parallel,
            }
        )
    });
    Ok(SVec::new(Shape::new(h, w, None), ImgData::F32(vec)).to_pyany::<f32>(py)?)
}
