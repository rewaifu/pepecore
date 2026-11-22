use fast_image_resize::ResizeAlg;
use pepecore::ops::svec_ops::new_halftone::screentone::{DotPatternCache, apply_rotate_ssaa_screentone_masked_cached};
use pepecore_array::{ImgData, SVec, Shape};

#[test]
fn masked_screentone_matches_golden_bytes() {
    // Bytes captured from the original sequential cached renderer.
    let cases: &[(usize, usize, f32, bool, &[u8])] = &[
        (
            3,
            5,
            2.0,
            false,
            &[0, 0, 204, 255, 255, 255, 0, 166, 0, 255, 255, 0, 128, 255, 0],
        ),
        (
            3,
            5,
            1.8,
            true,
            &[255, 255, 204, 255, 255, 255, 255, 166, 0, 255, 255, 0, 128, 255, 0],
        ),
        (
            3,
            5,
            1.8,
            false,
            &[0, 0, 204, 0, 255, 255, 0, 166, 0, 255, 255, 0, 128, 255, 255],
        ),
        (
            3,
            5,
            0.65,
            false,
            &[255, 255, 166, 255, 255, 255, 255, 166, 255, 255, 255, 255, 166, 255, 255],
        ),
        // Original Chess size 1 compares against NaN, so even a dark pixel renders white.
        (1, 1, 1.0, true, &[255]),
    ];
    for &(h, w, scale, disable_auto_dot, expected) in cases {
        let src = (0..h * w).map(|i| ((i * 7 + 2) % 19) as f32 / 20.0).collect();
        let channels = (0..h * w)
            .flat_map(|i| {
                [
                    (i % 3) as f32 / 2.0,
                    [0.0, 0.09, 0.17, 0.36][i % 4],
                    (i % 4) as f32 / 4.0,
                    ((i + 2) % 4) as f32 / 4.0,
                    (i % 2) as f32 * 0.5,
                    if i % 5 == 2 { 0.5 } else { 0.0 },
                ]
            })
            .collect();
        let mut img = SVec::new(Shape::new(h, w, None), ImgData::F32(src));
        let mask = SVec::new(Shape::new(h, w, Some(6)), ImgData::F32(channels));
        let cache = DotPatternCache::new((1, 3), scale, disable_auto_dot);
        apply_rotate_ssaa_screentone_masked_cached(&mut img, &mask, &cache, ResizeAlg::Nearest);
        assert_eq!(img.shape(), (h, w, None));
        assert_eq!(
            img.get_data::<u8>().unwrap(),
            expected,
            "scale={scale}, disable_auto_dot={disable_auto_dot}"
        );
    }
}
