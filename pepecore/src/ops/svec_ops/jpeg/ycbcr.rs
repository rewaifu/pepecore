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

pub fn data_to_ycbcr(data: &[u8], h: usize, w: usize, ver: usize, hor: usize) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    let chroma_w = (w + hor - 1) / hor;
    let chroma_h = (h + ver - 1) / ver;

    let mut yc = Vec::with_capacity(h * w);
    let mut cb = Vec::with_capacity(chroma_w * chroma_h);
    let mut cr = Vec::with_capacity(chroma_w * chroma_h);

    let row_stride = w * 3;

    for y in 0..h {
        let row = y * row_stride;
        let sample_chroma = y % ver == 0;

        for x in 0..w {
            let idx = row + x * 3;

            unsafe {
                let r = *data.get_unchecked(idx) as i32;
                let g = *data.get_unchecked(idx + 1) as i32;
                let b = *data.get_unchecked(idx + 2) as i32;

                // Всегда считаем Y
                yc.push(rgb_to_y(r, g, b));

                // CbCr только для нужных позиций
                if sample_chroma && x % hor == 0 {
                    let cb_val = (((-11059 * r - 21709 * g + 32768 * b + (128 << 16)) + 0x7FFF) >> 16) as u8;
                    let cr_val = (((32768 * r - 27439 * g - 5329 * b + (128 << 16)) + 0x7FFF) >> 16) as u8;

                    cb.push(cb_val);
                    cr.push(cr_val);
                }
            }
        }
    }

    (yc, cb, cr)
}

pub fn ycbcr_to_data(yc: &[u8], cb: &[u8], cr: &[u8], h: usize, w: usize, ver: usize, hor: usize, data: *mut u8) {
    let chroma_w = (w + hor - 1) / hor;

    unsafe {
        let mut y_idx = 0;
        for y in 0..h {
            let chroma_y = y / ver;
            let row_offset = y * w * 3;

            for x in 0..w {
                let chroma_idx = chroma_y * chroma_w + x / hor;

                let (r, g, b) = ycbcr_to_rgb(
                    *yc.get_unchecked(y_idx),
                    *cb.get_unchecked(chroma_idx),
                    *cr.get_unchecked(chroma_idx),
                );

                let pixel_offset = row_offset + x * 3;
                *data.add(pixel_offset) = r;
                *data.add(pixel_offset + 1) = g;
                *data.add(pixel_offset + 2) = b;

                y_idx += 1;
            }
        }
    }
}
