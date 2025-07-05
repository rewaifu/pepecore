pub fn create_lut_rgb2ycbcr(
    r: f32,
    g: f32,
    b: f32,
) -> (
    [u8; 256], // y_r
    [u8; 256], // y_g
    [u8; 256], // y_b
    [u8; 256], // cb_r  ⟵ cr0 * R
    [u8; 256], // cb_g  ⟵ cr1 * G
    [u8; 256], // cb_b  ⟵ 0.5 * B
    [u8; 256], // cr_r  ⟵ 0.5 * R
    [u8; 256], // cr_g  ⟵ cb0 * G
    [u8; 256], // cr_b  ⟵ cb1 * B
) {
    let cr0 = r / (2.0 * (1.0 - b));
    let cr1 = g / (2.0 * (1.0 - b));
    let cb0 = g / (2.0 * (1.0 - r));
    let cb1 = b / (2.0 * (1.0 - r));

    let mut lut_y_r = [0_u8; 256];
    let mut lut_y_g = [0_u8; 256];
    let mut lut_y_b = [0_u8; 256];
    let mut lut_cb_r = [0_u8; 256];
    let mut lut_cb_g = [0_u8; 256];
    let mut lut_cb_b = [0_u8; 256];
    let mut lut_cr_r = [0_u8; 256];
    let mut lut_cr_g = [0_u8; 256];
    let mut lut_cr_b = [0_u8; 256];

    for i in 0..256 {
        let f = i as f32;
        // Y-коэффициенты
        lut_y_r[i] = (r * f) as u8;
        lut_y_g[i] = (g * f) as u8;
        lut_y_b[i] = (b * f) as u8;

        // Cb:   cr0*R + cr1*G + 0.5*B
        lut_cb_r[i] = (cr0 * f).round() as u8;
        lut_cb_g[i] = (cr1 * f).round() as u8;
        lut_cb_b[i] = (0.5 * f).round() as u8;

        // Cr:   0.5*R + cb0*G + cb1*B
        lut_cr_r[i] = (0.5 * f).round() as u8;
        lut_cr_g[i] = (cb0 * f).round() as u8;
        lut_cr_b[i] = (cb1 * f).round() as u8;
    }

    (
        lut_y_r, lut_y_g, lut_y_b, lut_cb_r, lut_cb_g, lut_cb_b, lut_cr_r, lut_cr_g, lut_cr_b,
    )
}

pub fn create_lut_ycbcr2rgb(r: f32, g: f32, b: f32) -> ([i16; 256], [i16; 256], [i16; 256], [i16; 256]) {
    let r_cr = 2.0 * (1.0 - r);
    let g_cb = -(r / g) * (2.0 * (1.0 - r));
    let g_cr = -(r / g) * (2.0 * (1.0 - b));
    let b_cb = 2.0 * (1.0 - b);

    let mut lut_r_cr = [0_i16; 256];
    let mut lut_g_cb = [0_i16; 256];
    let mut lut_g_cr = [0_i16; 256];
    let mut lut_b_cb = [0_i16; 256];

    for i in 0..256 {
        let f = i as f32 - 128.0;

        lut_r_cr[i] = ((f) * r_cr) as i16;
        lut_g_cb[i] = ((f) * g_cb) as i16;
        lut_g_cr[i] = ((f) * g_cr) as i16;
        lut_b_cb[i] = ((f) * b_cb) as i16
    }

    (lut_r_cr, lut_g_cb, lut_g_cr, lut_b_cb)
}

pub fn create_lut_rgb2gray(r: f32, g: f32, b: f32) -> ([u8; 256], [u8; 256], [u8; 256]) {
    let mut lut_r = [0u8; 256];
    let mut lut_g = [0u8; 256];
    let mut lut_b = [0u8; 256];
    for i in 0..256 {
        lut_r[i] = (i as f32 * r).round() as u8;
        lut_g[i] = (i as f32 * g).round() as u8;
        lut_b[i] = (i as f32 * b).round() as u8;
    }
    (lut_r, lut_g, lut_b)
}
