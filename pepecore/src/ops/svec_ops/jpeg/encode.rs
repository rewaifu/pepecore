use crate::enums::YCbCrRatio;
use crate::ops::svec_ops::jpeg::fdct::{fdct, idct_int};
use crate::ops::svec_ops::jpeg::quantize::{QuantizationTable, QuantizationTableType};
use crate::ops::svec_ops::jpeg::ycbcr::{data_to_ycbcr, ycbcr_to_data};
use pepecore_array::SVec;

fn quantize_block(block: &mut [i16; 64], table: &QuantizationTable) {
    for i in 0..64 {
        block[i] = table.quantize(block[i], i);
    }
}

fn dequantize_block(q_block: &[i16; 64], d_block: &mut [i32; 64], table: &QuantizationTable) {
    for i in 0..64 {
        d_block[i] = q_block[i] as i32 * table.get(i) as i32;
    }
}

fn get_tile(img: &[u8], img_tile: &mut [i16; 64], x: usize, y: usize, w: usize, h: usize) {
    let mut n = 0;
    for iy in y..y + 8 {
        let clamped_y = if iy < h { iy } else { h - 1 };
        for ix in x..x + 8 {
            let clamped_x = if ix < w { ix } else { w - 1 };
            img_tile[n] = img[clamped_y * w + clamped_x] as i16 - 128;
            n += 1;
        }
    }
}

fn copy_tile(img: &mut [u8], tile: &[i16; 64], x: usize, y: usize, w: usize, h: usize) {
    for row in 0..8 {
        let iy = y + row;
        if iy >= h {
            break;
        }
        for col in 0..8 {
            let ix = x + col;
            if ix >= w {
                continue;
            }
            img[iy * w + ix] = tile[row * 8 + col] as u8;
        }
    }
}

pub fn jpeg_compress_one_ch(data: &mut [u8], h: usize, w: usize, q: &QuantizationTable) {
    let mut tile = [0i16; 64];
    let mut dct2 = [0i32; 64];

    let blocks_y = (h + 7) / 8;
    let blocks_x = (w + 7) / 8;

    for y_idx in 0..blocks_y {
        for x_idx in 0..blocks_x {
            let x = x_idx * 8;
            let y = y_idx * 8;
            get_tile(data, &mut tile, x, y, w, h);
            fdct(&mut tile);
            quantize_block(&mut tile, q);
            dequantize_block(&tile, &mut dct2, q);
            idct_int(&mut dct2, &mut tile, 8);
            copy_tile(data, &tile, x, y, w, h);
        }
    }
}
pub fn jpeg_compress(img: &mut SVec, quality: u8, qt: &QuantizationTableType, yuv: &YCbCrRatio) {
    img.as_u8();
    let (h, w, c) = img.shape();
    let data = img.get_data::<u8>().unwrap();
    let c = c.unwrap_or(1);
    if c == 3 {
        let (hor, ver) = match yuv {
            YCbCrRatio::R444 => (1, 1),
            YCbCrRatio::R440 => (2, 1),
            YCbCrRatio::R441 => (4, 1),
            YCbCrRatio::R422 => (1, 2),
            YCbCrRatio::R420 => (2, 2),
            YCbCrRatio::R411 => (1, 4),
            YCbCrRatio::R410 => (4, 4),
        };
        let (mut y, mut u, mut v) = data_to_ycbcr(data, h, w, ver, hor);

        jpeg_compress_one_ch(&mut y, h, w, &QuantizationTable::new_with_quality(qt, quality, true));
        jpeg_compress_one_ch(
            &mut u,
            h / ver,
            w / hor,
            &QuantizationTable::new_with_quality(qt, quality, false),
        );
        jpeg_compress_one_ch(
            &mut v,
            h / ver,
            w / hor,
            &QuantizationTable::new_with_quality(qt, quality, false),
        );
        ycbcr_to_data(&y, &u, &v, h, w, ver, hor, img.get_mut_ptr().unwrap());
    } else if c == 1 {
        jpeg_compress_one_ch(
            img.get_data_mut().unwrap(),
            h,
            w,
            &QuantizationTable::new_with_quality(qt, quality, true),
        );
    }
}
