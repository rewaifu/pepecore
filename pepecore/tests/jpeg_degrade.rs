use pepecore::{
    enums::YCbCrRatio,
    jpeg_compress,
    ops::svec_ops::jpeg::{
        encode::jpeg_compress_one_ch,
        quantize::{QuantizationTable, QuantizationTableType},
    },
};
use pepecore_array::{ImgData, SVec, Shape};

// Scalar oracle: form full-resolution chroma, box-filter actual pixels, then
// compress three independently sized planes and nearest-expand by (sx, sy).
fn reference(rgb: &[u8], w: usize, h: usize, sx: usize, sy: usize) -> Vec<u8> {
    let (cw, ch) = (w.div_ceil(sx), h.div_ceil(sy));
    let mut y = Vec::new();
    let mut sums = vec![[0u32; 3]; cw * ch];
    for (i, p) in rgb.chunks_exact(3).enumerate() {
        let (r, g, b) = (i32::from(p[0]), i32::from(p[1]), i32::from(p[2]));
        y.push(((19595 * r + 38470 * g + 7471 * b + 32767) >> 16) as u8);
        let s = &mut sums[(i / w / sy) * cw + i % w / sx];
        s[0] += ((-11059 * r - 21709 * g + 32768 * b + (128 << 16) + 32767) >> 16) as u32;
        s[1] += ((32768 * r - 27439 * g - 5329 * b + (128 << 16) + 32767) >> 16) as u32;
        s[2] += 1;
    }
    let plane = |k: usize| sums.iter().map(|s| ((s[k] + s[2] / 2) / s[2]) as u8).collect::<Vec<_>>();
    let (mut u, mut v) = (plane(0), plane(1));
    let q = |luma| QuantizationTable::new_with_quality(&QuantizationTableType::Default, 75, luma);
    jpeg_compress_one_ch(&mut y, h, w, &q(true));
    jpeg_compress_one_ch(&mut u, ch, cw, &q(false));
    jpeg_compress_one_ch(&mut v, ch, cw, &q(false));
    y.iter()
        .enumerate()
        .flat_map(|(i, &y)| {
            let k = i / w / sy * cw + i % w / sx;
            let (y, u, v) = (i32::from(y), i32::from(u[k]) - 128, i32::from(v[k]) - 128);
            [
                y + ((91881 * v + 32768) >> 16),
                y - ((22554 * u + 46802 * v + 32768) >> 16),
                y + ((116130 * u + 32768) >> 16),
            ]
            .map(|c| c.clamp(0, 255) as u8)
        })
        .collect()
}

#[test]
fn sampling_geometry_and_roundtrip() {
    use YCbCrRatio::*;
    for (ratio, sx, sy) in [
        (R444, 1, 1),
        (R420, 2, 2),
        (R422, 2, 1),
        (R440, 1, 2),
        (R411, 4, 1),
        (R441, 1, 4),
        (R410, 4, 2),
    ] {
        for (w, h) in [(19, 17), (1, 1), (1, 15), (17, 1), (32, 16), (257, 257)] {
            let rgb: Vec<_> = (0..w * h * 3)
                .map(|i| ((i * 71 + i / 3 * 13 + i / 11 * 37) % 256) as u8)
                .collect();
            let expected = reference(&rgb, w, h, sx, sy);
            let mut img = SVec::new(Shape::new(h, w, Some(3)), ImgData::U8(rgb));
            jpeg_compress(&mut img, 75, &QuantizationTableType::Default, &ratio);
            let actual = img.get_data::<u8>().unwrap();
            assert_eq!(img.shape(), (h, w, Some(3)));
            assert!(
                actual == expected,
                "{ratio:?} {w}x{h}, first mismatch {:?}",
                actual.iter().zip(expected).position(|(a, b)| *a != b)
            );
        }
    }
}

#[test]
fn custom_quantizer_above_255_preserves_dc() {
    let mut img = SVec::new(Shape::new(1, 1, None), ImgData::U8(vec![255]));
    let table = QuantizationTableType::Custom(Box::new([256; 64]));
    jpeg_compress(&mut img, 75, &table, &YCbCrRatio::R444);
    assert_eq!(img.get_data::<u8>().unwrap(), &[255]);
}
