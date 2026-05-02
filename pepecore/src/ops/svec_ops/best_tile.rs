use pepecore_array::SVec;
#[derive(Clone)]
struct SendPtr(*const f32);
unsafe impl Send for SendPtr {}
unsafe impl Sync for SendPtr {}
#[derive(Clone)]
pub struct BestTile {
    pub temp_vec: SendPtr,
    pub img_w: usize,
    pub h: usize,   
    pub w: usize,     
    pub lu: Vec<f32>,
    pub y_tile: usize,
    pub x_tile: usize,
    pub y_temp: usize,
}

impl BestTile {
    pub fn new(img: &SVec,y:u16,x:u16)->Self{
        let y_tile: usize = y as usize;
        let x_tile: usize = x as usize;
        let (img_h, img_w, _) = img.shape();
    
        let count_h = img_h - y_tile + 1;
        let count_w = img_w - x_tile + 1;
    
        let mut bt = BestTile {
            temp_vec: SendPtr(img.get_ptr::<f32>().unwrap()),
            img_w,
            lu: Vec::with_capacity(count_h * count_w),
            h: img_h - y_tile, 
            w: img_w - x_tile, 
            x_tile,
            y_tile,
            y_temp: 0,
        };
        bt.full_slide();
        bt
        
    }
    fn init_lu(&mut self) {
        let mut sum_tile = 0.0f32;
        unsafe {
            for y in 0..self.y_tile {
                let row_offset = y * self.img_w;
                for x in 0..self.x_tile {
                    sum_tile += *self.temp_vec.0.add(row_offset + x);
                }
            }
        }
        self.lu.push(sum_tile);
    }

    fn x_slide(&mut self) {
        let mut current_sum = *self.lu.last().unwrap_or(&0.0);

        unsafe {
            for x_offset in 1..=self.w {
                for y in self.y_temp..self.y_temp + self.y_tile {
                    let row_offset = y * self.img_w;
                    current_sum -= *self.temp_vec.0.add(row_offset + (x_offset - 1));
                    current_sum += *self.temp_vec.0.add(row_offset + (x_offset + self.x_tile - 1));
                }
                self.lu.push(current_sum);
            }
        }
    }

    fn y_slide(&mut self) {
        let first_tile_idx = self.y_temp * (self.w + 1);
        let mut sum_tile = self.lu[first_tile_idx];

        unsafe {
            let row_to_remove = self.y_temp * self.img_w;
            let row_to_add = (self.y_temp + self.y_tile) * self.img_w;

            for x in 0..self.x_tile {
                sum_tile -= *self.temp_vec.0.add(row_to_remove + x);
                sum_tile += *self.temp_vec.0.add(row_to_add + x);
            }
        }
        self.y_temp += 1;
        self.lu.push(sum_tile);
    }
    fn full_slide(&mut self){
        self.init_lu();
        for _ in 0..self.h{
            self.x_slide();
            self.y_slide();
        }
        self.x_slide();
    }
    pub fn get_max_coords(&self) -> Option<(usize, usize)> {
            let count_w = self.w + 1;
            self.lu.iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(idx, _)| (idx / count_w, idx % count_w))
        }
        pub fn get_top_n_non_overlapping(
            &mut self, 
            n: usize, 
            threshold: Option<f32> 
        ) -> Vec<(usize, usize)> {
            let mut results = Vec::with_capacity(n);
            let count_w = self.w + 1;
            let count_h = self.h + 1;
            let threshold =  if let Some(th)=threshold {
                Some(th*self.x_tile as f32*self.y_tile as f32)
            } else {
                None
            };
            let mut data = self.lu.clone();
        
            for _ in 0..n {
                let max_idx = data.iter()
                    .enumerate()
                    .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                    .map(|(idx, &val)| (idx, val));
        
                if let Some((idx, val)) = max_idx {
                    if val.is_infinite() && val.is_sign_negative() {
                        break;
                    }
        
                    if let Some(t) = threshold {
                        if val < t {
                            break;
                        }
                    }
        
                    let y = idx / count_w;
                    let x = idx % count_w;
                    results.push((y, x));
        
                    let y_start = y.saturating_sub(self.y_tile - 1);
                    let y_end = (y + self.y_tile).min(count_h);
                    let x_start = x.saturating_sub(self.x_tile - 1);
                    let x_end = (x + self.x_tile).min(count_w);
        
                    for row in y_start..y_end {
                        for col in x_start..x_end {
                            data[row * count_w + col] = f32::NEG_INFINITY; 
                        }
                    }
                } else {
                    break;
                }
            }
            results
        }
        pub fn get_top_n_non_overlapping_(
            &mut self, 
            n: usize, 
            threshold: Option<f32> 
        ) -> Vec<(usize, usize)> {
            let threshold =  if let Some(th)=threshold {
                Some(th*self.x_tile as f32*self.y_tile as f32)
            } else {
                None
            };
            let mut results = Vec::with_capacity(n);
            let count_w = self.w + 1;
            let count_h = self.h + 1;
        
            let data = &mut self.lu;
        
            for _ in 0..n {
                let max_idx = data.iter()
                    .enumerate()
                    .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                    .map(|(idx, &val)| (idx, val));
        
                if let Some((idx, val)) = max_idx {
                    if val.is_infinite() && val.is_sign_negative() {
                        break;
                    }
        
                    if let Some(t) = threshold {
                        if val < t {
                            break; 
                        }
                    }
        
                    let y = idx / count_w;
                    let x = idx % count_w;
                    results.push((y, x));
        
                    let y_start = y.saturating_sub(self.y_tile - 1);
                    let y_end = (y + self.y_tile).min(count_h);
                    let x_start = x.saturating_sub(self.x_tile - 1);
                    let x_end = (x + self.x_tile).min(count_w);
        
                    for row in y_start..y_end {
                        for col in x_start..x_end {
                            data[row * count_w + col] = f32::NEG_INFINITY; 
                        }
                    }
                } else {
                    break;
                }
            }
            results
        }
}
// #[cfg(test)]
// mod tests {
//     use pepecore_array::{ImgData, SVec, Shape};

//     use crate::{read::read_in_path, save::svec_save};

//     use super::*;

//     #[test]
//     fn test_create_u16_svec() {
//         let mut img = read_in_path("/home/umzi/Загрузки/1/test7.png", crate::enums::ImgColor::GRAY).unwrap();
//         img.as_f32();
//         let mut bt = BestTile::new(&img, 256, 256);
//         println!("{:?}",bt.get_max_coords());
//         // println!("{:?}",bt.get_top_n_non_overlapping_(4,None))
//         // let divisor = (x_tile * y_tile) as f32;
//         // let mut result = bt.lu;
//         // result.iter_mut().for_each(|val| *val /= divisor);
    
//         // // Указываем правильный размер: (количество_строк, количество_столбцов)
//         // let result_svec = SVec::new(
//         //     Shape::new(count_h, count_w, None), 
//         //     ImgData::F32(result)
//         // );
        
//         // svec_save(result_svec, "test.png");
//     }
// }