pub fn rotate_pixel_coordinates(x: f32, y: f32, center_y: f32, center_x: f32, cos_theta: f32, sin_theta: f32) -> (usize, usize) {
    let x_rel = x - center_x;
    let y_rel = y - center_y;

    let rotated_x = (cos_theta * x_rel - sin_theta * y_rel + center_x) as usize;
    let rotated_y = (sin_theta * x_rel + cos_theta * y_rel + center_y) as usize;

    (rotated_x, rotated_y)
}

pub fn compute_cos_sin(theta: f32) -> [f32; 2] {
    let cos_theta = theta.cos();
    let sin_theta = theta.sin();
    [cos_theta, sin_theta]
}
pub trait HalftonePixel: Sized + Copy + PartialOrd + 'static {
    const MIN_VALUE: Self;
    const MAX_VALUE: Self;

    fn prepare_dot_matrix(matrix: &[f32]) -> Vec<Self>;
}

impl HalftonePixel for f32 {
    const MIN_VALUE: Self = 0.0;
    const MAX_VALUE: Self = 1.0;

    fn prepare_dot_matrix(matrix: &[f32]) -> Vec<Self> {
        matrix.into()
    }
}

impl HalftonePixel for u8 {
    const MIN_VALUE: Self = u8::MIN;
    const MAX_VALUE: Self = u8::MAX;

    fn prepare_dot_matrix(matrix: &[f32]) -> Vec<Self> {
        let mut new_dot_matrix_data = Vec::with_capacity(matrix.len());

        unsafe {
            let src_ptr = matrix.as_ptr();
            let dst_ptr: *mut u8 = new_dot_matrix_data.as_mut_ptr();

            for i in 0..matrix.len() {
                *dst_ptr.add(i) = (*src_ptr.add(i) * 255.0).min(255.0) as u8;
            }

            new_dot_matrix_data.set_len(matrix.len());
        }

        new_dot_matrix_data
    }
}

impl HalftonePixel for u16 {
    const MIN_VALUE: Self = u16::MIN;
    const MAX_VALUE: Self = u16::MAX;

    fn prepare_dot_matrix(matrix: &[f32]) -> Vec<Self> {
        let mut new_dot_matrix_data = Vec::with_capacity(matrix.len());

        unsafe {
            let src_ptr = matrix.as_ptr();
            let dst_ptr: *mut u16 = new_dot_matrix_data.as_mut_ptr();

            for i in 0..matrix.len() {
                *dst_ptr.add(i) = (*src_ptr.add(i) * u16::MAX as f32).min(u16::MAX as f32) as u16;
            }

            new_dot_matrix_data.set_len(matrix.len());
        }

        new_dot_matrix_data
    }
}
