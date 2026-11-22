//! `RAYON_NUM_THREADS=8 cargo bench -p pepecore --features encode --bench masked_screentone`
//! Original sequential cached renderer, Ryzen 7 5700X (ms / output FNV-1a, in case order):
//! 8.890 / 60933f729a62d31b; 54.928 / 8a4b05f3d2ce40f2;
//! 158.048 / ddcb5735af579d81; 17.597 / 6a55d0ca59cd01b8.

use fast_image_resize::{FilterType, ResizeAlg};
use pepecore::ops::svec_ops::new_halftone::screentone::{DotPatternCache, apply_rotate_ssaa_screentone_masked_cached};
use pepecore_array::{ImgData, SVec, Shape};
use std::{hint::black_box, time::Instant};

fn main() {
    let (w, h) = (641, 481);
    for (scale, uniform, skip) in [
        (1.0, false, false),
        (2.5, false, false),
        (4.0, true, false),
        (4.0, true, true),
    ] {
        let mut seed = 1u32;
        let mut random = || {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            (seed >> 8) as f32 / 16777216.0
        };
        let src: Vec<_> = (0..w * h).map(|_| random()).collect();
        let mask = (0..w * h)
            .flat_map(|_| {
                if uniform {
                    [0.4, 0.125, 0.2, 0.8, 1.0, u8::from(skip) as f32]
                } else {
                    [random(), random(), random(), random(), random(), random() * 0.6]
                }
            })
            .collect();
        let mask = SVec::new(Shape::new(h, w, Some(6)), ImgData::F32(mask));
        let cache = DotPatternCache::new((2, 8), scale, false);
        let mut times = Vec::new();
        let mut hash = 0;
        for trial in 0..8 {
            let mut img = SVec::new(Shape::new(h, w, None), ImgData::F32(src.clone()));
            let start = Instant::now();
            apply_rotate_ssaa_screentone_masked_cached(&mut img, &mask, &cache, ResizeAlg::Convolution(FilterType::CatmullRom));
            let elapsed = start.elapsed();
            let out = black_box(img.get_data::<u8>().unwrap());
            let digest = out
                .iter()
                .fold(0xcbf29ce484222325u64, |a, &b| (a ^ u64::from(b)).wrapping_mul(0x100000001b3));
            if trial == 0 {
                hash = digest;
            } else {
                assert_eq!(hash, digest);
                times.push(elapsed);
            }
        }
        times.sort_unstable();
        println!(
            "{w}x{h} scale={scale} uniform={uniform} skip={skip} median={:.3} ms hash={hash:016x}",
            times[3].as_secs_f64() * 1000.0
        );
    }
}
