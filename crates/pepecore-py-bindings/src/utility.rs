use numpy::{Element, PyArrayDyn, PyArrayMethods, PyUntypedArrayMethods};
use pepecore::array::svec::{SVec, Shape};
use pepecore::enums::ImgData;
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::PyAnyMethods;
use pyo3::{Bound, PyAny, PyResult, Python};

pub fn svec_to_pyarray<'py, T>(py: Python<'py>, data: &SVec) -> Bound<'py, PyArrayDyn<T>>
where
    T: Element + Copy + 'static,
{
    let (height, width, channels_opt) = data.shape();
    let mut dims = vec![height, width];
    if let Some(c) = channels_opt {
        dims.push(c);
    }

    let arr = unsafe { PyArrayDyn::<T>::new(py, dims, false) };

    let flat = data.get_data::<T>().expect("Type mismatch: SVec does not contain T data");

    let dst = arr.data();
    unsafe { std::ptr::copy_nonoverlapping(flat.as_ptr(), dst, flat.len()) };

    arr
}

fn alloc_from_np<T: Element>(np: &Bound<PyArrayDyn<T>>) -> PyResult<SVec>
where
    T: Copy,
    ImgData: From<Vec<T>>,
{
    let readonly = np.try_readonly()?;

    let (data, shape) = (
        readonly.as_slice()?.iter().map(|x| (*x).into()).collect::<Vec<T>>(),
        readonly.shape().to_vec(),
    );

    Ok(SVec::new(Shape::from(shape), ImgData::from(data)))
}

pub fn downcast_pyany_to_svec(data: Bound<PyAny>) -> PyResult<SVec> {
    if let Ok(np_array) = data.downcast::<PyArrayDyn<f32>>() {
        return alloc_from_np(np_array);
    } else if let Ok(np_array) = data.downcast::<PyArrayDyn<u8>>() {
        return alloc_from_np(np_array);
    } else if let Ok(np_array) = data.downcast::<PyArrayDyn<u16>>() {
        return alloc_from_np(np_array);
    }

    Err(PyRuntimeError::new_err("Unsupported type: Expected NumPy ndarray or list"))
}
