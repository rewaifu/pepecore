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
