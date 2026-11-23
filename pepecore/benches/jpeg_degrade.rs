use pepecore::{enums::YCbCrRatio, jpeg_compress, ops::svec_ops::jpeg::quantize::QuantizationTableType};
use pepecore_array::{ImgData, SVec, Shape};
use std::{hint::black_box, time::Instant};

fn main() {
    let (w, h) = (1281, 721);
    let mut seed = 42u32;
    let rgb = (0..w * h * 3)
        .map(|_| {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            (seed >> 24) as u8
        })
        .collect::<Vec<_>>();
    for ratio in [YCbCrRatio::R444, YCbCrRatio::R422, YCbCrRatio::R420, YCbCrRatio::R410] {
        let mut times = Vec::new();
        let mut hash = None;
        for i in 0..8 {
            let mut frame = SVec::new(Shape::new(h, w, Some(3)), ImgData::U8(rgb.clone()));
            let start = Instant::now();
            jpeg_compress(&mut frame, 75, &QuantizationTableType::Default, &ratio);
            let dt = start.elapsed();
            let digest = black_box(frame.get_data::<u8>().unwrap())
                .iter()
                .fold(0xcbf29ce484222325u64, |h, &v| (h ^ u64::from(v)).wrapping_mul(0x100000001b3));
            if i > 0 {
                assert_eq!(hash, Some(digest));
                times.push(dt);
            }
            hash = Some(digest);
        }
        times.sort_unstable();
        println!(
            "{ratio:?} {w}x{h} median {:.3} ms checksum {:016x}",
            times[3].as_secs_f64() * 1000.,
            hash.unwrap()
        );
    }
}
