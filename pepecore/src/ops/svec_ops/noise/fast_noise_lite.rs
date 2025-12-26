use biski64::Biski64Rng;
use fastnoise_lite::*;
use pepecore_array::{ImgData, SVec, Shape};
use rand::{RngCore, SeedableRng};

pub fn create_noise_2d(shape: Shape, octaves: usize, amplitudes: &[f32], frequency: &[f32], noise_types: &[NoiseType]) -> SVec {
    let (h, w, c) = shape.get_shape();
    let mut data: Vec<f32> = vec![0.0; h * w * c.unwrap_or(1)];
    let mut noise_func = vec![];
    let mut rng = Biski64Rng::from_os_rng();
    let mut amp: Vec<f32> = Vec::with_capacity(octaves);
    let mut norm = 0.0;
    for index in 0..octaves {
        let mut noise = FastNoiseLite::new();
        noise.set_seed(Some(rng.next_u32() as i32));
        noise.set_frequency(Some(frequency[index % frequency.len()]));
        noise.set_noise_type(Some(noise_types[index % noise_types.len()]));
        noise_func.push(noise);
        norm += amplitudes[index % amplitudes.len()];
        amp.push(amplitudes[index % amplitudes.len()]);
    }
    amp.iter_mut().for_each(|value| *value /= norm);

    for (noise, amplitude) in noise_func.iter().zip(amp.iter()) {
        for y in 0..h {
            let index = y * w;
            let y = y as f32;
            for x in 0..w {
                data[index + x] += noise.get_noise_2d(x as f32, y) * amplitude;
            }
        }
    }
    SVec::new(shape, ImgData::F32(data))
}
pub fn create_noise_3d(shape: Shape, octaves: usize, amplitudes: &[f32], frequency: &[f32], noise_types: &[NoiseType]) -> SVec {
    let (h, w, c) = shape.get_shape();
    let c = c.unwrap_or(1);
    let mut data: Vec<f32> = vec![0.0; h * w * c];
    let mut noise_func = vec![];
    let mut rng = Biski64Rng::from_os_rng();
    let mut amp: Vec<f32> = Vec::with_capacity(octaves);
    let mut norm = 0.0;
    for ch in 0..c {
        for index in 0..octaves {
            let index = octaves * ch + index;
            let mut noise = FastNoiseLite::new();
            noise.set_seed(Some(rng.next_u32() as i32));
            noise.set_frequency(Some(frequency[index % frequency.len()]));
            noise.set_noise_type(Some(noise_types[index % noise_types.len()]));
            noise_func.push(noise);
            norm += amplitudes[index % amplitudes.len()];
            amp.push(amplitudes[index % amplitudes.len()]);
        }
    }
    amp.iter_mut().for_each(|value| *value /= norm);
    for (index, (noise, amplitude)) in noise_func.iter().zip(amp.iter()).enumerate() {
        let ch = index % c;
        for y in 0..h {
            let index = y * w;
            let y = y as f32;
            for x in 0..w {
                data[(index + x) * c + ch] += noise.get_noise_2d(x as f32, y) * amplitude;
            }
        }
    }
    SVec::new(shape, ImgData::F32(data))
}
