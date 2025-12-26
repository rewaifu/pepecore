use crate::structure::svec_traits::PySvec;
use numpy::ndarray::Array3;
use numpy::{IntoPyArray, PyArray3};
use pepecore::enums::PaletteAlg;
use pepecore::ops::svec_ops::palette_gen::gen_palette::svec_to_palette;
use pyo3::{Bound, PyAny, PyResult, Python, pyclass, pyfunction};
#[pyclass(name = "PaletteAlg")]
#[derive(Clone, Copy, Debug)]
pub enum PyPaletteAlg {
    OcTree,
    MedianCut,
    Wu,
    MinMaxUniform,
}
impl From<PyPaletteAlg> for PaletteAlg {
    fn from(value: PyPaletteAlg) -> Self {
        match value {
            PyPaletteAlg::OcTree => PaletteAlg::OcTree,
            PyPaletteAlg::MedianCut => PaletteAlg::MedianCut,
            PyPaletteAlg::Wu => PaletteAlg::Wu,
            PyPaletteAlg::MinMaxUniform => PaletteAlg::MinMaxUniform,
        }
    }
}
#[pyfunction(name = "get_palette")]
#[pyo3(signature = (img, num_ch, alg=PyPaletteAlg::OcTree))]
pub fn py_palette<'py>(
    py: Python<'py>,
    img: Bound<'py, PyAny>,
    num_ch: usize,
    alg: PyPaletteAlg,
) -> PyResult<Bound<'py, PyArray3<f32>>> {
    let mut img = img.to_svec(py).unwrap();
    let result = py.detach(|| svec_to_palette(&mut img, num_ch, alg.into()));
    let array = Array3::from_shape_vec([1, result.len() / 3, 3], result).unwrap();
    Ok(array.into_pyarray(py))
}
