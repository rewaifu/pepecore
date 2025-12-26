use crate::structure::enums::{ColorMode, ImgFormat};
use crate::structure::svec_traits::{PySvec, SvecPyArray};
use pepecore::enums::ImgColor;
use pepecore::read::{read_in_buffer, read_in_path};
use pepecore::save::svec_save;
use pepecore_array::{ImgData, PixelType, SVec, Shape};
use pyo3::exceptions::PyRuntimeError;
use pyo3::{Bound, PyAny, PyRef, PyResult, Python, pyclass, pyfunction, pymethods};
use std::panic::{AssertUnwindSafe, catch_unwind};

// для ловли паник

#[pyfunction]
#[pyo3(signature = (path, color_mode = ColorMode::DYNAMIC, img_format =  ImgFormat::DYNAMIC))]
pub fn read(py: Python<'_>, path: String, color_mode: ColorMode, img_format: ImgFormat) -> PyResult<Bound<'_, PyAny>> {
    // ловим панику на верхнем уровне
    let result = catch_unwind(AssertUnwindSafe(|| {
        py.detach(|| match img_format {
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
        })
    }));

    match result {
        Ok(img) => Ok(match img.pixel_type() {
            PixelType::U8 => img.to_pyany::<u8>(py)?,
            PixelType::F32 => img.to_pyany::<f32>(py)?,
            PixelType::U16 => img.to_pyany::<u16>(py)?,
        }),
        Err(_) => Err(PyRuntimeError::new_err("Rust panic caught in read()")),
    }
}

#[pyclass]
pub struct Image {
    inner: SVec,
    format: ImgFormat,
    tile_size: usize,
    x_y: Vec<[usize; 2]>,
    color: bool,
    pos: usize,
}

fn get_tile<T>(data: &[T], tile_size: usize, c: usize, w: usize, y_x: [usize; 2]) -> Vec<T>
where
    T: Clone,
{
    let mut result_vec: Vec<T> = Vec::with_capacity(tile_size * tile_size * c);
    let x = y_x[1];
    let row_stride = w * c;
    for y in y_x[0]..y_x[0] + tile_size {
        let start = y * row_stride + x * c;
        let end = start + tile_size * c;
        result_vec.extend_from_slice(&data[start..end])
    }
    result_vec
}

#[pymethods]
impl Image {
    fn __iter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }
    fn __len__(&self) -> usize {
        self.x_y.len()
    }
    fn __str__(&self) -> String {
        let (h, w, c) = self.inner.shape();

        let channels = match c {
            Some(v) => v.to_string(),
            None => "None".to_string(),
        };

        format!(
            "Image(shape=({}, {}, {}), tile_size={}, tiles={}, color={}, pos={})",
            h,
            w,
            channels,
            self.tile_size,
            self.x_y.len(),
            self.color,
            self.pos,
        )
    }

    fn __repr__(&self) -> String {
        self.__str__()
    }
    #[getter]
    fn shape(&self) -> (usize, usize, Option<usize>) {
        self.inner.shape()
    }
    fn __next__<'py>(&mut self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        if self.pos >= self.x_y.len() {
            self.pos = 0;
            return Ok(None);
        }

        let (_h, w, c) = self.inner.shape();
        let svec = py.detach(|| {
            let mut svec = match self.inner.pixel_type() {
                PixelType::U8 => {
                    let tile = get_tile::<u8>(
                        self.inner.get_data::<u8>().unwrap(),
                        self.tile_size,
                        c.unwrap_or(1),
                        w,
                        self.x_y[self.pos],
                    );
                    SVec::new(Shape::new(self.tile_size, self.tile_size, c), ImgData::U8(tile))
                }
                PixelType::F32 => {
                    let tile = get_tile::<f32>(
                        self.inner.get_data::<f32>().unwrap(),
                        self.tile_size,
                        c.unwrap_or(1),
                        w,
                        self.x_y[self.pos],
                    );
                    SVec::new(Shape::new(self.tile_size, self.tile_size, c), ImgData::F32(tile))
                }
                PixelType::U16 => {
                    let tile = get_tile::<u16>(
                        self.inner.get_data::<u16>().unwrap(),
                        self.tile_size,
                        c.unwrap_or(1),
                        w,
                        self.x_y[self.pos],
                    );
                    SVec::new(Shape::new(self.tile_size, self.tile_size, c), ImgData::U16(tile))
                }
            };
            match self.format {
                ImgFormat::F32 => svec.as_f32(),
                ImgFormat::U8 => svec.as_u8(),
                ImgFormat::U16 => svec.as_u16(),
                ImgFormat::DYNAMIC => {}
            }
            svec
        });

        self.pos += 1;
        Ok(Some(match svec.pixel_type() {
            PixelType::U8 => svec.to_pyany::<u8>(py)?,
            PixelType::F32 => svec.to_pyany::<f32>(py)?,
            PixelType::U16 => svec.to_pyany::<u16>(py)?,
        }))
    }
}
#[pyfunction]
#[pyo3(signature = (path, color_mode = ColorMode::DYNAMIC, img_format =  ImgFormat::DYNAMIC,tile_size=512))]
pub fn read_tiler(
    py: Python<'_>,
    path: String,
    color_mode: ColorMode,
    img_format: ImgFormat,
    tile_size: usize,
) -> PyResult<Option<Image>> {
    let result = catch_unwind(AssertUnwindSafe(|| {
        py.detach(|| read_in_path(&*path, ImgColor::from(color_mode)).unwrap())
    }));

    match result {
        Ok(img) => {
            let (h, w, _c) = img.shape();
            let n_h = h / tile_size;
            let n_w = w / tile_size;
            if n_h == 0 || n_w == 0 {
                return Ok(None);
            }
            let mut tiles: Vec<[usize; 2]> = Vec::new();
            for y in 0..n_h {
                for x in 0..n_w {
                    tiles.push([y * tile_size, x * tile_size])
                }
            }
            Ok(Some(Image {
                inner: img,
                format: img_format,
                tile_size,
                x_y: tiles,
                color: true,
                pos: 0,
            }))
        }
        Err(e) => Err(PyRuntimeError::new_err(format!("Rust panic caught in read(){:?}", e))),
    }
}
#[pyfunction]
#[pyo3(signature = (buffer, color_mode = ColorMode::DYNAMIC, img_format =  ImgFormat::DYNAMIC))]
pub fn buff_read<'py>(
    py: Python<'py>,
    buffer: &[u8],
    color_mode: ColorMode,
    img_format: ImgFormat,
) -> PyResult<Bound<'py, PyAny>> {
    let img = py.detach(|| match img_format {
        ImgFormat::F32 => {
            let mut buff = read_in_buffer(buffer, ImgColor::from(color_mode)).unwrap();
            buff.as_f32();
            buff
        }
        ImgFormat::U8 => {
            let mut buff = read_in_buffer(buffer, ImgColor::from(color_mode)).unwrap();
            buff.as_u8();
            buff
        }
        ImgFormat::U16 => {
            let mut buff = read_in_buffer(buffer, ImgColor::from(color_mode)).unwrap();
            buff.as_u16();
            buff
        }
        ImgFormat::DYNAMIC => read_in_buffer(buffer, ImgColor::from(color_mode)).unwrap(),
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
    py.detach(|| svec_save(img, &*path).expect("Failed to save image"));
    Ok(())
}
