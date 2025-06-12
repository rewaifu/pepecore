pub fn u8_color_level(img_vec: &mut [u8], in_low: u8, in_high: u8, out_low: u8, out_high: u8, gamma: f32) {
    let in_low = in_low as f32;
    let out_low = out_low as f32;
    let in_range = in_high as f32 - in_low;
    let out_range = out_high as f32 - out_low;

    if gamma == 1.0 {
        img_vec
            .iter_mut()
            .for_each(|i| *i = ((*i as f32 - in_low) / in_range * out_range + out_low).clamp(0.0, 255.0) as u8);
    } else {
        img_vec.iter_mut().for_each(|i| {
            *i = ((*i as f32 - in_low) / in_range * out_range + out_low)
                .clamp(0.0, 255.0)
                .powf(gamma) as u8
        });
    }
}

pub fn u16_color_level(img_vec: &mut [u16], in_low: u16, in_high: u16, out_low: u16, out_high: u16, gamma: f32) {
    let in_low = in_low as f32;
    let out_low = out_low as f32;
    let in_range = in_high as f32 - in_low;
    let out_range = out_high as f32 - out_low;
    let max = u16::MAX as f32;
    if gamma == 1.0 {
        img_vec
            .iter_mut()
            .for_each(|i| *i = ((*i as f32 - in_low) / in_range * out_range + out_low).clamp(0.0, max) as u16);
    } else {
        img_vec.iter_mut().for_each(|i| {
            *i = ((*i as f32 - in_low) / in_range * out_range + out_low)
                .clamp(0.0, max)
                .powf(gamma) as u16
        });
    }
}

pub fn f32_color_level(img_vec: &mut [f32], in_low: f32, in_high: f32, out_low: f32, out_high: f32, gamma: f32) {
    let in_range = in_high - in_low;
    let out_range = out_high - out_low;
    let in_low = in_low;
    let out_low = out_low;

    if gamma == 1.0 {
        img_vec
            .iter_mut()
            .for_each(|i| *i = ((*i - in_low) / in_range * out_range + out_low).clamp(0.0, 1.0));
    } else {
        img_vec
            .iter_mut()
            .for_each(|i| *i = ((*i - in_low) / in_range * out_range + out_low).clamp(0.0, 1.0).powf(gamma));
    }
}
