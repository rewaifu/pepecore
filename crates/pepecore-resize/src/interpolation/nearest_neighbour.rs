use crate::error::Error;
use pepecore::array::svec::{SVec, Shape};
use pepecore::enums::ImgData;

fn resize_nearest<T: Copy>(
    src: &[T],
    src_width: usize,
    src_height: usize,
    channels: usize,
    dst_width: usize,
    dst_height: usize,
) -> Vec<T> {
    let x_ratio = src_width as f32 / dst_width as f32;
    let y_ratio = src_height as f32 / dst_height as f32;
    let mut dst = Vec::with_capacity(dst_width * dst_height * channels);

    for y in 0..dst_height {
        let yf = (y as f32 * y_ratio).round().clamp(0.0, (src_height - 1) as f32) as usize;
        for x in 0..dst_width {
            let xf = (x as f32 * x_ratio).round().clamp(0.0, (src_width - 1) as f32) as usize;
            let base_idx = (yf * src_width + xf) * channels;
            for c in 0..channels {
                dst.push(src[base_idx + c]);
            }
        }
    }

    dst
}

pub fn nearest_neighbour(img: &SVec, dst_height: usize, dst_width: usize) -> Result<SVec, Error> {
    let (src_height, src_width, channels_opt) = img.shape();
    let channels = channels_opt.ok_or(Error::NoChannelsError)?;

    let shape = Shape::new(dst_height, dst_width, channels_opt);
    let resized = match &img.data {
        ImgData::F32(src) => ImgData::F32(resize_nearest(src, src_width, src_height, channels, dst_width, dst_height)),

        ImgData::U8(src) => ImgData::U8(resize_nearest(src, src_width, src_height, channels, dst_width, dst_height)),
        
        ImgData::U16(src) => ImgData::U16(resize_nearest(src, src_width, src_height, channels, dst_width, dst_height)),
    };

    Ok(SVec::new(shape, resized))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pepecore::enums::ImgColor;
    use std::path::PathBuf;
    use crate::utility::calculate_dst_size;

    #[test]
    fn test_nearest_neighbour() {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        
        let input_path = PathBuf::from(manifest_dir)
            .join("tests")
            .join("assets")
            .join("test.jpg");
        
        assert!(
            input_path.exists(),
            "Test image not found at {:?}. Please place test.jpg in tests/assets/",
            input_path
        );
        
        let src_img = pepecore::ops::read::read::read_in_path(&input_path, ImgColor::DYNAMIC).expect("Read image failed");
        let (src_height, src_width, _) = src_img.shape();
        let dst_size = calculate_dst_size(src_height, src_width, None, None, Some(0.25));
        let dst_img = nearest_neighbour(&src_img, dst_size.0, dst_size.1).expect("Nearest neighbour resize failed");
        
        let output_path = PathBuf::from(manifest_dir)
            .join("tests")
            .join("output");
        
        std::fs::create_dir_all(&output_path).expect("Failed to create output directory");

        pepecore::ops::save::save::svec_save(dst_img, &output_path.join("test_nearest_neighbour.png")).expect("Save failed");
    }
}
