use crate::ops::svec_ops::cvtcolor::lut::{create_lut_rgb2gray, create_lut_rgb2ycbcr, create_lut_ycbcr2rgb};
use pepecore_array::{ImgData, SVec, Shape};

pub fn rgb_to_gray_u8(img: &mut SVec, r: f32, g: f32, b: f32) {
    let (lut_r, lut_g, lut_b) = create_lut_rgb2gray(r, g, b);
    let (height, width, c) = img.shape();
    assert_eq!(c, Some(3));
    let num_pixels = height * width;
    img.shape = Shape::new(height, width, None);
    unsafe {
        let out_ptr = img.get_mut_ptr::<u8>().unwrap();
        for i in 0..num_pixels {
            let num = i.unchecked_mul(3);
            *out_ptr.add(i) = lut_r[*out_ptr.add(num) as usize]
                .unchecked_add(lut_g[*out_ptr.add(num.unchecked_add(1)) as usize])
                .unchecked_add(lut_b[*out_ptr.add(num.unchecked_add(2)) as usize]);
        }
    }
    img.truncate(num_pixels).unwrap()
}

pub fn rgb_to_ycbcr_u8(img: &mut SVec, r: f32, g: f32, b: f32) {
    let (lut_y_r, lut_y_g, lut_y_b, lut_cb_r, lut_cb_g, lut_cb_b, lut_cr_r, lut_cr_g, lut_cr_b) = create_lut_rgb2ycbcr(r, g, b);
    let (_, _, c) = img.shape();
    assert_eq!(c, Some(3));
    let len = img.get_len();
    let ptr = img.get_mut_ptr::<u8>().unwrap();
    unsafe {
        let mut i = 0;
        while i + 2 < len {
            let rr = *ptr.add(i) as usize;
            let gg = *ptr.add(i + 1) as usize;
            let bb = *ptr.add(i + 2) as usize;
            *ptr.add(i) = lut_y_r[rr] + lut_y_g[gg] + lut_y_b[bb];
            *ptr.add(i + 1) = 128u8 - lut_cb_r[rr] + lut_cb_b[bb] - lut_cb_g[gg];
            *ptr.add(i + 2) = 128u8 - lut_cr_g[gg] + lut_cr_r[rr] - lut_cr_b[bb];

            i += 3;
        }
    }
}

pub fn ycbcr_to_rgb_u8(img: &mut SVec, r: f32, g: f32, b: f32) {
    let (lut_r_cr, lut_g_cb, lut_g_cr, lut_b_cb) = create_lut_ycbcr2rgb(r, g, b);
    let (_, _, c) = img.shape();
    assert_eq!(c, Some(3));
    let len = img.get_len();
    let ptr = img.get_mut_ptr::<u8>().unwrap();
    unsafe {
        let mut i = 0;
        while i + 2 < len {
            let y = *ptr.add(i) as i16;
            let cb = *ptr.add(i + 1) as usize;
            let cr = *ptr.add(i + 2) as usize;

            *ptr.add(i) = (y + lut_r_cr[cr]).max(0) as u8;
            *ptr.add(i + 1) = (y + lut_g_cb[cb] + lut_g_cr[cr]).max(0) as u8;
            *ptr.add(i + 2) = (y + lut_b_cb[cb]).max(0) as u8;
            i += 3;
        }
    }
}

pub fn rgb_to_cmyk_u8(img: &mut SVec) {
    let (height, width, c) = img.shape();
    assert_eq!(c, Some(3));
    img.shape = Shape::new(height, width, Some(4));
    let num_pixels = height * width;
    let mut buff_vec: Vec<u8> = Vec::with_capacity(num_pixels * 4);

    let img_data = img.get_data::<u8>().unwrap();
    unsafe {
        for index in 0..num_pixels {
            let triple_index = index * 3;
            let r_s = 255u8.unchecked_sub(img_data[triple_index]) as f32 / 255.0;
            let g_s = 255u8.unchecked_sub(img_data[triple_index + 1]) as f32 / 255.0;
            let b_s = 255u8.unchecked_sub(img_data[triple_index + 2]) as f32 / 255.0;
            if r_s == 1.0 && g_s == 1.0 && b_s == 1.0 {
                buff_vec.extend([0, 0, 0, 255]);
                continue;
            }
            let min_cmy = r_s.min(g_s).min(b_s);
            let inv_cmy = 1.0 - min_cmy;
            buff_vec.extend([
                ((r_s - min_cmy) / inv_cmy * 255.0) as u8,
                ((g_s - min_cmy) / inv_cmy * 255.0) as u8,
                ((b_s - min_cmy) / inv_cmy * 255.0) as u8,
                (min_cmy * 255.0) as u8,
            ])
        }
    }

    img.data = ImgData::U8(buff_vec);
}

pub fn cmyk_to_rgb_u8(img: &mut SVec) {
    let (height, width, c) = img.shape();
    assert_eq!(c, Some(4));
    img.shape = Shape::new(height, width, Some(3));
    let num_pixels = height * width;
    let mut_img = img.get_data_mut::<u8>().unwrap();
    for index in 0..num_pixels {
        let img_index = index * 4;
        let k = (255 - mut_img[img_index + 3]) as f32;
        let rgb_index = index * 3;
        if k == 0.0 {
            mut_img[rgb_index] = 0;
            mut_img[rgb_index + 1] = 0;
            mut_img[rgb_index + 2] = 0;
            continue;
        }
        mut_img[rgb_index] = (((255 - mut_img[img_index]) as f32 / 255.0) * k) as u8;
        mut_img[rgb_index + 1] = (((255 - mut_img[img_index + 1]) as f32 / 255.0) * k) as u8;
        mut_img[rgb_index + 2] = (((255 - mut_img[img_index + 2]) as f32 / 255.0) * k) as u8;
    }
    img.truncate(num_pixels * 3).unwrap()
}
