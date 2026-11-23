//! The 8×8 blocks in a plane are independent; only edge blocks need padding.

use rayon::prelude::*;

use crate::ops::svec_ops::jpeg::fdct::{fdct, idct_int};
use crate::ops::svec_ops::jpeg::quantize::QuantizationTable;

/// Plane size (bytes) below which the per-stripe loop is run sequentially:
/// spawning rayon work for a couple of stripes is not worth it.
const PARALLEL_MIN_BYTES: usize = 1 << 18; // 256 KiB

/// Applies one JPEG compression/decompression round trip to a single plane.
///
/// * `data` - `h * w` row-major samples.
/// * `h` / `w` - plane height/width, need not be multiples of 8.
/// * `q` - quantization table already scaled for this plane (luma/chroma).
pub fn jpeg_compress_one_ch(data: &mut [u8], h: usize, w: usize, q: &QuantizationTable) {
    if h == 0 || w == 0 {
        return;
    }

    let data = &mut data[..h.checked_mul(w).expect("plane dimensions overflow")];
    let stripe_len = w * h.min(8);

    if data.len() >= PARALLEL_MIN_BYTES && h >= 16 {
        data.par_chunks_mut(stripe_len)
            .for_each(|stripe| compress_stripe(stripe, w, q));
    } else {
        data.chunks_mut(stripe_len).for_each(|stripe| compress_stripe(stripe, w, q));
    }
}

/// Processes at most 8 consecutive rows of the plane (one aligned tile row).
fn compress_stripe(stripe: &mut [u8], w: usize, q: &QuantizationTable) {
    let rows = stripe.len() / w;
    for x in (0..w).step_by(8) {
        if rows == 8 && w - x >= 8 {
            compress_block_full(stripe, x, w, q);
        } else {
            compress_block_partial(stripe, x, w, rows, q);
        }
    }
}

/// Quantize and immediately dequantize without storing an intermediate block.
#[inline]
fn fdct_quant_dequant(tile: &mut [i16; 64], q: &QuantizationTable) -> [i32; 64] {
    fdct(tile);

    let mut coeffs = [0i32; 64];
    for i in 0..64 {
        coeffs[i] = q.quantize(tile[i], i) as i32 * q.get(i) as i32;
    }
    coeffs
}

/// Fast path: the whole 8x8 tile is inside the plane, so no clamping is done.
#[inline]
fn compress_block_full(img: &mut [u8], x: usize, w: usize, q: &QuantizationTable) {
    let mut tile = [0i16; 64];
    for r in 0..8 {
        let row = r * w + x;
        let src = &img[row..row + 8];
        let dst = &mut tile[r * 8..r * 8 + 8];
        for (d, &s) in dst.iter_mut().zip(src) {
            *d = s as i16 - 128;
        }
    }

    let mut coeffs = fdct_quant_dequant(&mut tile, q);
    idct_int(&mut coeffs, &mut tile, 8);

    for r in 0..8 {
        let row = r * w + x;
        let src = &tile[r * 8..r * 8 + 8];
        for (d, &s) in img[row..row + 8].iter_mut().zip(src) {
            *d = s as u8;
        }
    }
}

/// Edge path: replicates the last valid row/column to fill the 8x8 tile and
/// writes back only the in-bounds part.
fn compress_block_partial(img: &mut [u8], x: usize, w: usize, rows: usize, q: &QuantizationTable) {
    let mut ys = [0usize; 8];
    let mut xs = [0usize; 8];
    for i in 0..8 {
        ys[i] = i.min(rows - 1);
        xs[i] = (x + i).min(w - 1);
    }

    let mut tile = [0i16; 64];
    for r in 0..8 {
        let row = ys[r] * w;
        let dst = &mut tile[r * 8..r * 8 + 8];
        for (c, d) in dst.iter_mut().enumerate() {
            *d = img[row + xs[c]] as i16 - 128;
        }
    }

    let mut coeffs = fdct_quant_dequant(&mut tile, q);
    idct_int(&mut coeffs, &mut tile, 8);

    let cols = (w - x).min(8);
    for r in 0..rows {
        let row = r * w + x;
        let src = &tile[r * 8..r * 8 + cols];
        for (d, &s) in img[row..row + cols].iter_mut().zip(src) {
            *d = s as u8;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ops::svec_ops::jpeg::fdct::{fdct, idct_int};
    use crate::ops::svec_ops::jpeg::quantize::QuantizationTableType;

    fn table() -> QuantizationTable {
        QuantizationTable::new_with_quality(&QuantizationTableType::Default, 70, true)
    }

    fn pattern(h: usize, w: usize) -> Vec<u8> {
        let mut v = vec![0u8; h * w];
        let mut s: u32 = 0x1234_5678;
        for p in v.iter_mut() {
            s = s.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            *p = (s >> 24) as u8;
        }
        v
    }

    /// Edge tiles must produce exactly what a manually edge-padded image would:
    /// compress the padded plane fully and compare the real top-left region.
    #[test]
    fn partial_tile_matches_manually_padded_plane() {
        let h = 13;
        let w = 11;
        let ph = (h + 7) & !7;
        let pw = (w + 7) & !7;
        let q = table();

        let src = pattern(h, w);
        let mut actual = src.clone();
        jpeg_compress_one_ch(&mut actual, h, w, &q);

        let mut padded = vec![0u8; ph * pw];
        for y in 0..ph {
            for x in 0..pw {
                padded[y * pw + x] = src[y.min(h - 1) * w + x.min(w - 1)];
            }
        }
        jpeg_compress_one_ch(&mut padded, ph, pw, &q);

        for y in 0..h {
            for x in 0..w {
                assert_eq!(actual[y * w + x], padded[y * pw + x], "partial tile differs at ({y}, {x})");
            }
        }
    }

    /// Original per-block implementation (pre-`plane.rs`), kept as the oracle.
    fn reference_one_ch(data: &mut [u8], h: usize, w: usize, q: &QuantizationTable) {
        let blocks_y = (h + 7) / 8;
        let blocks_x = (w + 7) / 8;

        for y_idx in 0..blocks_y {
            for x_idx in 0..blocks_x {
                let x = x_idx * 8;
                let y = y_idx * 8;

                let mut tile = [0i16; 64];
                let mut n = 0;
                for iy in y..y + 8 {
                    let cy = if iy < h { iy } else { h - 1 };
                    for ix in x..x + 8 {
                        let cx = if ix < w { ix } else { w - 1 };
                        tile[n] = data[cy * w + cx] as i16 - 128;
                        n += 1;
                    }
                }

                fdct(&mut tile);
                for i in 0..64 {
                    tile[i] = q.quantize(tile[i], i);
                }

                let mut dct2 = [0i32; 64];
                for i in 0..64 {
                    dct2[i] = tile[i] as i32 * q.get(i) as i32;
                }
                idct_int(&mut dct2, &mut tile, 8);

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
                        data[iy * w + ix] = tile[row * 8 + col] as u8;
                    }
                }
            }
        }
    }

    #[test]
    fn matches_reference_implementation() {
        let q = table();
        for &(h, w) in &[(1, 1), (8, 8), (9, 9), (13, 11), (16, 21), (40, 37), (33, 64)] {
            let src = pattern(h, w);
            let mut expected = src.clone();
            let mut actual = src;
            reference_one_ch(&mut expected, h, w, &q);
            jpeg_compress_one_ch(&mut actual, h, w, &q);
            assert_eq!(actual, expected, "mismatch for {h}x{w}");
        }
    }

    #[test]
    fn parallel_stripes_match_sequential() {
        use rayon::prelude::*;

        let h = 64;
        let w = 61;
        let q = table();
        let src = pattern(h, w);
        let stripe_len = w * 8;

        let mut sequential = src.clone();
        sequential
            .chunks_mut(stripe_len)
            .for_each(|stripe| compress_stripe(stripe, w, &q));

        let mut parallel = src;
        parallel
            .par_chunks_mut(stripe_len)
            .for_each(|stripe| compress_stripe(stripe, w, &q));

        assert_eq!(sequential, parallel);
    }
}
