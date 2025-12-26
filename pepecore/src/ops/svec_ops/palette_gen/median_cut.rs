use ahash::AHashMap;
#[derive(Clone, Debug)]
pub struct Color {
    rgb: [u8; 3],
    count: u64,
}
impl Color {
    fn channel(&self, ch: usize) -> u8 {
        self.rgb[ch]
    }
}
#[derive(Clone, Debug)]
struct Bucket {
    colors: Vec<Color>,
    rmin: u8,
    rmax: u8,
    gmin: u8,
    gmax: u8,
    bmin: u8,
    bmax: u8,
    total_count: u64,
}

impl Bucket {
    fn new(colors: Vec<Color>) -> Self {
        let mut b = Bucket {
            colors,
            rmin: u8::MAX,
            rmax: u8::MIN,
            gmin: u8::MAX,
            gmax: u8::MIN,
            bmin: u8::MAX,
            bmax: u8::MIN,
            total_count: 0,
        };
        b.recompute();
        b
    }

    fn recompute(&mut self) {
        if self.colors.is_empty() {
            self.rmin = 0;
            self.rmax = 0;
            self.gmin = 0;
            self.gmax = 0;
            self.bmin = 0;
            self.bmax = 0;
            self.total_count = 0;
            return;
        }
        let mut rmin = u8::MAX;
        let mut rmax = u8::MIN;
        let mut gmin = u8::MAX;
        let mut gmax = u8::MIN;
        let mut bmin = u8::MAX;
        let mut bmax = u8::MIN;
        let mut total = 0u64;

        for c in &self.colors {
            let r = c.rgb[0];
            if r < rmin {
                rmin = r
            } else if r > rmax {
                rmax = r
            }
            let g = c.rgb[1];
            if g < gmin {
                gmin = g
            } else if g > gmax {
                gmax = g
            }
            let b = c.rgb[2];
            if b < bmin {
                bmin = b
            } else if b > bmax {
                bmax = b
            }
            total += c.count;
        }

        self.rmin = rmin;
        self.rmax = rmax;
        self.gmin = gmin;
        self.gmax = gmax;
        self.bmin = bmin;
        self.bmax = bmax;
        self.total_count = total;
    }
    fn ranges(&self) -> (u8, u8, u8) {
        (
            self.rmax.saturating_sub(self.rmin),
            self.gmax.saturating_sub(self.gmin),
            self.bmax.saturating_sub(self.bmin),
        )
    }

    fn max_range(&self) -> u8 {
        let (r, g, b) = self.ranges();
        r.max(g).max(b)
    }

    fn channel_with_max_range(&self) -> usize {
        let (r, g, b) = self.ranges();
        if r >= g && r >= b {
            0
        } else if g >= r && g >= b {
            1
        } else {
            2
        }
    }

    fn average_color(&self) -> [f32; 3] {
        if self.colors.is_empty() {
            return [0.0, 0.0, 0.0];
        }
        let mut rs: f32 = 0.0;
        let mut gs: f32 = 0.0;
        let mut bs: f32 = 0.0;
        let mut total: f32 = 0.0;
        for c in &self.colors {
            rs += c.rgb[0] as f32 * c.count as f32;
            gs += c.rgb[1] as f32 * c.count as f32;
            bs += c.rgb[2] as f32 * c.count as f32;
            total += c.count as f32;
        }
        if total == 0.0 {
            return [0.0, 0.0, 0.0];
        }
        total *= 255.0;
        [(rs / total), (gs / total), (bs / total)]
    }
    fn split(mut self) -> (Bucket, Bucket) {
        if self.colors.len() <= 1 {
            let left = Bucket::new(self.colors.clone());
            let right = Bucket::new(Vec::new());
            return (left, right);
        }

        let ch = self.channel_with_max_range();

        self.colors.sort_by_key(|c| c.channel(ch));

        let total = self.total_count;
        if total == 0 {
            let mid = self.colors.len() / 2;
            let left = self.colors[..mid].to_vec();
            let right = self.colors[mid..].to_vec();
            return (Bucket::new(left), Bucket::new(right));
        }

        let mut acc = 0u64;
        let mut split_idx = 0usize;
        for (i, c) in self.colors.iter().enumerate() {
            acc += c.count;
            if acc * 2 >= total {
                split_idx = i + 1;
                break;
            }
        }

        if split_idx == 0 {
            split_idx = 1;
        } else if split_idx >= self.colors.len() {
            split_idx = self.colors.len() - 1;
        }

        let left_colors = self.colors[..split_idx].to_vec();
        let right_colors = self.colors[split_idx..].to_vec();
        (Bucket::new(left_colors), Bucket::new(right_colors))
    }
}

pub fn collect_colors(img: &[u8]) -> Vec<Color> {
    let mut map: AHashMap<u32, u64> = AHashMap::new();

    for pixel in img.chunks(3) {
        let key = ((pixel[0] as u32) << 16) | ((pixel[1] as u32) << 8) | (pixel[2] as u32);
        *map.entry(key).or_insert(0) += 1;
    }

    map.into_iter()
        .map(|(key, count)| {
            let r = ((key >> 16) & 0xFF) as u8;
            let g = ((key >> 8) & 0xFF) as u8;
            let b = (key & 0xFF) as u8;
            Color { rgb: [r, g, b], count }
        })
        .collect()
}
pub fn median_cut(colors: Vec<Color>, target_colors: usize) -> Vec<f32> {
    if target_colors == 0 {
        return vec![];
    }
    if colors.is_empty() {
        return vec![0.0, 0.0, 0.0];
    }
    let mut buckets: Vec<Bucket> = vec![Bucket::new(colors)];
    while buckets.len() < target_colors {
        let (idx, _) = buckets.iter().enumerate().max_by_key(|(_, b)| b.max_range()).unwrap();
        if buckets[idx].max_range() == 0 {
            break;
        }
        let bucket = buckets.remove(idx);
        let (left, right) = bucket.split();
        if left.colors.is_empty() || right.colors.is_empty() {
            buckets.push(left);
            buckets.push(right);
            break;
        } else {
            buckets.push(left);
            buckets.push(right);
        }
    }

    let mut result = Vec::with_capacity(target_colors * 3);
    for b in buckets {
        result.extend_from_slice(&b.average_color())
    }
    result
}
#[cfg(test)]
mod tests {

    use super::*;
    use crate::enums::ImgColor;
    use crate::read::read_in_path;

    #[test]
    fn test_basic_colors() {
        let img = read_in_path(
            "/run/media/umzi/H/nahuy_pixiv/WOSManga_train_test/hq/000012.png",
            ImgColor::RGB,
        )
        .unwrap();
        let data = img.get_data::<u8>().unwrap();
        let colors = collect_colors(data);
        let median = median_cut(colors, 8);
        // for (i, c) in palette.iter().enumerate() {
        //     println!("  {}: RGB({}, {}, {})", i, c.0, c.1, c.2);
        // }
    }
}
