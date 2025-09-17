use pepecore_array::SVec;

#[inline(always)]
fn sum_ch(dct: &[f32], base: usize, c: usize) -> f32 {
    match c {
        1 => unsafe { *dct.get_unchecked(base) },
        3 => unsafe { *dct.get_unchecked(base) + *dct.get_unchecked(base + 1) + *dct.get_unchecked(base + 2) },
        4 => unsafe {
            *dct.get_unchecked(base) + *dct.get_unchecked(base + 1) + *dct.get_unchecked(base + 2) + *dct.get_unchecked(base + 3)
        },
        _ => {
            let mut s = 0.0f32;
            for k in 0..c {
                unsafe { s += *dct.get_unchecked(base + k) }
            }
            s
        }
    }
}

pub fn get_full_original_size(img: &SVec) -> (usize, usize) {
    let mut dct_i = img.clone();
    dct_i.dct2().unwrap();

    let (h, w, c_opt) = img.shape.get_shape();
    let c = c_opt.unwrap_or(1);
    let dct = dct_i.get_data::<f32>().unwrap();
    let mut row_sum = vec![0.0f32; h];
    let mut col_sum = vec![0.0f32; w];

    if c == 1 {
        for i in 0..h {
            let mut rs = 0.0f32;
            let mut base = i * w;
            for j in 0..w {
                let v = unsafe { *dct.get_unchecked(base) };
                rs += v;
                unsafe { *col_sum.get_unchecked_mut(j) += v };
                base += 1;
            }
            unsafe { *row_sum.get_unchecked_mut(i) = rs };
        }
    } else {
        for i in 0..h {
            let mut rs = 0.0f32;
            let mut base = i * w * c; // шаг c
            for j in 0..w {
                let s = sum_ch(dct, base, c);
                rs += s;
                unsafe { *col_sum.get_unchecked_mut(j) += s };
                base += c;
            }
            unsafe { *row_sum.get_unchecked_mut(i) = rs };
        }
    }
    let w_f = w as f32 * c as f32;
    let h_f = h as f32 * c as f32;
    for r in &mut row_sum {
        *r /= w_f;
    }
    for x in &mut col_sum {
        *x /= h_f;
    }
    let threshold = 1e-3f32;
    let mut min_diff = f32::MAX;

    let mut index_h = h;
    for i in 0..h {
        let dist = unsafe { *row_sum.get_unchecked(i) };
        if dist < min_diff {
            min_diff = dist;
            if dist < threshold {
                index_h = i;
                break;
            }
        }
    }

    min_diff = f32::MAX;
    let mut index_w = w;
    for j in 0..w {
        let dist = unsafe { *col_sum.get_unchecked(j) };
        if dist < min_diff {
            min_diff = dist;
            if dist < threshold {
                index_w = j;
                break;
            }
        }
    }

    (index_h, index_w)
}
pub fn get_original_height_only(img: &SVec) -> usize {
    let mut dct_i = img.clone();
    dct_i.dct2().unwrap();

    let (h, w, c_opt) = img.shape.get_shape();
    let c = c_opt.unwrap_or(1);
    let dct = dct_i.get_data::<f32>().unwrap();

    let mut row_sum = vec![0.0f32; h];

    if c == 1 {
        for i in 0..h {
            let mut rs = 0.0f32;
            let mut base = i * w;
            for _j in 0..w {
                let v = unsafe { *dct.get_unchecked(base) };
                rs += v;
                base += 1;
            }
            unsafe { *row_sum.get_unchecked_mut(i) = rs };
        }
    } else {
        for i in 0..h {
            let mut rs = 0.0f32;
            let mut base = i * w * c;
            for _j in 0..w {
                rs += sum_ch(dct, base, c);
                base += c;
            }
            unsafe { *row_sum.get_unchecked_mut(i) = rs };
        }
    }

    let w_f = w as f32 * c as f32;
    for r in &mut row_sum {
        *r /= w_f;
    }

    let threshold = 1e-3f32;
    let mut min_diff = f32::MAX;
    let mut index_h = h;

    for i in 0..h {
        let dist = unsafe { *row_sum.get_unchecked(i) };
        if dist < min_diff {
            min_diff = dist;
            if dist < threshold {
                index_h = i;
                break;
            }
        }
    }
    index_h
}

pub fn get_original_width_only(img: &SVec) -> usize {
    let mut dct_i = img.clone();
    dct_i.dct2().unwrap();

    let (h, w, c_opt) = img.shape.get_shape();
    let c = c_opt.unwrap_or(1);
    let dct = dct_i.get_data::<f32>().unwrap();

    let mut col_sum = vec![0.0f32; w];

    if c == 1 {
        for i in 0..h {
            let mut base = i * w;
            for j in 0..w {
                let v = unsafe { *dct.get_unchecked(base) };
                unsafe { *col_sum.get_unchecked_mut(j) += v };
                base += 1;
            }
        }
    } else {
        for i in 0..h {
            let mut base = i * w * c;
            for j in 0..w {
                let s = sum_ch(dct, base, c);
                unsafe { *col_sum.get_unchecked_mut(j) += s };
                base += c;
            }
        }
    }

    // нормировка по высоте и каналам точно как у тебя
    let h_f = h as f32 * c as f32;
    for x in &mut col_sum {
        *x /= h_f;
    }

    // поиск первого индекса ниже порога
    let threshold = 1e-3f32;
    let mut min_diff = f32::MAX;
    let mut index_w = w;

    for j in 0..w {
        let dist = unsafe { *col_sum.get_unchecked(j) };
        if dist < min_diff {
            min_diff = dist;
            if dist < threshold {
                index_w = j;
                break;
            }
        }
    }
    index_w
}
