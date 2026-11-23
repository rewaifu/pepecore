//! X and Y are the horizontal and vertical chroma sample extents.
//! Box-filter real pixels on reduction and replicate the sample on expansion.

#[inline(always)]
fn rgb_to_cb(r: i32, g: i32, b: i32) -> i32 {
    (((-11059 * r - 21709 * g + 32768 * b + (128 << 16)) + 0x7FFF) >> 16).clamp(0, 255)
}

#[inline(always)]
fn rgb_to_cr(r: i32, g: i32, b: i32) -> i32 {
    (((32768 * r - 27439 * g - 5329 * b + (128 << 16)) + 0x7FFF) >> 16).clamp(0, 255)
}

#[inline(always)]
pub fn rgb_to_y(r: i32, g: i32, b: i32) -> u8 {
    (((19595 * r + 38470 * g + 7471 * b) + 0x7FFF) >> 16) as u8
}

#[inline(always)]
pub fn ycbcr_to_rgb(y: u8, cb: u8, cr: u8) -> (u8, u8, u8) {
    let y = y as i32;
    let cb = cb as i32 - 128;
    let cr = cr as i32 - 128;

    let r = (y + ((91881 * cr + 32768) >> 16)).clamp(0, 255) as u8;
    let g = (y - ((22554 * cb + 46802 * cr + 32768) >> 16)).clamp(0, 255) as u8;
    let b = (y + ((116130 * cb + 32768) >> 16)).clamp(0, 255) as u8;

    (r, g, b)
}

/// Converts interleaved RGB `data` (`h * w * 3` bytes, row-major) to full
/// resolution Y and box-averaged Cb/Cr planes with ceil chroma geometry.
pub(super) fn data_to_ycbcr<const X: usize, const Y: usize>(data: &[u8], h: usize, w: usize) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    let stride = w.checked_mul(3).unwrap();
    let yc_len = h.checked_mul(w).unwrap();
    assert_eq!(data.len(), yc_len.checked_mul(3).unwrap());
    let chroma_w = w.div_ceil(X);
    let chroma_h = h.div_ceil(Y);
    let chroma_len = chroma_w * chroma_h;
    let mut yc = vec![0u8; yc_len];
    let mut cb = vec![0u8; chroma_len];
    let mut cr = vec![0u8; chroma_len];
    if X == 1 && Y == 1 {
        for (((p, y), u), v) in data.chunks_exact(3).zip(&mut yc).zip(&mut cb).zip(&mut cr) {
            let (r, g, b) = (p[0] as i32, p[1] as i32, p[2] as i32);
            *y = rgb_to_y(r, g, b);
            *u = rgb_to_cb(r, g, b) as u8;
            *v = rgb_to_cr(r, g, b) as u8;
        }
        return (yc, cb, cr);
    }

    for cy in 0..chroma_h {
        let y0 = cy * Y;
        let y1 = (y0 + Y).min(h);

        for cx in 0..chroma_w {
            let x0 = cx * X;
            let x1 = (x0 + X).min(w);

            let mut sum_cb: u32 = 0;
            let mut sum_cr: u32 = 0;

            for y in y0..y1 {
                let row = &data[y * stride..y * stride + stride];
                let row_y = &mut yc[y * w..y * w + w];
                let pixels = &row[x0 * 3..x1 * 3];
                let out_y = &mut row_y[x0..x1];

                for (rgb, yv) in pixels.chunks_exact(3).zip(out_y.iter_mut()) {
                    let r = rgb[0] as i32;
                    let g = rgb[1] as i32;
                    let b = rgb[2] as i32;

                    *yv = rgb_to_y(r, g, b);
                    sum_cb += rgb_to_cb(r, g, b) as u32;
                    sum_cr += rgb_to_cr(r, g, b) as u32;
                }
            }

            // Rounded box average over the pixels that actually exist in
            // this (possibly partial) cell.
            let count = ((y1 - y0) * (x1 - x0)) as u32;
            let idx = cy * chroma_w + cx;
            cb[idx] = ((sum_cb + count / 2) / count) as u8;
            cr[idx] = ((sum_cr + count / 2) / count) as u8;
        }
    }

    (yc, cb, cr)
}

/// Converts full resolution Y plus ceil-geometry Cb/Cr planes back to
/// interleaved RGB in `data`, replicating each chroma sample over its block
/// (nearest upsampling).
pub(super) fn ycbcr_to_data<const X: usize, const Y: usize>(
    yc: &[u8],
    cb: &[u8],
    cr: &[u8],
    h: usize,
    w: usize,
    data: &mut [u8],
) {
    let n = h.checked_mul(w).unwrap();
    assert_eq!(yc.len(), n);
    assert_eq!(data.len(), n.checked_mul(3).unwrap());
    let cw = w.div_ceil(X);
    assert_eq!(cb.len(), cw * h.div_ceil(Y));
    assert_eq!(cr.len(), cb.len());
    for (y, (values, out)) in yc.chunks_exact(w).zip(data.chunks_exact_mut(w * 3)).enumerate() {
        let base = (y / Y) * cw;
        let u = &cb[base..base + cw];
        let v = &cr[base..base + cw];
        for (x, (&luma, pixel)) in values.iter().zip(out.chunks_exact_mut(3)).enumerate() {
            // X is constant; no run-time division in the pixel loop.
            let (r, g, b) = ycbcr_to_rgb(luma, u[x / X], v[x / X]);
            pixel.copy_from_slice(&[r, g, b]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neutral_grey_roundtrip_all_values_no_wrap() {
        for k in 0..=255u8 {
            let data = [k, k, k, k, k, k, k, k, k, k, k, k]; // 2x2
            let (yc, cb, cr) = data_to_ycbcr::<2, 2>(&data, 2, 2);
            assert_eq!(yc, [k, k, k, k]);
            assert!(cb.iter().all(|&v| v == 128), "Cb must be neutral at k={k}");
            assert!(cr.iter().all(|&v| v == 128), "Cr must be neutral at k={k}");

            let mut out = [0u8; 12];
            ycbcr_to_data::<2, 2>(&yc, &cb, &cr, 2, 2, &mut out);
            assert_eq!(out, data, "grey roundtrip must be exact at k={k}");

            // 4:4:4 geometry as well.
            let (yc, cb, cr) = data_to_ycbcr::<1, 1>(&data, 2, 2);
            let mut out = [0u8; 12];
            ycbcr_to_data::<1, 1>(&yc, &cb, &cr, 2, 2, &mut out);
            assert_eq!(out, data, "4:4:4 grey roundtrip must be exact at k={k}");
        }
    }
}
