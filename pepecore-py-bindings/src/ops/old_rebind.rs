use numpy::PyReadonlyArray2;
use numpy::ndarray::s;
use pyo3::{PyResult, pyfunction};

#[pyfunction]
pub fn best_tile(input: PyReadonlyArray2<f32>, tile_size: usize) -> PyResult<(usize, usize)> {
    let laplacian_abs = input.as_array().to_owned();
    let img_shape = laplacian_abs.dim();
    let tile_area = (tile_size * tile_size) as f32;

    let mut mean_intensity = laplacian_abs.slice(s![0..tile_size, 0..tile_size]).mean().unwrap();
    let mut right = true;
    let mut best_tile = [mean_intensity, 0f32, 0f32];
    for row in 0..(img_shape.0 - tile_size) {
        if right {
            for col in 0..(img_shape.1 - tile_size) {
                let mean_left = laplacian_abs.slice(s![row..(tile_size + row), col..(col + 1)]).sum();
                let mean_right = laplacian_abs
                    .slice(s![row..(tile_size + row), (tile_size + col)..(tile_size + col + 1)])
                    .sum();
                mean_intensity -= (mean_left - mean_right) / tile_area;
                if best_tile[0] < mean_intensity {
                    best_tile[0] = mean_intensity;
                    best_tile[1] = row as f32;
                    best_tile[2] = col as f32;
                }
            }
            let col = img_shape.1 - tile_size;
            let mean_up = laplacian_abs.slice(s![row, col..(col + tile_size)]).sum();
            let mean_down = laplacian_abs.slice(s![tile_size + row, col..(col + tile_size)]).sum();
            mean_intensity -= (mean_up - mean_down) / tile_area;
            if best_tile[0] < mean_intensity {
                best_tile[0] = mean_intensity;
                best_tile[1] = row as f32 + 1.0;
                best_tile[2] = col as f32;
            }
            right = false;
        } else {
            for col in 0..(img_shape.1 - tile_size) {
                let mean_right = laplacian_abs
                    .slice(s![
                        row..(tile_size + row),
                        img_shape.1 - (tile_size + col) - 1..img_shape.1 - tile_size - col
                    ])
                    .sum();
                let mean_left = laplacian_abs
                    .slice(s![row..(tile_size + row), img_shape.1 - col - 1..img_shape.1 - col])
                    .sum();
                mean_intensity -= (mean_left - mean_right) / tile_area;
                if best_tile[0] < mean_intensity {
                    best_tile[0] = mean_intensity;
                    best_tile[1] = row as f32;
                    best_tile[2] = (img_shape.1 - col - tile_size) as f32;
                }
            }
            let mean_up = laplacian_abs.slice(s![row, 0..tile_size]).sum();
            let mean_down = laplacian_abs.slice(s![tile_size + row, 0..tile_size]).sum();
            mean_intensity -= (mean_up - mean_down) / tile_area;
            if best_tile[0] < mean_intensity {
                best_tile[0] = mean_intensity;
                best_tile[1] = row as f32 + 1.0;
                best_tile[2] = 0.0;
            }
            right = true;
        }
    }
    Ok((best_tile[1] as usize, best_tile[2] as usize))
}
