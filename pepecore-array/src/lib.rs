pub mod error;
#[cfg(feature = "type-convert")]
pub mod type_convert;
#[cfg(not(feature = "type-convert"))]
mod type_convert;

use crate::error::Error;
use crate::type_convert::f32_to::{convert_f32_to_u8_normalized, convert_f32_to_u16_normalized};
use crate::type_convert::u8_to::{convert_u8_to_f32_normalized, convert_u8_to_u16_normalized};
use crate::type_convert::u16_to::{convert_u16_to_f32_normalized, convert_u16_to_u8_normalized};
use realfft::{RealFftPlanner, RealToComplex};
use rustfft::num_complex::Complex;
use std::any::TypeId;
use std::f32::consts::{FRAC_1_SQRT_2, PI};
use std::fmt;
use std::ops::Range;

#[inline]
fn precompute_twiddle_alpha(n: usize) -> (Vec<Complex<f32>>, Vec<f32>) {
    let base = 0.5 * (2.0f32 / n as f32).sqrt();
    let mut tw = Vec::with_capacity(n);
    let mut alpha = Vec::with_capacity(n);
    for k in 0..n {
        let theta = PI * (k as f32) / (2.0 * n as f32);
        let (s, c) = theta.sin_cos();
        tw.push(Complex { re: c, im: -s }); // exp(-jθ)
        alpha.push(if k == 0 { base * FRAC_1_SQRT_2 } else { base }); // 0.5*sqrt(2/N)*(k==0?1/√2:1)
    }
    (tw, alpha)
}

#[inline(always)]
fn dct1d_ortho_realfft_into(
    input: &[f32],
    output: &mut [f32],
    r2c: &std::sync::Arc<dyn RealToComplex<f32>>,
    vbuf: &mut [f32],
    spec: &mut [Complex<f32>],
    tw: &[Complex<f32>],
    alpha: &[f32],
) {
    let n = input.len();
    debug_assert_eq!(output.len(), n);
    debug_assert_eq!(vbuf.len(), 2 * n);
    debug_assert_eq!(spec.len(), n + 1);

    // even-reflect до 2N
    for i in 0..n {
        let v = unsafe { *input.get_unchecked(i) };
        unsafe {
            *vbuf.get_unchecked_mut(i) = v;
            *vbuf.get_unchecked_mut(2 * n - 1 - i) = v;
        }
    }

    // Real FFT (2N) → spec
    r2c.process(vbuf, spec).unwrap();

    // DCT-II: X[k] = Re{ V[k]*exp(-jπk/2N) } * 0.5 * sqrt(2/N), k=0: *1/√2
    for k in 0..n {
        unsafe {
            let z = *spec.get_unchecked(k);
            let t = *tw.get_unchecked(k);
            let re = z.re * t.re - z.im * t.im;
            *output.get_unchecked_mut(k) = re * *alpha.get_unchecked(k);
        }
    }
}

#[derive(Clone, Debug)]
pub struct Shape {
    height: usize,
    width: usize,
    channels: Option<usize>,
}

impl From<Vec<usize>> for Shape {
    fn from(vec: Vec<usize>) -> Self {
        Self {
            height: vec[0],
            width: vec[1],
            channels: vec.get(2).cloned(),
        }
    }
}

impl From<&[usize]> for Shape {
    fn from(value: &[usize]) -> Self {
        Self {
            height: value[0],
            width: value[1],
            channels: value.get(2).cloned(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PixelType {
    U8,
    U16,
    F32,
}

#[derive(Clone, Debug)]
pub enum ImgData {
    F32(Vec<f32>),
    U8(Vec<u8>),
    U16(Vec<u16>),
}

impl From<Vec<f32>> for ImgData {
    fn from(value: Vec<f32>) -> Self {
        ImgData::F32(value)
    }
}

impl From<Vec<u8>> for ImgData {
    fn from(value: Vec<u8>) -> Self {
        ImgData::U8(value)
    }
}

impl From<Vec<u16>> for ImgData {
    fn from(value: Vec<u16>) -> Self {
        ImgData::U16(value)
    }
}

impl ImgData {
    pub fn pixel_type(&self) -> PixelType {
        match self {
            ImgData::U8(_) => PixelType::U8,
            ImgData::U16(_) => PixelType::U16,
            ImgData::F32(_) => PixelType::F32,
        }
    }
}

#[derive(Clone)]
pub struct SVec {
    pub shape: Shape,
    pub data: ImgData,
}

impl Shape {
    pub fn new(height: usize, width: usize, channels: Option<usize>) -> Self {
        Self { height, width, channels }
    }
    pub fn get_height(&self) -> usize {
        self.height
    }
    pub fn get_width(&self) -> usize {
        self.width
    }
    pub fn get_channels(&self) -> Option<usize> {
        self.channels
    }
    pub fn get_shape(&self) -> (usize, usize, Option<usize>) {
        (self.height, self.width, self.channels)
    }
    pub fn get_ndims(&self) -> usize {
        if self.channels.is_some() { 3 } else { 2 }
    }
}
impl SVec {
    pub fn new(shape: Shape, data: ImgData) -> Self {
        SVec { shape, data }
    }
    pub fn shape(&self) -> (usize, usize, Option<usize>) {
        self.shape.get_shape()
    }
    pub fn get_len(&self) -> usize {
        let shape = self.shape.get_shape();
        shape.0 * shape.1 * shape.2.unwrap_or(1)
    }

    pub fn get_data<T: 'static>(&self) -> Result<&[T], Error> {
        match &self.data {
            ImgData::U8(data) => {
                if TypeId::of::<T>() == TypeId::of::<u8>() {
                    Ok(unsafe { std::mem::transmute::<&[u8], &[T]>(data) })
                } else {
                    Err(Error::TypeMismatch {
                        expected: "u8",
                        actual: std::any::type_name::<T>(),
                    })
                }
            }
            ImgData::U16(data) => {
                if TypeId::of::<T>() == TypeId::of::<u16>() {
                    Ok(unsafe { std::mem::transmute::<&[u16], &[T]>(data) })
                } else {
                    Err(Error::TypeMismatch {
                        expected: "u16",
                        actual: std::any::type_name::<T>(),
                    })
                }
            }
            ImgData::F32(data) => {
                if TypeId::of::<T>() == TypeId::of::<f32>() {
                    Ok(unsafe { std::mem::transmute::<&[f32], &[T]>(data) })
                } else {
                    Err(Error::TypeMismatch {
                        expected: "f32",
                        actual: std::any::type_name::<T>(),
                    })
                }
            }
        }
    }
    pub fn pixel_type(&self) -> PixelType {
        self.data.pixel_type()
    }
    pub fn get_data_mut<T: 'static>(&mut self) -> Result<&mut [T], Error> {
        match &mut self.data {
            ImgData::U8(data) => {
                if TypeId::of::<T>() == TypeId::of::<u8>() {
                    Ok(unsafe { std::mem::transmute::<&mut [u8], &mut [T]>(data) })
                } else {
                    Err(Error::TypeMismatch {
                        expected: "u8",
                        actual: std::any::type_name::<T>(),
                    })
                }
            }
            ImgData::U16(data) => {
                if TypeId::of::<T>() == TypeId::of::<u16>() {
                    Ok(unsafe { std::mem::transmute::<&mut [u16], &mut [T]>(data) })
                } else {
                    Err(Error::TypeMismatch {
                        expected: "u16",
                        actual: std::any::type_name::<T>(),
                    })
                }
            }
            ImgData::F32(data) => {
                if TypeId::of::<T>() == TypeId::of::<f32>() {
                    Ok(unsafe { std::mem::transmute::<&mut [f32], &mut [T]>(data) })
                } else {
                    Err(Error::TypeMismatch {
                        expected: "f32",
                        actual: std::any::type_name::<T>(),
                    })
                }
            }
        }
    }
    pub fn as_f32(&mut self) {
        match &mut self.data {
            ImgData::U8(_) => convert_u8_to_f32_normalized(self),
            ImgData::U16(_) => convert_u16_to_f32_normalized(self),
            ImgData::F32(_) => {}
        }
    }
    pub fn as_u8(&mut self) {
        match &mut self.data {
            ImgData::U8(_) => {}
            ImgData::U16(_) => convert_u16_to_u8_normalized(self),
            ImgData::F32(_) => convert_f32_to_u8_normalized(self),
        }
    }
    pub fn as_u16(&mut self) {
        match &mut self.data {
            ImgData::U8(_) => convert_u8_to_u16_normalized(self),
            ImgData::U16(_) => {}
            ImgData::F32(_) => convert_f32_to_u16_normalized(self),
        }
    }
    pub fn dct2(&mut self) -> Result<(), Error> {
        let (h, w, c_opt) = self.shape();
        let c = c_opt.unwrap_or(1);

        self.as_f32();
        let buf = self.get_data_mut::<f32>()?;

        // планы real-FFT
        let mut planner = RealFftPlanner::<f32>::new();
        let r2c_row = planner.plan_fft_forward(2 * w);
        let r2c_col = planner.plan_fft_forward(2 * h);

        // рабочие буферы (reuse)
        let mut v_row = r2c_row.make_input_vec(); // 2W
        let mut v_col = r2c_col.make_input_vec(); // 2H
        let mut spec_row = r2c_row.make_output_vec(); // W+1
        let mut spec_col = r2c_col.make_output_vec(); // H+1

        let (tw_row, alpha_row) = precompute_twiddle_alpha(w);
        let (tw_col, alpha_col) = precompute_twiddle_alpha(h);

        // РАЗДЕЛЬНЫЕ буферы ввода/вывода!
        let mut row_in = vec![0.0f32; w];
        let mut row_out = vec![0.0f32; w];
        let mut col_in = vec![0.0f32; h];
        let mut col_out = vec![0.0f32; h];

        // --- DCT по строкам ---
        for ch in 0..c {
            for y in 0..h {
                // gather строки
                let base = y * w * c + ch;
                let mut idx = base;
                for x in 0..w {
                    unsafe {
                        *row_in.get_unchecked_mut(x) = *buf.get_unchecked(idx);
                    }
                    idx += c;
                }

                dct1d_ortho_realfft_into(
                    &row_in,
                    &mut row_out,
                    &r2c_row,
                    &mut v_row,
                    &mut spec_row,
                    &tw_row,
                    &alpha_row,
                );

                // scatter обратно
                let mut idx = base;
                for x in 0..w {
                    unsafe {
                        *buf.get_unchecked_mut(idx) = *row_out.get_unchecked(x);
                    }
                    idx += c;
                }
            }
        }

        for ch in 0..c {
            for x in 0..w {
                let mut idx = x * c + ch;
                for y in 0..h {
                    unsafe {
                        *col_in.get_unchecked_mut(y) = *buf.get_unchecked(idx);
                    }
                    idx += w * c;
                }

                dct1d_ortho_realfft_into(
                    &col_in,
                    &mut col_out,
                    &r2c_col,
                    &mut v_col,
                    &mut spec_col,
                    &tw_col,
                    &alpha_col,
                );

                let mut idx = x * c + ch;
                for y in 0..h {
                    unsafe {
                        *buf.get_unchecked_mut(idx) = (*col_out.get_unchecked(y)).abs();
                    }
                    idx += w * c;
                }
            }
        }
        Ok(())
    }

    pub fn truncate(&mut self, new_len: usize) -> Result<(), Error> {
        match &mut self.data {
            ImgData::U8(data) => {
                data.truncate(new_len);
                Ok(())
            }
            ImgData::U16(data) => {
                data.truncate(new_len);
                Ok(())
            }
            ImgData::F32(data) => {
                data.truncate(new_len);
                Ok(())
            }
        }
    }
    pub fn drain(&mut self, new_len: Range<usize>) -> Result<(), Error> {
        match &mut self.data {
            ImgData::U8(data) => {
                data.drain(new_len);
                Ok(())
            }
            ImgData::U16(data) => {
                data.drain(new_len);
                Ok(())
            }
            ImgData::F32(data) => {
                data.drain(new_len);
                Ok(())
            }
        }
    }
    pub fn get_mut_vec<T: 'static>(&mut self) -> Result<&mut Vec<T>, Error> {
        match &mut self.data {
            ImgData::U8(data) => {
                if TypeId::of::<T>() == TypeId::of::<u8>() {
                    Ok(unsafe { std::mem::transmute::<&mut Vec<u8>, &mut Vec<T>>(data) })
                } else {
                    Err(Error::TypeMismatch {
                        expected: "u8",
                        actual: std::any::type_name::<T>(),
                    })
                }
            }
            ImgData::U16(data) => {
                if TypeId::of::<T>() == TypeId::of::<u16>() {
                    Ok(unsafe { std::mem::transmute::<&mut Vec<u16>, &mut Vec<T>>(data) })
                } else {
                    Err(Error::TypeMismatch {
                        expected: "u16",
                        actual: std::any::type_name::<T>(),
                    })
                }
            }
            ImgData::F32(data) => {
                if TypeId::of::<T>() == TypeId::of::<f32>() {
                    Ok(unsafe { std::mem::transmute::<&mut Vec<f32>, &mut Vec<T>>(data) })
                } else {
                    Err(Error::TypeMismatch {
                        expected: "f32",
                        actual: std::any::type_name::<T>(),
                    })
                }
            }
        }
    }
    pub fn get_mut_ptr<T: 'static>(&mut self) -> Result<*mut T, Error> {
        self.get_data_mut::<T>().map(|slice| slice.as_mut_ptr())
    }
}
impl fmt::Debug for SVec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (w, h, c_opt) = self.shape();
        let c = c_opt.unwrap_or(1);

        writeln!(f, "SVec (shape: {}x{}x{}):", w, h, c)?;

        match &self.data {
            ImgData::U8(data) => fmt_data(f, data, w, h, c),
            ImgData::U16(data) => fmt_data(f, data, w, h, c),
            ImgData::F32(data) => fmt_data(f, data, w, h, c),
        }
    }
}

fn fmt_data<T: fmt::Debug>(f: &mut fmt::Formatter<'_>, data: &[T], w: usize, h: usize, c: usize) -> fmt::Result {
    for y in 0..h {
        for x in 0..w {
            write!(f, "[")?;
            for ch in 0..c {
                let idx = (y * w + x) * c + ch;
                if idx < data.len() {
                    write!(f, "{:?}", data[idx])?;
                    if ch != c - 1 {
                        write!(f, ", ")?;
                    }
                }
            }
            write!(f, "] ")?;
        }
        writeln!(f)?;
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_u16_svec() {
        let shape = Shape::new(2, 2, Some(1));
        let data = ImgData::U16(vec![100, 200, 300, 400]);
        let svec = SVec::new(shape, data);

        assert_eq!(svec.shape(), (2, 2, Some(1)));
        assert_eq!(svec.get_len(), 4);

        match svec.get_data::<u16>() {
            Ok(data) => assert_eq!(data, &[100, 200, 300, 400]),
            Err(e) => panic!("Ожидали данные типа u16, но получили ошибку: {:?}", e),
        }
    }

    #[test]
    fn test_create_f32_svec() {
        let shape = Shape::new(2, 2, Some(1));
        let data = ImgData::F32(vec![1.1, 2.2, 3.3, 4.4]);
        let svec = SVec::new(shape, data);

        assert_eq!(svec.shape(), (2, 2, Some(1)));
        assert_eq!(svec.get_len(), 4);

        match svec.get_data::<f32>() {
            Ok(data) => assert_eq!(data, &[1.1, 2.2, 3.3, 4.4]),
            Err(e) => panic!("Ожидали данные типа f32, но получили ошибку: {:?}", e),
        }
    }
    #[test]
    fn test_u8_to_f32_svec() {
        let shape = Shape::new(2, 2, Some(1));
        let data = ImgData::F32(vec![1.0, 2.0, 127.0, 255.0]);
        let mut svec = SVec::new(shape, data);
        svec.as_f32();
        println!("{:?}", svec);
    }
    #[test]
    fn test_type_mismatch_error() {
        let shape = Shape::new(2, 2, Some(1));
        let data = ImgData::U8(vec![1, 2, 3, 4]);
        let svec = SVec::new(shape, data);
        // let result= svec.data;
        let result = svec.get_data::<u16>();
        // println!("{:?}",result);
        match result {
            Err(Error::TypeMismatch { expected, actual }) => {
                assert_eq!(expected, "u8");
                assert_eq!(actual, "u16");
            }
            _ => panic!("Ожидалась ошибка TypeMismatch, но получили что-то другое"),
        }
    }

    #[test]
    fn test_get_data_mut() {
        let shape = Shape::new(2, 2, Some(1));
        let data = ImgData::U8(vec![1, 2, 3, 4]);
        let mut svec = SVec::new(shape, data);

        match svec.get_data_mut::<u8>() {
            Ok(data_mut) => {
                data_mut[0] = 10;
            }
            Err(e) => panic!("Ожидали мутабельные данные типа u8, но получили ошибку: {:?}", e),
        }

        match svec.get_data::<u8>() {
            Ok(data) => assert_eq!(data, &[10, 2, 3, 4]),
            Err(e) => panic!("Ожидали данные типа u8, но получили ошибку: {:?}", e),
        }
    }
}
