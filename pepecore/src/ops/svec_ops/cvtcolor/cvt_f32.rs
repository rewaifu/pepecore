use pepecore_array::{ImgData, SVec, Shape};
use std::ops::Mul;

pub fn rgb_to_gray_f32(img: &mut SVec, r: f32, g: f32, b: f32) {
    let (height, width, c) = img.shape();
    assert_eq!(c, Some(3));
    let num_pixels = height * width;
    img.shape = Shape::new(height, width, None);
    unsafe {
        let out_ptr = img.get_mut_ptr::<f32>().unwrap();
        for i in 0..num_pixels {
            let num = i.unchecked_mul(3);
            *out_ptr.add(i) = (*out_ptr.add(num)).mul_add(r, (*out_ptr.add(num + 1)).mul_add(g, *out_ptr.add(num + 2) * b));
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

pub fn ycbcr_to_rgb_f32(img: &mut SVec, r: f32, g: f32, b: f32) {
    let len = img.get_len();
    let (_, _, c) = img.shape();
    assert_eq!(c, Some(3));
    let ptr = img.get_mut_ptr::<f32>().unwrap();
    let (ke, kd, crg, cbg) = get_crg_cbg(r, g, b);
    unsafe {
        let mut i = 0;
        while i + 2 < len {
            let y = *ptr.add(i);
            let cb = *ptr.add(i + 1) - 0.5;
            let cr = *ptr.add(i + 2) - 0.5;

            *ptr.add(i) = cr.mul_add(ke, y);
            *ptr.add(i + 1) = cb.mul_add(cbg, cr.mul_add(crg, y));
            *ptr.add(i + 2) = cb.mul_add(kd, y);
            i += 3;
        }
    }
}

pub fn rgb_to_ycbcr_f32(img: &mut SVec, r: f32, g: f32, b: f32) {
    let len = img.get_len();
    let (_, _, c) = img.shape();
    assert_eq!(c, Some(3));

    let ptr = img.get_mut_ptr::<f32>().unwrap();
    let ke: f32 = 1.0 / ((1_f32 - r) / 0.5);
    let kd: f32 = 1.0 / ((1_f32 - b) / 0.5);
    unsafe {
        let mut i = 0;
        while i + 2 < len {
            let vr = *ptr.add(i);
            let vg = *ptr.add(i.unchecked_add(1));
            let vb = *ptr.add(i.unchecked_add(2));
            let y = vr.mul_add(r, vg.mul_add(g, vb.mul(b)));
            *ptr.add(i.unchecked_add(1)) = (vb - y).mul_add(kd, 0.5);
            *ptr.add(i.unchecked_add(2)) = (vr - y).mul_add(ke, 0.5);
            *ptr.add(i) = y;
            i += 3;
        }
    }
}

pub fn rgb_to_cmyk_f32(img: &mut SVec) {
    let (height, width, c) = img.shape();
    assert_eq!(c, Some(3));
    img.shape = Shape::new(height, width, Some(4));
    let num_pixels = height * width;
    let mut buff_vec: Vec<f32> = Vec::with_capacity(num_pixels * 4);

    let img_data = img.get_data::<f32>().unwrap();

    for index in 0..num_pixels {
        let triple_index = index * 3;
        let r_s = 1.0 - img_data[triple_index];
        let g_s = 1.0 - img_data[triple_index + 1];
        let b_s = 1.0 - img_data[triple_index + 2];

        if r_s == 1.0 && g_s == 1.0 && b_s == 1.0 {
            buff_vec.extend([0.0, 0.0, 0.0, 1.0]);
            continue;
        }
        let min_cmy = r_s.min(g_s).min(b_s);
        let inv_min_cmy = 1.0 - min_cmy;
        buff_vec.extend([
            (r_s - min_cmy) / inv_min_cmy,
            (g_s - min_cmy) / inv_min_cmy,
            (b_s - min_cmy) / inv_min_cmy,
            min_cmy,
        ])
    }

    img.data = ImgData::F32(buff_vec);
}

pub fn cmyk_to_rgb_f32(img: &mut SVec) {
    let (height, width, c) = img.shape();
    assert_eq!(c, Some(4));
    img.shape = Shape::new(height, width, Some(3));
    let num_pixels = height * width;
    let mut_img = img.get_data_mut::<f32>().unwrap();
    for index in 0..num_pixels {
        let img_index = index * 4;
        let k = 1.0 - mut_img[img_index + 3];
        let rgb_index = index * 3;
        if k == 0.0 {
            mut_img[rgb_index] = 0.0;
            mut_img[rgb_index + 1] = 0.0;
            mut_img[rgb_index + 2] = 0.0;
            continue;
        }
        mut_img[rgb_index] = (1.0 - mut_img[img_index]) * k;
        mut_img[rgb_index + 1] = (1.0 - mut_img[img_index + 1]) * k;
        mut_img[rgb_index + 2] = (1.0 - mut_img[img_index + 2]) * k;
    }
    img.truncate(num_pixels * 3).unwrap()
}
