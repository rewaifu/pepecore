pub fn f32_color_level(img_vec: &mut [f32], in_low: f32, in_high: f32, out_low: f32, out_high: f32, gamma: f32) {
    let in_range = in_high - in_low;
    let out_range = out_high - out_low;

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
