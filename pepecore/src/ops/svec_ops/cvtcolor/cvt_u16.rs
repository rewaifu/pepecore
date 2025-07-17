use pepecore_array::{ImgData, SVec, Shape};
use std::ops::Mul;

pub fn rgb_to_gray_u16(img: &mut SVec, r: f32, g: f32, b: f32) {
    let (height, width, _) = img.shape();
    let num_pixels = height * width;
    img.shape = Shape::new(height, width, None);
    unsafe {
        let out_ptr = img.get_mut_ptr::<u16>().unwrap();
        for i in 0..num_pixels {
            let num = i.unchecked_mul(3);
            *out_ptr.add(i) = (*out_ptr.add(num) as f32)
                .mul_add(r, (*out_ptr.add(num + 1) as f32).mul_add(g, *out_ptr.add(num + 2) as f32 * b))
                as u16;
        }
    }
    img.truncate(num_pixels).unwrap()
}

pub fn get_crg_cbg(r: f32, g: f32, b: f32) -> (f32, f32, f32, f32) {
    let ke = 0.5 / (1.0 - r);
    let kd = 0.5 / (1.0 - b);
    let crg = -(r / g) * ke;
    let cbg = -(b / g) * kd;
    (1.0 / ke, 1.0 / kd, crg, cbg)
}

const U16_HALF: f32 = u16::MAX as f32 / 2.0;

pub fn ycbcr_to_rgb_u16(img: &mut SVec, r: f32, g: f32, b: f32) {
    let len = img.get_len();
    let ptr = img.get_mut_ptr::<u16>().unwrap();
    let (ke, kd, crg, cbg) = get_crg_cbg(r, g, b);
    unsafe {
        let mut i = 0;
        while i + 2 < len {
            let y = *ptr.add(i) as f32;
            let cb = *ptr.add(i + 1) as f32 - U16_HALF;
            let cr = *ptr.add(i + 2) as f32 - U16_HALF;

            *ptr.add(i) = cr.mul_add(ke, y) as u16;
            *ptr.add(i + 1) = cb.mul_add(cbg, cr.mul_add(crg, y)) as u16;
            *ptr.add(i + 2) = cb.mul_add(kd, y) as u16;
            i += 3;
        }
    }
}

pub fn rgb_to_ycbcr_u16(img: &mut SVec, r: f32, g: f32, b: f32) {
    let len = img.get_len();
    let ptr = img.get_mut_ptr::<u16>().unwrap();
    let ke: f32 = 1.0 / ((1_f32 - r) / 0.5);
    let kd: f32 = 1.0 / ((1_f32 - b) / 0.5);
    unsafe {
        let mut i = 0;
        while i + 2 < len {
            let vr = *ptr.add(i) as f32;
            let vg = *ptr.add(i.unchecked_add(1)) as f32;
            let vb = *ptr.add(i.unchecked_add(2)) as f32;
            let y = vr.mul_add(r, vg.mul_add(g, vb.mul(b)));
            *ptr.add(i.unchecked_add(1)) = (vb - y).mul_add(kd, U16_HALF) as u16;
            *ptr.add(i.unchecked_add(2)) = (vr - y).mul_add(ke, U16_HALF) as u16;
            *ptr.add(i) = y as u16;
            i += 3;
        }
    }
}

const U16_MAX_F32: f32 = u16::MAX as f32;

pub fn rgb_to_cmyk_u16(img: &mut SVec) {
    let (height, width, _) = img.shape();
    img.shape = Shape::new(height, width, Some(4));
    let num_pixels = height * width;
    let mut buff_vec: Vec<u16> = Vec::with_capacity(num_pixels * 4);

    let img_data = img.get_data::<u16>().unwrap();

    for index in 0..num_pixels {
        let triple_index = index * 3;
        let r_s = (u16::MAX - img_data[triple_index]) as f32 / U16_MAX_F32;
        let g_s = (u16::MAX - img_data[triple_index + 1]) as f32 / U16_MAX_F32;
        let b_s = (u16::MAX - img_data[triple_index + 2]) as f32 / U16_MAX_F32;

        if r_s == 1.0 && g_s == 1.0 && b_s == 1.0 {
            buff_vec.extend([0, 0, 0, u16::MAX]);
            continue;
        }
        let min_cmy = r_s.min(g_s).min(b_s);
        let inv_min_cmy = 1.0 - min_cmy;
        buff_vec.extend([
            ((r_s - min_cmy) / inv_min_cmy * U16_MAX_F32) as u16,
            ((g_s - min_cmy) / inv_min_cmy * U16_MAX_F32) as u16,
            ((b_s - min_cmy) / inv_min_cmy * U16_MAX_F32) as u16,
            (min_cmy * U16_MAX_F32) as u16,
        ])
    }

    img.data = ImgData::U16(buff_vec);
}

pub fn cmyk_to_rgb_u16(img: &mut SVec) {
    let (height, width, _) = img.shape();
    img.shape = Shape::new(height, width, Some(3));
    let num_pixels = height * width;
    let mut_img = img.get_data_mut::<u16>().unwrap();
    for index in 0..num_pixels {
        let img_index = index * 4;
        let k = (u16::MAX - mut_img[img_index + 3]) as f32 / U16_MAX_F32;
        let rgb_index = index * 3;
        if k == 0.0 {
            mut_img[rgb_index] = 0;
            mut_img[rgb_index + 1] = 0;
            mut_img[rgb_index + 2] = 0;
            continue;
        }
        mut_img[rgb_index] = ((u16::MAX - mut_img[img_index]) as f32 * k) as u16;
        mut_img[rgb_index + 1] = ((u16::MAX - mut_img[img_index + 1]) as f32 * k) as u16;
        mut_img[rgb_index + 2] = ((u16::MAX - mut_img[img_index + 2]) as f32 * k) as u16;
    }
    img.truncate(num_pixels * 3).unwrap()
}
