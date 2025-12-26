use crate::structure::enums::TypeNoise;
use crate::structure::svec_traits::SvecPyArray;
use fastnoise_lite::NoiseType;
use pepecore::ops::svec_ops::noise::fast_noise_lite::{create_noise_2d, create_noise_3d};
use pepecore_array::Shape;
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
