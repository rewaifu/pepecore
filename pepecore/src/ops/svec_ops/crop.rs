use pepecore_array::error::Error;
use pepecore_array::{PixelType, SVec, Shape};

pub fn crop(img: &mut SVec, x: usize, y: usize, w: usize, h: usize) -> Result<(), Error> {
    let (img_h, img_w, opt_c) = img.shape.get_shape();
    let c = opt_c.unwrap_or(1);

    if x + w > img_w || y + h > img_h {
        return Err(Error::OutOfBounds);
    }

    match img.pixel_type() {
        PixelType::F32 => {
            let data = img.get_mut_vec::<f32>()?;

            let mut write_index = 0;

            for row in 0..h {
                let read_start = ((y + row) * img_w + x) * c;
                let read_end = read_start + w * c;

                data.copy_within(read_start..read_end, write_index);
                write_index += w * c;
            }

            data.truncate(w * h * c);
        }
        PixelType::U16 => {
            let data = img.get_mut_vec::<u16>()?;

            let mut write_index = 0;

            for row in 0..h {
                let read_start = ((y + row) * img_w + x) * c;
                let read_end = read_start + w * c;

                data.copy_within(read_start..read_end, write_index);
                write_index += w * c;
            }

            data.truncate(w * h * c);
        }
        PixelType::U8 => {
            let data = img.get_mut_vec::<u8>()?;

            let mut write_index = 0;

            for row in 0..h {
                let read_start = ((y + row) * img_w + x) * c;
                let read_end = read_start + w * c;

                data.copy_within(read_start..read_end, write_index);
                write_index += w * c;
            }

            data.truncate(w * h * c);
        }
    }

    img.shape = Shape::new(h, w, Some(c));

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::enums::ImgColor;
    use crate::ops::read::read::read_in_path;
    use crate::ops::save::save::svec_save;
    use std::path::Path;

    #[test]
    fn test_crop() {
        let img_path = Path::new(r"C:\Users\zzzcx\Downloads\79907108_p0.jpg");
        let mut img = read_in_path(img_path, ImgColor::DYNAMIC).expect("read_in_path failed");

        let _ = super::crop(&mut img, 200, 200, 400, 400);

        svec_save(img, "temp.png").expect("save failed");
    }
}
