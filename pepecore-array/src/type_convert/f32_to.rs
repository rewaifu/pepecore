use crate::{ImgData, SVec};
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn convert_simd_avx2_f32_to_u8(input: &mut SVec) {
    let len = input.get_len();
    let mut output = Vec::with_capacity(len);
    let in_ptr: *const f32 = input.get_data::<f32>().unwrap().as_ptr();
    let out_ptr: *mut u8 = output.as_mut_ptr();
    let chunks = len / 8;
    unsafe {
        let mul = _mm256_set1_ps(255.0);
        let zero128: __m128i = _mm_setzero_si128();

        for i in 0..chunks {
            let v = _mm256_loadu_ps(in_ptr.add(8 * i));
            let vi = _mm256_cvtps_epi32(_mm256_mul_ps(v, mul));
            let vi_lo = _mm256_castsi256_si128(vi);
            let vi_hi = _mm256_extracti128_si256(vi, 1);
            let pack16 = _mm_packs_epi32(vi_lo, vi_hi);
            let pack8 = _mm_packus_epi16(pack16, zero128);
            _mm_storel_epi64(out_ptr.add(8 * i) as *mut __m128i, pack8);
        }
        for i in chunks * 8..len {
            *out_ptr.add(i) = (*in_ptr.add(i) * 255.0) as u8;
        }

        output.set_len(len);
    }
    input.data = ImgData::U8(output);
}

fn convert_fallback_f32_to_u8(input: &mut SVec) {
    let len = input.get_len();
    let mut out = Vec::with_capacity(len);
    unsafe {
        let ip: *const f32 = input.get_data::<f32>().unwrap().as_ptr();
        let op: *mut u8 = out.as_mut_ptr();
        for i in 0..len {
            *op.add(i) = (*ip.add(i) * 255.0) as u8;
        }
        out.set_len(len);
    }
    input.data = ImgData::U8(out);
}
pub fn convert_f32_to_u8_normalized(input: &mut SVec) {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") {
            unsafe {
                return convert_simd_avx2_f32_to_u8(input);
            }
        }
    }
    convert_fallback_f32_to_u8(input);
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn convert_simd_avx2_f32_to_u16(input: &mut SVec) {
    let len = input.get_len();
    let mut out = Vec::with_capacity(len);
    let ip: *const f32 = input.get_data::<f32>().unwrap().as_ptr();
    let op: *mut u16 = out.as_mut_ptr();
    let chunks = len / 8;
    unsafe {
        let mul = _mm256_set1_ps(65535.0);
        let bias32 = _mm256_set1_epi32(32768);
        let xor16 = _mm_set1_epi16(0x8000u16 as i16);

        for i in 0..chunks {
            let v = _mm256_loadu_ps(ip.add(8 * i));
            let vi32 = _mm256_cvttps_epi32(_mm256_mul_ps(v, mul));

            let biased = _mm256_sub_epi32(vi32, bias32);

            let lo32 = _mm256_castsi256_si128(biased);
            let hi32 = _mm256_extracti128_si256(biased, 1);

            let packed16_128 = _mm_packs_epi32(lo32, hi32);

            let result16 = _mm_xor_si128(packed16_128, xor16);

            _mm_storeu_si128(op.add(8 * i) as *mut __m128i, result16);
        }

        // остаток скалярно
        for i in chunks * 8..len {
            *op.add(i) = (*ip.add(i) * 65535.0) as u16;
        }

        out.set_len(len);
    }
    input.data = ImgData::U16(out);
}

fn convert_fallback_f32_to_u16(input: &mut SVec) {
    let len = input.get_len();
    let mut out = Vec::with_capacity(len);
    unsafe {
        let ip: *const f32 = input.get_data::<f32>().unwrap().as_ptr();
        let op: *mut u16 = out.as_mut_ptr();
        for i in 0..len {
            *op.add(i) = (*ip.add(i) * 65535.0) as u16;
        }
        out.set_len(len);
    }
    input.data = ImgData::U16(out);
}

pub fn convert_f32_to_u16_normalized(input: &mut SVec) {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") {
            unsafe {
                return convert_simd_avx2_f32_to_u16(input);
            }
        }
    }
    convert_fallback_f32_to_u16(input);
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ImgData, Shape};

    #[test]
    fn f32_to_u16_test() {
        let mut img = SVec::new(
            Shape::new(3, 3, None),
            ImgData::F32(vec![
                0.003921569,
                1.0,
                0.5019608,
                0.3529412,
                0.043137256,
                0.0627451,
                0.23529412,
                0.39215687,
                0.4745098,
            ]),
        );
        convert_f32_to_u16_normalized(&mut img);
        assert_eq!(
            img.get_data::<u16>().unwrap().to_vec(),
            vec![257, 65535, 32896, 23130, 2827, 4112, 15420, 25700, 31097]
        );
    }
    #[test]
    fn f32_u8_test() {
        let mut img = SVec::new(
            Shape::new(3, 3, None),
            ImgData::F32(vec![
                0.003921569,
                1.0,
                0.5019608,
                0.3529412,
                0.043137256,
                0.0627451,
                0.23529412,
                0.39215687,
                0.4745098,
            ]),
        );
        convert_f32_to_u8_normalized(&mut img);
        assert_eq!(
            img.get_data::<u8>().unwrap().to_vec(),
            vec![1, 255, 128, 90, 11, 16, 60, 100, 121]
        );
    }
}
