use crate::{ImgData, SVec};
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn convert_simd_avx2_u16_to_f32(input: &mut SVec) {
    let len = input.get_len();
    let mut output = Vec::with_capacity(len);
    let mut i = 0;
    let chunks = len / 16; // 16 u16 → 16 f32

    let in_ptr: *const u16 = input.get_data::<u16>().unwrap().as_ptr();
    let out_ptr: *mut f32 = output.as_mut_ptr();
    unsafe {
        let div = _mm256_set1_ps(65535.0);

        while i < chunks * 16 {
            let chunk = _mm256_loadu_si256(in_ptr.add(i) as *const __m256i);
            let lo = _mm256_cvtepu16_epi32(_mm256_castsi256_si128(chunk));
            let hi = _mm256_cvtepu16_epi32(_mm256_extracti128_si256(chunk, 1));

            let v0 = _mm256_div_ps(_mm256_cvtepi32_ps(lo), div);
            let v1 = _mm256_div_ps(_mm256_cvtepi32_ps(hi), div);

            _mm256_storeu_ps(out_ptr.add(i), v0);
            _mm256_storeu_ps(out_ptr.add(i + 8), v1);
            i += 16;
        }

        while i < len {
            *out_ptr.add(i) = *in_ptr.add(i) as f32 / 65535.0;
            i += 1;
        }

        output.set_len(len);
    }
    input.data = ImgData::F32(output);
}

fn convert_fallback_u16_to_f32(input: &mut SVec) {
    let len = input.get_len();
    let mut out = Vec::with_capacity(len);
    unsafe {
        let ip: *const u16 = input.get_data::<u16>().unwrap().as_ptr();
        let op: *mut f32 = out.as_mut_ptr();
        for i in 0..len {
            *op.add(i) = *ip.add(i) as f32 / 65535.0;
        }
        out.set_len(len);
    }
    input.data = ImgData::F32(out);
}

pub fn convert_u16_to_f32_normalized(input: &mut SVec) {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") {
            unsafe {
                return convert_simd_avx2_u16_to_f32(input);
            }
        }
    }
    convert_fallback_u16_to_f32(input);
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn convert_simd_avx2_u16_to_u8_normalized(input: &mut SVec) {
    let len = input.get_len();
    let mut output = Vec::with_capacity(len);
    let mut i = 0;
    let chunks = len / 32;

    let in_ptr: *const u16 = input.get_data::<u16>().unwrap().as_ptr();
    let out_ptr: *mut u8 = output.as_mut_ptr();
    unsafe {
        let inv = _mm256_set1_ps(1.0_f32 / 257.0);

        while i < chunks * 32 {
            // читаем по 16 u16 в два регистра
            let v0 = _mm256_loadu_si256(in_ptr.add(i) as *const __m256i);
            let v1 = _mm256_loadu_si256(in_ptr.add(i + 16) as *const __m256i);

            // расширяем каждую половину на 32-бит, чтобы конвертить в f32
            let a0 = _mm256_cvtepu16_epi32(_mm256_castsi256_si128(v0));
            //noinspection E0061
            let a1 = _mm256_cvtepu16_epi32(_mm256_extracti128_si256(v0, 1));
            let a2 = _mm256_cvtepu16_epi32(_mm256_castsi256_si128(v1));
            let a3 = _mm256_cvtepu16_epi32(_mm256_extracti128_si256(v1, 1));

            // конвертация в f32 и нормализация
            let f0 = _mm256_mul_ps(_mm256_cvtepi32_ps(a0), inv);
            let f1 = _mm256_mul_ps(_mm256_cvtepi32_ps(a1), inv);
            let f2 = _mm256_mul_ps(_mm256_cvtepi32_ps(a2), inv);
            let f3 = _mm256_mul_ps(_mm256_cvtepi32_ps(a3), inv);

            // обратно в целые (с округлением к ближайшему)
            let i0 = _mm256_cvtps_epi32(f0);
            let i1 = _mm256_cvtps_epi32(f1);
            let i2 = _mm256_cvtps_epi32(f2);
            let i3 = _mm256_cvtps_epi32(f3);

            // упакуем по 32-бит → 16-бит
            let p0 = _mm256_packs_epi32(i0, i1); // 16×i16
            let p1 = _mm256_packs_epi32(i2, i3); // 16×i16

            // затем 16-бит → 8-бит с насыщением
            let packed = _mm256_packus_epi16(p0, p1); // 32×u8

            // сохраняем 32 байта
            _mm256_storeu_si256(out_ptr.add(i) as *mut __m256i, packed);

            i += 32;
        }

        // остаток «на руках»
        while i < len {
            *out_ptr.add(i) = (*in_ptr.add(i) / 257) as u8;
            i += 1;
        }

        output.set_len(len);
    }
    input.data = ImgData::U8(output);
}

fn convert_fallback_u16_to_u8_normalized(input: &mut SVec) {
    let len = input.get_len();
    let mut out = Vec::with_capacity(len);
    unsafe {
        let in_ptr: *const u16 = input.get_data::<u16>().unwrap().as_ptr();
        let out_ptr: *mut u8 = out.as_mut_ptr();
        for i in 0..len {
            let v = *in_ptr.add(i);
            out_ptr.add(i).write((v / 257) as u8);
        }
        out.set_len(len);
    }
    input.data = ImgData::U8(out);
}

pub fn convert_u16_to_u8_normalized(input: &mut SVec) {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") {
            unsafe {
                return convert_simd_avx2_u16_to_u8_normalized(input);
            }
        }
    }
    convert_fallback_u16_to_u8_normalized(input)
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ImgData, Shape};

    #[test]
    fn u16_to_u8_test() {
        let mut img = SVec::new(
            Shape::new(3, 3, None),
            ImgData::U16(vec![257, 65535, 32896, 23130, 2827, 4112, 15420, 25700, 31097]),
        );
        convert_u16_to_u8_normalized(&mut img);
        assert_eq!(
            img.get_data::<u8>().unwrap().to_vec(),
            vec![1, 255, 128, 90, 11, 16, 60, 100, 121]
        );
    }
    #[test]
    fn u16_to_f32_test() {
        let mut img = SVec::new(
            Shape::new(3, 3, None),
            ImgData::U16(vec![257, 65535, 32896, 23130, 2827, 4112, 15420, 25700, 31097]),
        );
        convert_u16_to_f32_normalized(&mut img);
        assert_eq!(
            img.get_data::<f32>().unwrap().to_vec(),
            vec![
                0.003921569,
                1.0,
                0.5019608,
                0.3529412,
                0.043137256,
                0.0627451,
                0.23529412,
                0.39215687,
                0.4745098
            ]
        );
    }
}
