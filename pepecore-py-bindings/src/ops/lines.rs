use crate::structure::svec_traits::SvecPyArray;
use pepecore::{Line, Point, draw_lines};
use pepecore_array::{ImgData, SVec, Shape};
use pyo3::types::{PyAnyMethods, PySequence, PyTypeMethods};
use pyo3::{Bound, PyAny, PyRef, PyResult, Python, exceptions::PyTypeError, pyclass, pyfunction, pymethods};
use std::cmp::min;
use std::collections::HashSet;
#[pyclass(name = "Point")]
#[derive(Clone, Copy)]
#[allow(non_camel_case_types)]
pub struct PyPoint {
    x: usize,
    y: usize,
    size: usize,
}

#[pyclass(name = "Bresenham")]
#[derive(Clone, Copy)]
#[allow(non_camel_case_types)]
pub struct PyBresenham {
    p0: PyPoint,
    p1: PyPoint,
}
#[pyclass(name = "Bezier")]
#[derive(Clone, Copy)]
#[allow(non_camel_case_types)]
pub struct PyBezier {
    p0: PyPoint,
    p1: PyPoint,
    p2: PyPoint,
    p3: PyPoint,
    step: f64,
}
#[pymethods]
impl PyPoint {
    #[new]
    pub fn new(x: usize, y: usize, size: usize) -> Self {
        Self { x, y, size }
    }
}
#[pymethods]
impl PyBresenham {
    #[new]
    pub fn new(p0: PyPoint, p1: PyPoint) -> Self {
        Self { p0, p1 }
    }
}
#[pymethods]
impl PyBezier {
    #[new]
    pub fn new(p0: PyPoint, p1: PyPoint, p2: PyPoint, p3: PyPoint, step: f64) -> Self {
        Self { p0, p1, p2, p3, step }
    }
}
impl From<PyPoint> for Point {
    fn from(p: PyPoint) -> Point {
        Point {
            x: p.x,
            y: p.y,
            size: p.size,
        }
    }
}
fn gather_line_cmds<'py>(lines_obj: &Bound<'py, PyAny>) -> PyResult<Vec<Line>> {
    fn one<'py>(o: &Bound<'py, PyAny>) -> PyResult<Line> {
        if let Ok(ld) = o.extract::<PyRef<'py, PyBresenham>>() {
            let l = *ld;
            return Ok(Line::Bresenham(l.p0.into(), l.p1.into()));
        }
        if let Ok(bd) = o.extract::<PyRef<'py, PyBezier>>() {
            let b = *bd;
            return Ok(Line::Bezier(b.p0.into(), b.p1.into(), b.p2.into(), b.p3.into(), b.step));
        }
        Err(PyTypeError::new_err(format!(
            "Ожидал LineDraw или BezierDraw, получил: {}",
            o.get_type().name()?
        )))
    }

    if let Ok(seq) = lines_obj.cast::<PySequence>() {
        let mut out = Vec::with_capacity(seq.len().unwrap_or(0));
        for it in seq.try_iter()? {
            let item = it?;
            out.push(one(&item)?);
        }
        Ok(out)
    } else {
        Ok(vec![one(lines_obj)?])
    }
}
fn draw(lines: &[Line], img: &mut SVec) {
    let (h, w, _) = img.shape.get_shape();
    let data = img.get_data_mut::<u8>().unwrap();
    let (h, w) = (h - 1, w - 1);
    let mut pixels: HashSet<(usize, usize)> = HashSet::new();
    draw_lines(lines, &mut pixels);
    for pixel in pixels.iter() {
        data[min(h, pixel.1) * (w + 1) + min(w, pixel.0)] = 1
    }
}
#[pyfunction(name = "line")]
pub fn py_line<'py>(py: Python<'py>, lines: Bound<'py, PyAny>, h: usize, w: usize) -> PyResult<Bound<'py, PyAny>> {
    let lines = gather_line_cmds(&lines)?;
    let mut img = SVec::new(Shape::new(h, w, None), ImgData::U8(vec![0; h * w]));
    py.detach(|| draw(&lines, &mut img));
    Ok(img.to_pyany::<u8>(py)?)
}
