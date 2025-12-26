pub fn min_max_uniform(img: &[u8], color_count: usize) -> Vec<f32> {
    if img.is_empty() || color_count == 0 {
        return vec![];
    }

    // 1. Поиск границ (быстрый проход)
    let (mut r_min, mut r_max) = (img[0], img[0]);
    let (mut g_min, mut g_max) = (img[1], img[1]);
    let (mut b_min, mut b_max) = (img[2], img[2]);

    for c in img.chunks_exact(3) {
        if c[0] < r_min {
            r_min = c[0];
        } else if c[0] > r_max {
            r_max = c[0];
        }
        if c[1] < g_min {
            g_min = c[1];
        } else if c[1] > g_max {
            g_max = c[1];
        }
        if c[2] < b_min {
            b_min = c[2];
        } else if c[2] > b_max {
            b_max = c[2];
        }
    }

    // 2. Жадное распределение уровней по приоритетам
    let mut r_l: usize = 1;
    let mut g_l: usize = 1;
    let mut b_l: usize = 1;

    loop {
        // Пробуем увеличить G (приоритет 1)
        if (g_l + 1) * r_l * b_l <= color_count {
            g_l += 1;
        } else {
            break;
        } // Если даже G не лезет, выходим

        // Пробуем увеличить R (приоритет 2)
        if (r_l + 1) * g_l * b_l <= color_count {
            r_l += 1;
        }

        // Пробуем увеличить B (приоритет 3)
        if (b_l + 1) * g_l * r_l <= color_count {
            b_l += 1;
        }
    }

    // 3. Генерация палитры
    let mut palette = Vec::with_capacity(r_l * g_l * b_l * 3);
    for r in 0..r_l {
        let rv = lerp(r_min, r_max, r, r_l);
        for g in 0..g_l {
            let gv = lerp(g_min, g_max, g, g_l);
            for b in 0..b_l {
                palette.push(rv);
                palette.push(gv);
                palette.push(lerp(b_min, b_max, b, b_l));
            }
        }
    }
    palette
}

#[inline(always)]
fn lerp(min: u8, max: u8, step: usize, total: usize) -> f32 {
    if total <= 1 {
        return min as f32;
    }
    let t = step as f32 / (total - 1) as f32;
    ((min as f32) * (1.0 - t) + (max as f32) * t) / 255.0
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
        let mi = min_max_uniform(data, 32);
        // for (i, c) in palette.iter().enumerate() {
        //     println!("  {}: RGB({}, {}, {})", i, c.0, c.1, c.2);
        // }
    }
}
