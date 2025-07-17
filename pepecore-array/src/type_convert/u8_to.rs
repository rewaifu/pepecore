use crate::{ImgData, SVec};
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn convert_simd_avx2_normalized_u8_to_f32(input: &mut SVec) {
    let len = input.get_len();
    let mut output = Vec::with_capacity(len);

    let mut i = 0;
    let chunks = len / 32;

    let in_ptr: *const u8 = input.get_data::<u8>().unwrap().as_ptr();
    let out_ptr: *mut f32 = output.as_mut_ptr();
    unsafe {
        let max_val = _mm256_set1_ps(255.0);

        while i < chunks * 32 {
            // Загружаем 32 байта
            let bytes = _mm256_loadu_si256(in_ptr.add(i) as *const __m256i);

            // Распаковываем низшие 16 байт и старшие 16 байт в два __m256i с 16 u16
            let lo_u16 = _mm256_cvtepu8_epi16(_mm256_castsi256_si128(bytes));
            let hi_u16 = _mm256_cvtepu8_epi16(_mm256_extracti128_si256(bytes, 1));

            // Каждое __m256i (16 × u16) разбиваем на две __m128i (по 8 × u16)
            let lo_lo = _mm256_castsi256_si128(lo_u16);
            let lo_hi = _mm256_extracti128_si256(lo_u16, 1);
            let hi_lo = _mm256_castsi256_si128(hi_u16);
            let hi_hi = _mm256_extracti128_si256(hi_u16, 1);

            // Преобразуем каждую половину (8 × u16) в 8 × i32 → __m256i
            let lo32_0 = _mm256_cvtepu16_epi32(lo_lo);
            let lo32_1 = _mm256_cvtepu16_epi32(lo_hi);
            let hi32_0 = _mm256_cvtepu16_epi32(hi_lo);
            let hi32_1 = _mm256_cvtepu16_epi32(hi_hi);

            // Конвертируем в f32 и нормируем сразу же «дроблением»
            let v0 = _mm256_div_ps(_mm256_cvtepi32_ps(lo32_0), max_val);
            let v1 = _mm256_div_ps(_mm256_cvtepi32_ps(lo32_1), max_val);
            let v2 = _mm256_div_ps(_mm256_cvtepi32_ps(hi32_0), max_val);
            let v3 = _mm256_div_ps(_mm256_cvtepi32_ps(hi32_1), max_val);

            // Сохраняем подряд 32 значения f32
            _mm256_storeu_ps(out_ptr.add(i) as *mut f32, v0);
            _mm256_storeu_ps(out_ptr.add(i + 8) as *mut f32, v1);
            _mm256_storeu_ps(out_ptr.add(i + 16) as *mut f32, v2);
            _mm256_storeu_ps(out_ptr.add(i + 24) as *mut f32, v3);

            i += 32;
        }

        while i < len {
            *out_ptr.add(i) = *in_ptr.add(i) as f32 / 255.0;
            i += 1;
        }

        output.set_len(len);
    }
    input.data = ImgData::F32(output)
}

fn convert_fallback_normalized_u8_to_f32(input: &mut SVec) {
    let len_img = input.get_len();
    let mut out = Vec::with_capacity(len_img);
    unsafe {
        let in_ptr: *const u8 = input.get_data::<u8>().unwrap().as_ptr();
        let out_ptr: *mut f32 = out.as_mut_ptr();
        for i in 0..len_img {
            *out_ptr.add(i) = *in_ptr.add(i) as f32 / 255.0;
        }
        out.set_len(len_img);
    }
    input.data = ImgData::F32(out)
}

pub fn convert_u8_to_f32_normalized(input: &mut SVec) {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") {
            unsafe {
                return convert_simd_avx2_normalized_u8_to_f32(input);
            }
        }
    }
    convert_fallback_normalized_u8_to_f32(input)
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn convert_simd_avx2_u8_to_u16_normalized(input: &mut SVec) {
    let len = input.get_len();
    let mut output = Vec::with_capacity(len);
    let mut i = 0;
    let chunks = len / 32; // 32 байта за итерацию

    let in_ptr: *const u8 = input.get_data::<u8>().unwrap().as_ptr();
    let out_ptr: *mut u16 = output.as_mut_ptr();
    // коэффициент масштабирования: 65535 / 255 = 257
    unsafe {
        let factor = _mm256_set1_epi16(257_i16);

        while i < chunks * 32 {
            // загрузка 32 u8
            let bytes = _mm256_loadu_si256(in_ptr.add(i) as *const __m256i);

            // распаковка в два вектора по 16 × u16
            let lo = _mm256_cvtepu8_epi16(_mm256_castsi256_si128(bytes));
            let hi = _mm256_cvtepu8_epi16(_mm256_extracti128_si256(bytes, 1));

            // масштабирование (каждое u16 умножаем на 257)
            let lo_scaled = _mm256_mullo_epi16(lo, factor);
            let hi_scaled = _mm256_mullo_epi16(hi, factor);

            // сохраняем по 16 слов
            _mm256_storeu_si256(out_ptr.add(i) as *mut __m256i, lo_scaled);
            _mm256_storeu_si256(out_ptr.add(i + 16) as *mut __m256i, hi_scaled);

            i += 32;
        }
        // хвостовой остаток
        while i < len {
            let v = *in_ptr.add(i) as u16;
            *out_ptr.add(i) = v.wrapping_mul(257); // безопасно, т.к. результат ≤ 65535
            i += 1;
        }
        output.set_len(len);
    }
    input.data = ImgData::U16(output);
}

fn convert_fallback_u8_to_u16_normalized(input: &mut SVec) {
    let len = input.get_len();
    let mut out = Vec::with_capacity(len);
    unsafe {
        let in_ptr: *const u8 = input.get_data::<u8>().unwrap().as_ptr();
        let out_ptr: *mut u16 = out.as_mut_ptr();
        for i in 0..len {
            let v = *in_ptr.add(i) as u16;
            *out_ptr.add(i) = v.wrapping_mul(257);
        }
        out.set_len(len);
    }
    input.data = ImgData::U16(out);
}

pub fn convert_u8_to_u16_normalized(input: &mut SVec) {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") {
            unsafe {
                return convert_simd_avx2_u8_to_u16_normalized(input);
            }
        }
    }
    convert_fallback_u8_to_u16_normalized(input)
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ImgData, Shape};

    #[test]
    fn u8_to_u16_test() {
        let mut img = SVec::new(
            Shape::new(3, 3, None),
            ImgData::U8(vec![1, 255, 128, 90, 11, 16, 60, 100, 121]),
        );
        convert_u8_to_u16_normalized(&mut img);
        assert_eq!(
            img.get_data::<u16>().unwrap().to_vec(),
            vec![257, 65535, 32896, 23130, 2827, 4112, 15420, 25700, 31097]
        );
    }
    #[test]
    fn u8_to_f32_test() {
        let mut img = SVec::new(
            Shape::new(3, 3, None),
            ImgData::U8(vec![1, 255, 128, 90, 11, 16, 60, 100, 121]),
        );
        convert_u8_to_f32_normalized(&mut img);
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
