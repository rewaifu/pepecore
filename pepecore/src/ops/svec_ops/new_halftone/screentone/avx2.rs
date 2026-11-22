use core::arch::x86_64::*;

#[target_feature(enable = "avx2")]
unsafe fn round_i32(v: __m256) -> __m256i {
    // Truncate first: adding 0.5 to v can round a near-half up prematurely.
    let v = _mm256_and_ps(v, _mm256_cmp_ps::<_CMP_ORD_Q>(v, v));
    let v = _mm256_max_ps(v, _mm256_set1_ps(i32::MIN as f32));
    let overflow = _mm256_castps_si256(_mm256_cmp_ps::<_CMP_GE_OQ>(v, _mm256_set1_ps(2147483648.0)));
    let i = _mm256_cvttps_epi32(v);
    let frac = _mm256_sub_ps(v, _mm256_cvtepi32_ps(i));
    let up = _mm256_castps_si256(_mm256_cmp_ps::<_CMP_GE_OQ>(frac, _mm256_set1_ps(0.5)));
    let down = _mm256_castps_si256(_mm256_cmp_ps::<_CMP_LE_OQ>(frac, _mm256_set1_ps(-0.5)));
    let rounded = _mm256_add_epi32(_mm256_sub_epi32(i, up), down);
    _mm256_blendv_epi8(rounded, _mm256_set1_epi32(i32::MAX), overflow)
}

#[target_feature(enable = "avx2")]
unsafe fn rem_euclid(v: __m256i, p: __m256i, reciprocal: __m256i) -> __m256i {
    let mag = _mm256_abs_epi32(v); // i32::MIN has unsigned magnitude 2^31.
    let even = _mm256_mul_epu32(mag, reciprocal);
    let odd = _mm256_mul_epu32(_mm256_srli_epi64::<32>(mag), _mm256_srli_epi64::<32>(reciprocal));
    let q = _mm256_blend_epi32::<0xaa>(_mm256_srli_epi64::<32>(even), odd);
    // floor(2^32 / p) underestimates the quotient by at most one.
    let r = _mm256_sub_epi32(mag, _mm256_mullo_epi32(q, p));
    let r = _mm256_min_epu32(r, _mm256_sub_epi32(r, p));
    let negative_nonzero = _mm256_andnot_si256(_mm256_cmpeq_epi32(r, _mm256_setzero_si256()), _mm256_srai_epi32::<31>(v));
    _mm256_blendv_epi8(r, _mm256_sub_epi32(p, r), negative_nonzero)
}

/// # Safety
/// Requires AVX2. In every lane (including skipped lanes), `period >= 2`,
/// `reciprocal == floor(2^32 / period)`, and `[offset, offset + period²)`
/// must be within `atlas` with its end representable in i32. `y + by` must fit i32.
#[target_feature(enable = "avx2")]
pub(super) unsafe fn render(l: &super::Lanes, y: i32, cx: f32, cy: f32, atlas: &[f32], dst: &mut [u8; 8]) {
    unsafe {
        let dx = _mm256_loadu_ps(l.dx.as_ptr());
        let by = _mm256_loadu_si256(l.by.as_ptr().cast());
        let cx = _mm256_set1_ps(cx);
        let cy = _mm256_set1_ps(cy);
        let dy = _mm256_sub_ps(_mm256_cvtepi32_ps(_mm256_add_epi32(_mm256_set1_epi32(y), by)), cy);
        let sin = _mm256_loadu_ps(l.sin.as_ptr());
        let cos = _mm256_loadu_ps(l.cos.as_ptr());
        let rx = _mm256_add_ps(_mm256_sub_ps(_mm256_mul_ps(cos, dx), _mm256_mul_ps(sin, dy)), cx);
        let ry = _mm256_add_ps(_mm256_add_ps(_mm256_mul_ps(sin, dx), _mm256_mul_ps(cos, dy)), cy);
        let p = _mm256_loadu_si256(l.period.as_ptr().cast());
        let reciprocal = _mm256_loadu_si256(l.reciprocal.as_ptr().cast());
        let col = rem_euclid(round_i32(rx), p, reciprocal);
        let row = rem_euclid(round_i32(ry), p, reciprocal);
        let offset = _mm256_loadu_si256(l.offset.as_ptr().cast());
        let index = _mm256_add_epi32(offset, _mm256_add_epi32(col, _mm256_mullo_epi32(row, p)));
        // Both residues are below p, so each index lies in its caller-validated table.
        let threshold = _mm256_i32gather_ps::<4>(atlas.as_ptr(), index);
        let dark = _mm256_cmp_ps::<_CMP_LT_OQ>(_mm256_loadu_ps(l.tone.as_ptr()), threshold);
        let binary = _mm256_andnot_si256(_mm256_castps_si256(dark), _mm256_set1_epi32(255));
        let skip = _mm256_loadu_si256(l.skip.as_ptr().cast());
        let pixels = _mm256_blendv_epi8(skip, binary, _mm256_srai_epi32::<31>(skip));
        let words = _mm_packs_epi32(_mm256_castsi256_si128(pixels), _mm256_extracti128_si256::<1>(pixels));
        _mm_storel_epi64(dst.as_mut_ptr().cast(), _mm_packus_epi16(words, words));
    }
}
