use crate::structure::enums::TypeNoise;
use noise::{NoiseFn, OpenSimplex, Perlin, PerlinSurflet, Simplex, SuperSimplex};
use numpy::ndarray::{Array2, Array3, s};
use numpy::{PyArrayDyn, PyReadonlyArray2, ToPyArray};
use pyo3::{Py, PyResult, Python, pyfunction};
use rand::Rng;

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

pub fn noise_2d<T>(noise_fn: &T, x: usize, y: usize, octaves: u8, frequency: f64, lacunarity: f64) -> f32
where
    T: NoiseFn<f64, 2>,
{
    let mut total = 0.0;
    let mut frequency = frequency;
    let mut amplitude = 1.0;
    let mut max_amplitude = 0.0;

    for _ in 0..octaves {
        let val = noise_fn.get([x as f64 * frequency, y as f64 * frequency]);

        total += val * amplitude;
        max_amplitude += amplitude;
        frequency *= lacunarity;
        amplitude /= 2.0;
    }

    total as f32 / max_amplitude as f32
}

pub fn noise_3d<T>(perlin: &T, x: usize, y: usize, z: usize, octaves: u8, frequency: f64, lacunarity: f64) -> f32
where
    T: NoiseFn<f64, 3>,
{
    let mut total = 0.0;
    let mut frequency = frequency;
    let mut amplitude = 1.0;
    let mut max_amplitude = 0.0;

    for _ in 0..octaves {
        let val = perlin.get([x as f64 * frequency, y as f64 * frequency, z as f64 * frequency]);

        total += val * amplitude;
        max_amplitude += amplitude;
        frequency *= lacunarity;
        amplitude /= 2.0;
    }

    total as f32 / max_amplitude as f32
}

fn generate_noise2d(type_noise: TypeNoise, seed: u32) -> Box<dyn NoiseFn<f64, 2>> {
    match type_noise {
        TypeNoise::PERLIN => Box::new(Perlin::new(seed)),
        TypeNoise::SIMPLEX => Box::new(Simplex::new(seed)),
        TypeNoise::OPENSIMPLEX => Box::new(OpenSimplex::new(seed)),
        TypeNoise::SUPERSIMPLEX => Box::new(SuperSimplex::new(seed)),
        TypeNoise::PERLINSURFLET => Box::new(PerlinSurflet::new(seed)),
    }
}

fn generate_noise3d(type_noise: TypeNoise, seed: u32) -> Box<dyn NoiseFn<f64, 3>> {
    match type_noise {
        TypeNoise::PERLIN => Box::new(Perlin::new(seed)),
        TypeNoise::SIMPLEX => Box::new(Simplex::new(seed)),
        TypeNoise::OPENSIMPLEX => Box::new(OpenSimplex::new(seed)),
        TypeNoise::SUPERSIMPLEX => Box::new(SuperSimplex::new(seed)),
        TypeNoise::PERLINSURFLET => Box::new(PerlinSurflet::new(seed)),
    }
}

#[pyfunction]
pub fn noise_generate<'py>(
    size: Vec<usize>,
    type_noise: TypeNoise,
    octaves: u8,
    frequency: f64,
    lacunarity: f64,
    seed: Option<u32>,
    py: Python,
) -> PyResult<Py<PyArrayDyn<f32>>> {
    let seed = seed.unwrap_or(rand::rng().random_range(1..=10000) as u32);
    match size.len() {
        2 => {
            let mut array: Array2<f32> = Array2::zeros((size[0], size[1]));
            let type_fn = generate_noise2d(type_noise, seed);
            for ((x, y), value) in array.indexed_iter_mut() {
                *value = noise_2d(&type_fn, x, y, octaves, frequency, lacunarity);
            }
            Ok(array.into_dyn().to_pyarray(py).into())
        }
        3 => {
            let mut array: Array3<f32> = Array3::zeros((size[0], size[1], size[2]));
            let type_fn = generate_noise3d(type_noise, seed);
            for ((x, y, z), value) in array.indexed_iter_mut() {
                *value = noise_3d(&type_fn, x, y, z, octaves, frequency, lacunarity);
            }
            Ok(array.into_dyn().to_pyarray(py).into())
        }
        _ => Err(pyo3::exceptions::PyValueError::new_err("Unsupported dimensions")),
    }
}
