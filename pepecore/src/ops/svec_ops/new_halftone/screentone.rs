use super::dot::patterns;
use fast_image_resize::{
    PixelType, ResizeAlg, ResizeOptions, Resizer,
    images::{Image, ImageRef},
};
use pepecore_array::{ImgData, SVec, Shape};
use rayon::prelude::*;

#[cfg(target_arch = "x86_64")]
mod avx2;

#[derive(Clone, Copy)]
struct Entry {
    size: usize,
    offset: i32,
    period: i32,
    reciprocal: u32,
}

pub struct DotPatternCache {
    min_size: usize,
    max_size: usize,
    scale: f32,
    disable_auto_dot: bool,
    buckets: Vec<[Entry; 2]>,
    atlas: Vec<f32>,
}

impl DotPatternCache {
    pub fn new((min_size, max_size): (usize, usize), scale: f32, disable_auto_dot: bool) -> Self {
        assert!(scale.is_finite() && scale > 0.0);
        let max_size = max_size.max(min_size);
        let mut buckets: Vec<[Entry; 2]> = Vec::new();
        let mut atlas = Vec::new();
        for raw in min_size..=max_size {
            let size = if disable_auto_dot {
                raw
            } else {
                (raw as f32 * scale).round() as usize
            }
            .max(1);
            if let Some(&last) = buckets.last().filter(|pair| pair[0].size == size) {
                buckets.push(last);
                continue;
            }
            // Two square tiles of side 2*size; SIMD gather uses signed i32 indices.
            let count = size
                .checked_mul(size)
                .and_then(|n| n.checked_mul(8))
                .expect("dot size overflow");
            assert!(count <= i32::MAX as usize - atlas.len(), "dot atlas too large");
            let pair = patterns(size).map(|p| {
                let offset = atlas.len() as i32;
                atlas.extend_from_slice(&p.values);
                Entry {
                    size: p.size,
                    offset,
                    period: p.period,
                    reciprocal: ((1u64 << 32) / p.period as u64) as u32,
                }
            });
            buckets.push(pair);
        }
        Self {
            min_size,
            max_size,
            scale,
            disable_auto_dot,
            buckets,
            atlas,
        }
    }

    pub fn min_size(&self) -> usize {
        self.min_size
    }
    pub fn max_size(&self) -> usize {
        self.max_size
    }
    pub fn scale(&self) -> f32 {
        self.scale
    }
    pub fn disable_auto_dot(&self) -> bool {
        self.disable_auto_dot
    }
}

#[derive(Default)]
struct Lanes {
    dx: [f32; 8],
    by: [i32; 8],
    sin: [f32; 8],
    cos: [f32; 8],
    tone: [f32; 8],
    period: [i32; 8],
    reciprocal: [u32; 8],
    offset: [i32; 8],
    skip: [i32; 8],
}

#[derive(Clone, Copy)]
struct Sample {
    pattern: Entry,
    bx: usize,
    by: usize,
    sin: f32,
    cos: f32,
    tone: f32,
    skip: i32,
}

impl DotPatternCache {
    fn sample(&self, m: &[f32; 6], tone: f32, last_angle: &mut (f32, (f32, f32))) -> Sample {
        if m[5] >= 0.5 {
            return Sample {
                pattern: self.buckets[0][0],
                bx: 0,
                by: 0,
                sin: 0.0,
                cos: 1.0,
                tone,
                skip: (tone.clamp(0.0, 1.0) * 255.0).round() as u8 as i32,
            };
        }
        let span = self.buckets.len() - 1;
        let bucket = ((m[0] * span as f32).round() as usize).min(span);
        let pattern = self.buckets[bucket][usize::from(!(m[4] < 0.5))];
        let bias = |v: f32| (pattern.size / 2).saturating_add((v * pattern.size as f32).round() as usize);
        if last_angle.0 != m[1] {
            *last_angle = (m[1], (m[1] * 2.0 * std::f32::consts::PI).sin_cos());
        }
        let (sin, cos) = last_angle.1;
        Sample {
            pattern,
            bx: bias(m[2]),
            by: bias(m[3]),
            sin,
            cos,
            tone,
            skip: -1,
        }
    }
}

fn nearest(src: usize, dst: usize) -> Vec<usize> {
    let ratio = src as f64 / dst as f64;
    (0..dst)
        .map(|i| ((ratio * 0.5 + ratio * i as f64) as usize).min(src - 1))
        .collect()
}

struct Raster<'a> {
    cache: &'a DotPatternCache,
    xs: &'a [usize],
    cx: f32,
    cy: f32,
    simd: bool,
}

impl Raster<'_> {
    // One cache-sized horizontal block feeds all SSAA rows sampling this source row.
    fn rows(&self, dst: &mut [u8], y0: usize, tones: &[f32], mask: &[[f32; 6]]) {
        let width = self.xs.len();
        let rows = dst.len() / width;
        if mask.iter().all(|m| m[5] >= 0.5) {
            // All rows in this band sample the same tones; quantize just once.
            let (first, rest) = dst.split_at_mut(width);
            for (byte, &x) in first.iter_mut().zip(self.xs) {
                *byte = (tones[x].clamp(0.0, 1.0) * 255.0).round() as u8;
            }
            for row in rest.chunks_exact_mut(width) {
                row.copy_from_slice(first);
            }
            return;
        }
        let mut angle = (0.0, (0.0, 1.0));
        let mut last = self.cache.sample(&mask[self.xs[0]], tones[self.xs[0]], &mut angle);
        let mut prev = self.xs[0];
        for (block, xs) in self.xs.chunks(8).enumerate() {
            let x0 = block * 8;
            let mut l = Lanes::default();
            let mut wide_by = [0; 8];
            let mut vector = self.simd && xs.len() == 8 && y0 + rows <= i32::MAX as usize;
            let mut all_skip = true;
            for (i, &x) in xs.iter().enumerate() {
                if x != prev {
                    last = self.cache.sample(&mask[x], tones[x], &mut angle);
                    prev = x;
                }
                l.dx[i] = last.bx.saturating_add(x0 + i) as f32 - self.cx;
                l.by[i] = last.by as i32;
                wide_by[i] = last.by;
                vector &= last.by <= (i32::MAX as usize).saturating_sub(y0 + rows);
                l.sin[i] = last.sin;
                l.cos[i] = last.cos;
                l.period[i] = last.pattern.period;
                l.reciprocal[i] = last.pattern.reciprocal;
                l.offset[i] = last.pattern.offset;
                l.tone[i] = last.tone;
                l.skip[i] = last.skip;
                all_skip &= last.skip >= 0;
            }
            if all_skip {
                let bytes = l.skip.map(|s| s as u8);
                for row in dst.chunks_exact_mut(width) {
                    row[x0..x0 + xs.len()].copy_from_slice(&bytes[..xs.len()]);
                }
                continue;
            }
            for (dy, row) in dst.chunks_exact_mut(width).enumerate() {
                let y = y0 + dy;
                let out = &mut row[x0..x0 + xs.len()];
                #[cfg(target_arch = "x86_64")]
                if vector {
                    // Full block, AVX2 detected, y+bias fits i32. Each cache entry
                    // (including skipped lanes) owns period² consecutive atlas values.
                    unsafe { avx2::render(&l, y as i32, self.cx, self.cy, &self.cache.atlas, out.try_into().unwrap()) };
                    continue;
                }
                for (i, byte) in out.iter_mut().enumerate() {
                    if l.skip[i] >= 0 {
                        *byte = l.skip[i] as u8;
                        continue;
                    }
                    let dy = wide_by[i].saturating_add(y) as f32 - self.cy;
                    let rx = l.cos[i] * l.dx[i] - l.sin[i] * dy + self.cx;
                    let ry = l.sin[i] * l.dx[i] + l.cos[i] * dy + self.cy;
                    let p = l.period[i];
                    let ix = (rx.round() as i32).rem_euclid(p);
                    let iy = (ry.round() as i32).rem_euclid(p);
                    let threshold = self.cache.atlas[(l.offset[i] + iy * p + ix) as usize];
                    *byte = if l.tone[i] < threshold { 0 } else { 255 };
                }
            }
        }
    }
}

/// Renders an H×W grayscale f32 image with an H×W×6 source-resolution mask.
/// Mask channels: size, rotation (turns), x/y phase, grid selector, bypass.
pub fn apply_rotate_ssaa_screentone_masked_cached(img: &mut SVec, mask: &SVec, cache: &DotPatternCache, resize_alg: ResizeAlg) {
    let (h, w, channels) = img.shape();
    assert!(channels.unwrap_or(1) == 1 && mask.shape() == (h, w, Some(6)));
    let tones = img.get_data::<f32>().unwrap();
    let data = mask.get_data::<f32>().unwrap();
    assert_eq!(tones.len(), h.checked_mul(w).unwrap());
    assert_eq!(data.len(), tones.len().checked_mul(6).unwrap());
    // [f32; 6] has f32 alignment and no padding; the validated slice spans exactly H*W arrays.
    let mask = unsafe { std::slice::from_raw_parts(data.as_ptr().cast::<[f32; 6]>(), tones.len()) };
    let (sh, sw) = ((h as f32 * cache.scale) as usize, (w as f32 * cache.scale) as usize);
    assert!(sh > 0 && sw > 0 && [h, w, sh, sw].iter().all(|&v| u32::try_from(v).is_ok()));
    let xs = nearest(w, sw);
    let ys = nearest(h, sh);
    #[cfg(target_arch = "x86_64")]
    let simd = is_x86_feature_detected!("avx2");
    #[cfg(not(target_arch = "x86_64"))]
    let simd = false;
    let raster = Raster {
        cache,
        xs: &xs,
        cx: sw as f32 * 0.5,
        cy: sh as f32 * 0.5,
        simd,
    };
    let mut out = vec![0u8; sh.checked_mul(sw).unwrap()];
    let band = sh.min(8);
    let render = |(b, mut data): (usize, &mut [u8])| {
        let mut y = b * band;
        while !data.is_empty() {
            let src = ys[y];
            let n = ys[y..y + data.len() / sw].partition_point(|&s| s == src);
            let (rows, rest) = data.split_at_mut(n * sw);
            let range = src * w..(src + 1) * w;
            raster.rows(rows, y, &tones[range.clone()], &mask[range]);
            data = rest;
            y += n;
        }
    };
    if out.len() >= 32768 {
        out.par_chunks_mut(band * sw).enumerate().for_each(render);
    } else {
        out.chunks_mut(band * sw).enumerate().for_each(render);
    }
    if (sh, sw) != (h, w) {
        let src = ImageRef::new(sw as u32, sh as u32, &out, PixelType::U8).unwrap();
        let mut dst = Image::new(w as u32, h as u32, PixelType::U8);
        Resizer::new()
            .resize(&src, &mut dst, &ResizeOptions::new().resize_alg(resize_alg).use_alpha(false))
            .unwrap();
        out = dst.into_vec();
    }
    img.data = ImgData::U8(out);
    img.shape = Shape::new(h, w, None);
}

#[cfg(all(test, target_arch = "x86_64"))]
mod tests {
    use super::*;

    #[test]
    fn vector_matches_scalar_for_mixed_lanes_and_tail() {
        if !is_x86_feature_detected!("avx2") {
            return;
        }
        let cache = DotPatternCache::new((1, 17), 1.0, false);
        let tones: Vec<_> = (0..129).map(|x| if x == 5 { f32::NAN } else { x as f32 / 129.0 }).collect();
        let mask: Vec<_> = (0..129)
            .map(|x| {
                [
                    (x % 23) as f32 / 16.0,
                    [0.125, -0.375, f32::NAN, 0.25][x % 4],
                    if x % 7 == 0 { 3e9 } else { 0.125 },
                    0.875,
                    (x % 2) as f32,
                    (x % 5 == 0) as u8 as f32,
                ]
            })
            .collect();
        let xs = nearest(129, 259);
        let raster = Raster {
            cache: &cache,
            xs: &xs,
            cx: 129.5,
            cy: 31.5,
            simd: false,
        };
        let mut scalar = vec![0; xs.len() * 3];
        let mut vector = scalar.clone();
        raster.rows(&mut scalar, 7, &tones, &mask);
        Raster { simd: true, ..raster }.rows(&mut vector, 7, &tones, &mask);
        assert_eq!(vector, scalar);
    }
}
