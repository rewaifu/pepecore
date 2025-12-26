use crate::ops::svec_ops::line::objects::{Draw, Line};
use std::collections::HashSet;
pub fn draw_lines(lines: &[Line], pixel_hash: &mut HashSet<(usize, usize)>) {
    for line in lines {
        line.draw(pixel_hash);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ops::svec_ops::line::objects::Point;
    use crate::save::svec_save;
    use pepecore_array::{ImgData, SVec, Shape};
    use std::cmp::min;
    use std::vec;

    #[test]
    fn test_create_u16_svec() {
        let mut pixels: HashSet<(usize, usize)> = HashSet::new();
        let line = vec![
            Line::Bezier(
                Point { x: 0, y: 100, size: 1 },
                Point {
                    x: 500,
                    y: 600,
                    size: 50,
                },
                Point { x: 200, y: 900, size: 5 },
                Point { x: 900, y: 400, size: 1 },
                1.0 / 2000.0,
            ),
            Line::Bresenham(
                Point {
                    x: 1000,
                    y: 500,
                    size: 100,
                },
                Point {
                    x: 900,
                    y: 600,
                    size: 10,
                },
            ),
        ];
        draw_lines(&line, &mut pixels);
        let mut img = SVec::new(Shape::new(1000, 1000, None), ImgData::U8(vec![0; 1000 * 1000]));
        let img_data = img.get_data_mut::<u8>().unwrap();
        for pixel in pixels.iter() {
            let pixel = (min(1000 - 1, pixel.0), min(1000 - 1, pixel.1));
            img_data[pixel.1 * 1000 + pixel.0] = 255
        }
        svec_save(img, "test.png");
        // println!("{:?}", pixels);
    }
}
