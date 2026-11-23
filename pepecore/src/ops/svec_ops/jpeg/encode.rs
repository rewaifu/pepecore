pub use super::plane::jpeg_compress_one_ch;
use crate::enums::YCbCrRatio;
use crate::ops::svec_ops::jpeg::quantize::{QuantizationTable, QuantizationTableType};
use crate::ops::svec_ops::jpeg::ycbcr::{data_to_ycbcr, ycbcr_to_data};
use pepecore_array::SVec;
use rayon::prelude::*;

fn compress_rgb<const X: usize, const Y: usize>(
    data: &mut [u8],
    h: usize,
    w: usize,
    luma: &QuantizationTable,
    chroma: &QuantizationTable,
) {
    let stride = w.checked_mul(3).expect("RGB row overflow");
    assert_eq!(data.len(), h.checked_mul(stride).expect("RGB frame overflow"));
    // Align bands to both DCT grids. Each worker converts, transforms and
    // reconstructs its own band while its three planes are still cache-hot.
    let band_height = h.min(8 * Y);
    let process = |rgb: &mut [u8]| {
        let rows = rgb.len() / stride;
        let (mut y, mut cb, mut cr) = data_to_ycbcr::<X, Y>(rgb, rows, w);
        let cw = w.div_ceil(X);
        let ch = rows.div_ceil(Y);
        jpeg_compress_one_ch(&mut y, rows, w, luma);
        jpeg_compress_one_ch(&mut cb, ch, cw, chroma);
        jpeg_compress_one_ch(&mut cr, ch, cw, chroma);
        ycbcr_to_data::<X, Y>(&y, &cb, &cr, rows, w, rgb);
    };
    if data.len() >= 3 * 65536 {
        data.par_chunks_mut(stride * band_height).for_each(process);
    } else {
        data.chunks_mut(stride * band_height).for_each(process);
    }
}

pub fn jpeg_compress(img: &mut SVec, quality: u8, qt: &QuantizationTableType, yuv: &YCbCrRatio) {
    img.as_u8();
    let (h, w, c) = img.shape();
    if h == 0 || w == 0 {
        return;
    }
    let data = img.get_data_mut::<u8>().unwrap();
    let luma = QuantizationTable::new_with_quality(qt, quality, true);
    match c.unwrap_or(1) {
        1 => jpeg_compress_one_ch(data, h, w, &luma),
        3 => {
            let chroma = QuantizationTable::new_with_quality(qt, quality, false);
            // Horizontal then vertical extent of one chroma sample in luma pixels.
            let render = match yuv {
                YCbCrRatio::R444 => compress_rgb::<1, 1>,
                YCbCrRatio::R440 => compress_rgb::<1, 2>,
                YCbCrRatio::R441 => compress_rgb::<1, 4>,
                YCbCrRatio::R422 => compress_rgb::<2, 1>,
                YCbCrRatio::R420 => compress_rgb::<2, 2>,
                YCbCrRatio::R411 => compress_rgb::<4, 1>,
                YCbCrRatio::R410 => compress_rgb::<4, 2>,
            };
            render(data, h, w, &luma, &chroma);
        }
        _ => {}
    }
}
