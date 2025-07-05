use numpy::{Element, PyArrayDyn, PyArrayMethods, PyUntypedArrayMethods};
use pepecore_array::{ImgData, SVec, Shape};
use pyo3::prelude::PyAnyMethods;
use pyo3::{Bound, IntoPyObjectExt, PyAny, PyResult, Python, exceptions::PyRuntimeError};
use rayon::prelude::*;
use std::mem::size_of;

struct UnsafeSend<T>(*const T);
unsafe impl<T> Send for UnsafeSend<T> {}

struct UnsafeMutSend<T>(*mut T);
unsafe impl<T> Send for UnsafeMutSend<T> {}

unsafe fn parallel_memcpy_typed<T: Copy + Send + Sync>(src: UnsafeSend<T>, dst: UnsafeMutSend<T>, len: usize) {
    let threads = rayon::current_num_threads();
    let chunk_size = len.div_ceil(threads);

    let src_addr = src.0 as usize;
    let dst_addr = dst.0 as usize;
    let elem_size = size_of::<T>();

    (0..threads).into_par_iter().for_each(|i| {
        let start = i * chunk_size;
        if start >= len {
            return;
        }
        let end = ((i + 1) * chunk_size).min(len);
        let count = end - start;
        let size = count * elem_size;

        let s = (src_addr + start * elem_size) as *const u8;
        let d = (dst_addr + start * elem_size) as *mut u8;

        unsafe {
            libc::memcpy(d as *mut _, s as *const _, size);
        }
    });
}

pub trait PySvec {
    fn to_svec(self, py: Python) -> PyResult<SVec>;
}

pub trait SvecPyArray {
    fn to_pyany<T>(self, py: Python) -> PyResult<Bound<PyAny>>
    where
        T: Element + Copy + 'static;
}

fn alloc_from_np<T>(py: Python, np: &Bound<PyArrayDyn<T>>) -> PyResult<SVec>
where
    T: Element + Copy + Send,
    ImgData: From<Vec<T>>,
{
    // Получаем readonly‑view
    let readonly = np.try_readonly()?;
    let shape = readonly.shape();
    let total = readonly.len();
    let mut buffer: Vec<T> = Vec::with_capacity(total);

    if readonly.is_contiguous() {
        let dst_ptr = UnsafeMutSend(buffer.as_mut_ptr());
        let src_ptr = UnsafeSend(readonly.data());
        py.allow_threads(|| unsafe {
            buffer.set_len(total);
            parallel_memcpy_typed(src_ptr, dst_ptr, total);
        });
    } else {
        buffer = readonly.to_owned_array().into_raw_vec_and_offset().0;
    }

    Ok(SVec::new(Shape::from(shape), ImgData::from(buffer)))
}

impl PySvec for Bound<'_, PyAny> {
    fn to_svec(self, py: Python) -> PyResult<SVec> {
        if let Ok(np_array) = self.downcast::<PyArrayDyn<f32>>() {
            alloc_from_np(py, np_array)
        } else if let Ok(np_array) = self.downcast::<PyArrayDyn<u8>>() {
            alloc_from_np(py, np_array)
        } else if let Ok(np_array) = self.downcast::<PyArrayDyn<u16>>() {
            alloc_from_np(py, np_array)
        } else {
            Err(PyRuntimeError::new_err("Unsupported type: Expected NumPy ndarray or list"))
        }
    }
}

impl SvecPyArray for SVec {
    fn to_pyany<T>(self, py: Python) -> PyResult<Bound<PyAny>>
    where
        T: Element + Copy + 'static + Clone,
    {
        let (height, width, channels_opt) = self.shape();
        let dims: &[usize] = match channels_opt {
            Some(c) => &[height, width, c],
            None => &[height, width],
        };

        let arr = unsafe {
            let arr = PyArrayDyn::<T>::new(py, dims, false);
            let data = UnsafeSend(
                self.get_data::<T>()
                    .expect("Type mismatch: SVec does not contain T data")
                    .as_ptr(),
            );
            // Array2::to_pyarray()
            let new_data = UnsafeMutSend(arr.data());
            let len = self.get_len();
            py.allow_threads(|| parallel_memcpy_typed(data, new_data, len));
            arr
        };

        arr.into_bound_py_any(py)
    }
}
