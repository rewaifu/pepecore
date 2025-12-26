//  Чёта я пока ебал это сам писать, так что ретрансляция с си на раст
pub const MAXCOLOR: usize = 4096;
const RED: usize = 2;
const GREEN: usize = 1;
const BLUE: usize = 0;
const HIST_SIZE: usize = 33 * 33 * 33;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Box_ {
    pub r0: i32,
    pub r1: i32,
    pub g0: i32,
    pub g1: i32,
    pub b0: i32,
    pub b1: i32,
    pub vol: i32,
}

pub struct WuQuantizer {
    pub ir: Vec<u8>,
    pub ig: Vec<u8>,
    pub ib: Vec<u8>,
    pub size: usize,
    pub k: usize,
    pub wt: Vec<i64>,
    pub mr: Vec<i64>,
    pub mg: Vec<i64>,
    pub mb: Vec<i64>,
    pub m2: Vec<f32>,
    pub qadd: Vec<u16>,
    pub cube: Vec<Box_>,
    pub tag: Vec<u16>,
    pub lut_r: Vec<u8>,
    pub lut_g: Vec<u8>,
    pub lut_b: Vec<u8>,
}

impl WuQuantizer {
    pub fn new(image: &[u8], k: usize) -> Result<Self, String> {
        if image.len() % 3 != 0 {
            return Err("Image length must be divisible by 3 (HWC format with 3 channels)".to_string());
        }
        let size = image.len() / 3;
        if size == 0 {
            return Err("Image must have at least one pixel".to_string());
        }
        if k == 0 || k > MAXCOLOR {
            return Err(format!("k must be between 1 and {MAXCOLOR} inclusive").to_string());
        }
        let mut ir = Vec::with_capacity(size);
        let mut ig = Vec::with_capacity(size);
        let mut ib = Vec::with_capacity(size);
        for i in 0..size {
            ir.push(image[i * 3]);
            ig.push(image[i * 3 + 1]);
            ib.push(image[i * 3 + 2]);
        }
        Ok(WuQuantizer {
            ir,
            ig,
            ib,
            size,
            k,
            wt: vec![0; HIST_SIZE],
            mr: vec![0; HIST_SIZE],
            mg: vec![0; HIST_SIZE],
            mb: vec![0; HIST_SIZE],
            m2: vec![0.0; HIST_SIZE],
            qadd: Vec::new(),
            cube: Vec::new(),
            tag: Vec::new(),
            lut_r: Vec::new(),
            lut_g: Vec::new(),
            lut_b: Vec::new(),
        })
    }

    fn get_index(&self, inr: usize, ing: usize, inb: usize) -> usize {
        (inr << 10) + (inr << 6) + inr + (ing << 5) + ing + inb
    }

    pub fn hist3d(&mut self) {
        let table: Vec<i32> = (0..256).map(|i| i * i).collect();
        for i in 0..self.size {
            let r = self.ir[i] as usize;
            let g = self.ig[i] as usize;
            let b = self.ib[i] as usize;
            let inr = (r >> 3) + 1;
            let ing = (g >> 3) + 1;
            let inb = (b >> 3) + 1;
            let ind = self.get_index(inr, ing, inb);
            self.qadd.push(ind as u16);
            self.wt[ind] += 1;
            self.mr[ind] += r as i64;
            self.mg[ind] += g as i64;
            self.mb[ind] += b as i64;
            self.m2[ind] += (table[r] + table[g] + table[b]) as f32;
        }
    }

    pub fn m3d(&mut self) {
        for r in 1..=32 {
            let mut area_w: Vec<i64> = vec![0; 33];
            let mut area_r: Vec<i64> = vec![0; 33];
            let mut area_g: Vec<i64> = vec![0; 33];
            let mut area_b: Vec<i64> = vec![0; 33];
            let mut area2: Vec<f32> = vec![0.0; 33];
            for g in 1..=32 {
                let mut line_w: i64 = 0;
                let mut line_r: i64 = 0;
                let mut line_g: i64 = 0;
                let mut line_b: i64 = 0;
                let mut line2: f32 = 0.0;
                for b in 1..=32 {
                    let ind1 = self.get_index(r, g, b);
                    line_w += self.wt[ind1];
                    line_r += self.mr[ind1];
                    line_g += self.mg[ind1];
                    line_b += self.mb[ind1];
                    line2 += self.m2[ind1];
                    area_w[b] += line_w;
                    area_r[b] += line_r;
                    area_g[b] += line_g;
                    area_b[b] += line_b;
                    area2[b] += line2;
                    let ind2 = ind1 - 1089;
                    self.wt[ind1] = self.wt[ind2] + area_w[b];
                    self.mr[ind1] = self.mr[ind2] + area_r[b];
                    self.mg[ind1] = self.mg[ind2] + area_g[b];
                    self.mb[ind1] = self.mb[ind2] + area_b[b];
                    self.m2[ind1] = self.m2[ind2] + area2[b];
                }
            }
        }
    }

    fn vol(&self, cube: &Box_, mmt: &[i64]) -> i64 {
        mmt[self.get_index(cube.r1 as usize, cube.g1 as usize, cube.b1 as usize)]
            - mmt[self.get_index(cube.r1 as usize, cube.g1 as usize, cube.b0 as usize)]
            - mmt[self.get_index(cube.r1 as usize, cube.g0 as usize, cube.b1 as usize)]
            + mmt[self.get_index(cube.r1 as usize, cube.g0 as usize, cube.b0 as usize)]
            - mmt[self.get_index(cube.r0 as usize, cube.g1 as usize, cube.b1 as usize)]
            + mmt[self.get_index(cube.r0 as usize, cube.g1 as usize, cube.b0 as usize)]
            + mmt[self.get_index(cube.r0 as usize, cube.g0 as usize, cube.b1 as usize)]
            - mmt[self.get_index(cube.r0 as usize, cube.g0 as usize, cube.b0 as usize)]
    }

    fn vol_m2(&self, cube: &Box_) -> f32 {
        self.m2[self.get_index(cube.r1 as usize, cube.g1 as usize, cube.b1 as usize)]
            - self.m2[self.get_index(cube.r1 as usize, cube.g1 as usize, cube.b0 as usize)]
            - self.m2[self.get_index(cube.r1 as usize, cube.g0 as usize, cube.b1 as usize)]
            + self.m2[self.get_index(cube.r1 as usize, cube.g0 as usize, cube.b0 as usize)]
            - self.m2[self.get_index(cube.r0 as usize, cube.g1 as usize, cube.b1 as usize)]
            + self.m2[self.get_index(cube.r0 as usize, cube.g1 as usize, cube.b0 as usize)]
            + self.m2[self.get_index(cube.r0 as usize, cube.g0 as usize, cube.b1 as usize)]
            - self.m2[self.get_index(cube.r0 as usize, cube.g0 as usize, cube.b0 as usize)]
    }

    fn var(&self, cube: &Box_) -> f32 {
        let dr = self.vol(cube, &self.mr) as f32;
        let dg = self.vol(cube, &self.mg) as f32;
        let db = self.vol(cube, &self.mb) as f32;
        let xx = self.vol_m2(cube);
        xx - (dr * dr + dg * dg + db * db) / self.vol(cube, &self.wt) as f32
    }

    fn bottom(&self, cube: &Box_, dir: usize, mmt: &[i64]) -> i64 {
        match dir {
            RED => {
                -mmt[self.get_index(cube.r0 as usize, cube.g1 as usize, cube.b1 as usize)]
                    + mmt[self.get_index(cube.r0 as usize, cube.g1 as usize, cube.b0 as usize)]
                    + mmt[self.get_index(cube.r0 as usize, cube.g0 as usize, cube.b1 as usize)]
                    - mmt[self.get_index(cube.r0 as usize, cube.g0 as usize, cube.b0 as usize)]
            }
            GREEN => {
                -mmt[self.get_index(cube.r1 as usize, cube.g0 as usize, cube.b1 as usize)]
                    + mmt[self.get_index(cube.r1 as usize, cube.g0 as usize, cube.b0 as usize)]
                    + mmt[self.get_index(cube.r0 as usize, cube.g0 as usize, cube.b1 as usize)]
                    - mmt[self.get_index(cube.r0 as usize, cube.g0 as usize, cube.b0 as usize)]
            }
            BLUE => {
                -mmt[self.get_index(cube.r1 as usize, cube.g1 as usize, cube.b0 as usize)]
                    + mmt[self.get_index(cube.r1 as usize, cube.g0 as usize, cube.b0 as usize)]
                    + mmt[self.get_index(cube.r0 as usize, cube.g1 as usize, cube.b0 as usize)]
                    - mmt[self.get_index(cube.r0 as usize, cube.g0 as usize, cube.b0 as usize)]
            }
            _ => 0,
        }
    }

    fn top(&self, cube: &Box_, dir: usize, pos: usize, mmt: &[i64]) -> i64 {
        match dir {
            RED => {
                mmt[self.get_index(pos, cube.g1 as usize, cube.b1 as usize)]
                    - mmt[self.get_index(pos, cube.g1 as usize, cube.b0 as usize)]
                    - mmt[self.get_index(pos, cube.g0 as usize, cube.b1 as usize)]
                    + mmt[self.get_index(pos, cube.g0 as usize, cube.b0 as usize)]
            }
            GREEN => {
                mmt[self.get_index(cube.r1 as usize, pos, cube.b1 as usize)]
                    - mmt[self.get_index(cube.r1 as usize, pos, cube.b0 as usize)]
                    - mmt[self.get_index(cube.r0 as usize, pos, cube.b1 as usize)]
                    + mmt[self.get_index(cube.r0 as usize, pos, cube.b0 as usize)]
            }
            BLUE => {
                mmt[self.get_index(cube.r1 as usize, cube.g1 as usize, pos)]
                    - mmt[self.get_index(cube.r1 as usize, cube.g0 as usize, pos)]
                    - mmt[self.get_index(cube.r0 as usize, cube.g1 as usize, pos)]
                    + mmt[self.get_index(cube.r0 as usize, cube.g0 as usize, pos)]
            }
            _ => 0,
        }
    }

    fn maximize(
        &self,
        cube: &Box_,
        dir: usize,
        first: i32,
        last: i32,
        cut: &mut i32,
        whole_r: i64,
        whole_g: i64,
        whole_b: i64,
        whole_w: i64,
    ) -> f32 {
        let base_r = self.bottom(cube, dir, &self.mr);
        let base_g = self.bottom(cube, dir, &self.mg);
        let base_b = self.bottom(cube, dir, &self.mb);
        let base_w = self.bottom(cube, dir, &self.wt);
        let mut maxx = 0.0;
        *cut = -1;
        for i in first as usize..last as usize {
            let mut half_r = base_r + self.top(cube, dir, i, &self.mr);
            let mut half_g = base_g + self.top(cube, dir, i, &self.mg);
            let mut half_b = base_b + self.top(cube, dir, i, &self.mb);
            let mut half_w = base_w + self.top(cube, dir, i, &self.wt);
            if half_w == 0 {
                continue;
            }
            let mut temp =
                (half_r as f32 * half_r as f32 + half_g as f32 * half_g as f32 + half_b as f32 * half_b as f32) / half_w as f32;
            half_r = whole_r - half_r;
            half_g = whole_g - half_g;
            half_b = whole_b - half_b;
            half_w = whole_w - half_w;
            if half_w == 0 {
                continue;
            }
            temp +=
                (half_r as f32 * half_r as f32 + half_g as f32 * half_g as f32 + half_b as f32 * half_b as f32) / half_w as f32;
            if temp > maxx {
                maxx = temp;
                *cut = i as i32;
            }
        }
        maxx
    }

    fn cut(&self, set1: &mut Box_, set2: &mut Box_) -> bool {
        let whole_r = self.vol(set1, &self.mr);
        let whole_g = self.vol(set1, &self.mg);
        let whole_b = self.vol(set1, &self.mb);
        let whole_w = self.vol(set1, &self.wt);
        let mut cutr = 0i32;
        let maxr = self.maximize(set1, RED, set1.r0 + 1, set1.r1, &mut cutr, whole_r, whole_g, whole_b, whole_w);
        let mut cutg = 0i32;
        let maxg = self.maximize(
            set1,
            GREEN,
            set1.g0 + 1,
            set1.g1,
            &mut cutg,
            whole_r,
            whole_g,
            whole_b,
            whole_w,
        );
        let mut cutb = 0i32;
        let maxb = self.maximize(
            set1,
            BLUE,
            set1.b0 + 1,
            set1.b1,
            &mut cutb,
            whole_r,
            whole_g,
            whole_b,
            whole_w,
        );
        let dir = if maxr >= maxg && maxr >= maxb {
            if cutr < 0 {
                return false;
            }
            RED
        } else if maxg >= maxr && maxg >= maxb {
            GREEN
        } else {
            BLUE
        };
        set2.r1 = set1.r1;
        set2.g1 = set1.g1;
        set2.b1 = set1.b1;
        match dir {
            RED => {
                set2.r0 = cutr;
                set1.r1 = cutr;
                set2.g0 = set1.g0;
                set2.b0 = set1.b0;
            }
            GREEN => {
                set2.g0 = cutg;
                set1.g1 = cutg;
                set2.r0 = set1.r0;
                set2.b0 = set1.b0;
            }
            BLUE => {
                set2.b0 = cutb;
                set1.b1 = cutb;
                set2.r0 = set1.r0;
                set2.g0 = set1.g0;
            }
            _ => {}
        }
        set1.vol = ((set1.r1 - set1.r0) as i64 * (set1.g1 - set1.g0) as i64 * (set1.b1 - set1.b0) as i64) as i32;
        set2.vol = ((set2.r1 - set2.r0) as i64 * (set2.g1 - set2.g0) as i64 * (set2.b1 - set2.b0) as i64) as i32;
        true
    }
    fn get_index_static(inr: usize, ing: usize, inb: usize) -> usize {
        (inr << 10) + (inr << 6) + inr + (ing << 5) + ing + inb
    }

    // Сделали статической. Теперь она принимает только то, что ей нужно.
    fn mark_static(cube: &Box_, label: u16, tag: &mut [u16]) {
        for r in (cube.r0 + 1)..=cube.r1 {
            for g in (cube.g0 + 1)..=cube.g1 {
                for b in (cube.b0 + 1)..=cube.b1 {
                    let ind = Self::get_index_static(r as usize, g as usize, b as usize);
                    tag[ind] = label;
                }
            }
        }
    }

    pub fn quantize(&mut self) -> Result<(), String> {
        self.hist3d();
        self.m3d();

        let mut cube_vec = vec![Box_ {
            r0: 0,
            r1: 32,
            g0: 0,
            g1: 32,
            b0: 0,
            b1: 32,
            vol: 32 * 32 * 32,
        }];
        let mut vv = vec![if cube_vec[0].vol > 1 { self.var(&cube_vec[0]) } else { 0.0 }];
        while cube_vec.len() < self.k {
            let next = vv
                .iter()
                .enumerate()
                .max_by(|a, b| a.1.partial_cmp(b.1).expect("REASON"))
                .map(|(index, _)| index);
            if next.is_none() {
                return Err(String::from("next not_found"));
            }
            let next = next.unwrap();
            let mut new_cube = Box_::default();
            if self.cut(&mut cube_vec[next], &mut new_cube) {
                vv[next] = if cube_vec[next].vol > 1 {
                    self.var(&cube_vec[next])
                } else {
                    0.0
                };
                vv.push(if new_cube.vol > 1 { self.var(&new_cube) } else { 0.0 });
                cube_vec.push(new_cube);
            } else {
                vv[next] = 0.0;
                if vv.iter().all(|&v| v <= 0.0) {
                    break;
                }
            }
        }
        self.k = cube_vec.len();
        self.cube = cube_vec;

        self.lut_r.resize_with(self.k, || 0);
        self.lut_g.resize_with(self.k, || 0);
        self.lut_b.resize_with(self.k, || 0);
        self.tag = vec![0; HIST_SIZE];
        for k in 0..self.k {
            Self::mark_static(&self.cube[k], k as u16, &mut self.tag);
            let weight = self.vol(&self.cube[k], &self.wt);
            if weight > 0 {
                self.lut_r[k] = (((self.vol(&self.cube[k], &self.mr) as f64 / weight as f64).round() as i64).clamp(0, 255)) as u8;
                self.lut_g[k] = (((self.vol(&self.cube[k], &self.mg) as f64 / weight as f64).round() as i64).clamp(0, 255)) as u8;
                self.lut_b[k] = (((self.vol(&self.cube[k], &self.mb) as f64 / weight as f64).round() as i64).clamp(0, 255)) as u8;
            } else {
                self.lut_r[k] = 0;
                self.lut_g[k] = 0;
                self.lut_b[k] = 0;
            }
        }
        for i in 0..self.size {
            self.qadd[i] = self.tag[self.qadd[i] as usize] as u16;
        }
        Ok(())
    }

    pub fn get_palette(&self) -> Vec<f32> {
        let mut palette = Vec::with_capacity(self.k * 3);
        for i in 0..self.k {
            palette.push(self.lut_r[i] as f32 / 255.0);
            palette.push(self.lut_g[i] as f32 / 255.0);
            palette.push(self.lut_b[i] as f32 / 255.0);
        }
        palette
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::ImgColor;
    use crate::read::read_in_path;
    use std::time::Instant;

    #[test]
    fn test_basic_colors() {
        let img = read_in_path(
            "/run/media/umzi/H/nahuy_pixiv/WOSManga_train_test/hq/000012.png",
            ImgColor::RGB,
        )
        .unwrap();
        let data = img.get_data::<u8>().unwrap();
        let t = Instant::now();
        let mut quantizer = WuQuantizer::new(data, 3000).unwrap();
        quantizer.quantize();
        quantizer.quantize();
        println!("{:?}", t.elapsed());
        // Output the palette
        let palette = quantizer.get_palette();

        println!("Palette (RGBRGB... normalized): {:?}", palette);
        // let colors = collect_colors(data);
        // let median = median_cut(colors, 8);
        // for (i, c) in palette.iter().enumerate() {
        //     println!("  {}: RGB({}, {}, {})", i, c.0, c.1, c.2);
        // }
    }
}
