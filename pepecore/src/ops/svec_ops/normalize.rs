use crate::global_params::rayon_get_mode;
use pepecore_array::{PixelType, SVec};
use rayon::iter::{IntoParallelRefIterator, IntoParallelRefMutIterator};
use rayon::prelude::*;
use std::usize;

pub trait NormalizeSVec {
    fn normalize(&mut self, scale: f32);
}
pub trait SVecPixel: Sized + Copy + PartialOrd + 'static {
    const MIN_VALUE: Self;
    const MAX_VALUE: Self;
    fn min(&mut self, value: Self);
    fn max(&mut self, value: Self);
    fn as_f32(&self) -> f32;
    fn from_f32(value: f32) -> Self;
}
impl SVecPixel for f32 {
    const MIN_VALUE: Self = 0.0;
    const MAX_VALUE: Self = 1.0;
    fn min(&mut self, value: Self) {
        if *self > value {
            *self = value
        }
    }
    fn max(&mut self, value: Self) {
        if *self < value {
            *self = value
        }
    }
    fn as_f32(&self) -> f32 {
        *self
    }
    fn from_f32(value: f32) -> Self {
        value
    }
}
impl SVecPixel for u8 {
    const MIN_VALUE: Self = u8::MIN;
    const MAX_VALUE: Self = u8::MAX;
    fn min(&mut self, value: Self) {
        if *self > value {
            *self = value
        }
    }
    fn max(&mut self, value: Self) {
        if *self < value {
            *self = value
        }
    }
    fn as_f32(&self) -> f32 {
        *self as f32
    }
    fn from_f32(value: f32) -> Self {
        (value * 255.0) as u8
    }
}
impl SVecPixel for u16 {
    const MIN_VALUE: Self = u16::MIN;
    const MAX_VALUE: Self = u16::MAX;
    fn min(&mut self, value: Self) {
        if *self > value {
            *self = value
        }
    }
    fn max(&mut self, value: Self) {
        if *self < value {
            *self = value
        }
    }
    fn as_f32(&self) -> f32 {
        *self as f32
    }
    fn from_f32(value: f32) -> Self {
        (value * 65535.0) as u16
    }
}
fn normalize<T: SVecPixel>(img: &mut SVec, scale: f32) {
    let (h, w, c) = img.shape();
    let data = img.get_data_mut::<T>().unwrap();
    let mut min = T::MAX_VALUE;
    let mut max = T::MIN_VALUE;
    let c = c.unwrap_or(1);
    let (x_in_tab, y_in_tab): (Vec<usize>, Vec<usize>) = if scale < 1.0 {
        (
            (0..(w as f32 * scale) as usize)
                .map(|x| ((x as f32 / scale).round() as usize).min(w - 1))
                .collect(),
            (0..(h as f32 * scale) as usize)
                .map(|y| ((y as f32 / scale).round() as usize).min(h - 1))
                .collect(),
        )
    } else {
        ((0..w).collect(), (0..h).collect())
    };
    for y in y_in_tab {
        for x in x_in_tab.iter() {
            for z in 0..c {
                let value = &data[(y * w + x) * c + z];
                min.min(*value);
                max.max(*value);
            }
        }
    }
    if min == max {
        return;
    }
    let min: f32 = min.as_f32();
    let div: f32 = max.as_f32() - min;
    for val in data.iter_mut() {
        *val = T::from_f32((val.as_f32() - min) / div)
    }
}
fn rayon_normalize<T>(img: &mut SVec, scale: f32)
where
    T: SVecPixel + Send + Sync,
{
    let (h, w, c_opt) = img.shape();
    let data = img.get_data_mut::<T>().unwrap();
    let c = c_opt.unwrap_or(1);

    // Вычисляем таблицы выборочных координат
    let (x_in_tab, y_in_tab): (Vec<usize>, Vec<usize>) = if scale < 1.0 {
        let x_tab = (0..(w as f32 * scale) as usize)
            .map(|x| ((x as f32 / scale).round() as usize).min(w - 1))
            .collect();
        let y_tab = (0..(h as f32 * scale) as usize)
            .map(|y| ((y as f32 / scale).round() as usize).min(h - 1))
            .collect();
        (x_tab, y_tab)
    } else {
        ((0..w).collect(), (0..h).collect())
    };

    // Плоский вектор индексов для выборки
    let sample_indices: Vec<usize> = y_in_tab
        .iter()
        .flat_map(|&y| x_in_tab.iter().flat_map(move |&x| (0..c).map(move |z| (y * w + x) * c + z)))
        .collect();

    // Параллельно находим (min, max) по выборке
    let (min_pixel, max_pixel) = sample_indices
        .par_iter()
        .map(|&idx| {
            let v = data[idx];
            (v, v) // локальный (min, max) для одного элемента
        })
        .reduce(
            || (T::MAX_VALUE, T::MIN_VALUE),
            |(min_v, max_v), (mn, mx)| {
                let mut min_acc = min_v;
                let mut max_acc = max_v;
                min_acc.min(mn);
                max_acc.max(mx);
                (min_acc, max_acc)
            },
        );

    // Если все пиксели одинаковы — выходим
    if min_pixel == max_pixel {
        return;
    }

    let min_f = min_pixel.as_f32();
    let div = max_pixel.as_f32() - min_f;

    // Параллельно нормализуем весь буфер
    data.par_iter_mut().for_each(|val| {
        let normalized = (val.as_f32() - min_f) / div;
        *val = T::from_f32(normalized);
    });
}

impl NormalizeSVec for SVec {
    fn normalize(&mut self, scale: f32) {
        match self.pixel_type() {
            PixelType::F32 => {
                if rayon_get_mode() {
                    rayon_normalize::<f32>(self, scale)
                } else {
                    normalize::<f32>(self, scale);
                }
            }
            PixelType::U8 => {
                if rayon_get_mode() {
                    rayon_normalize::<u8>(self, scale)
                } else {
                    normalize::<u8>(self, scale);
                }
            }
            PixelType::U16 => {
                if rayon_get_mode() {
                    rayon_normalize::<u16>(self, scale)
                } else {
                    normalize::<u16>(self, scale);
                }
            }
        }
    }
}
