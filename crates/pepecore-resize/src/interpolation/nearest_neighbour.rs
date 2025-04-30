use std::ptr;
use crate::error::Error;
use pepecore::array::svec::{SVec, Shape};
use pepecore::enums::ImgData;

fn resize_nn_const<T: Copy, const C: usize>(
    src: *const T,
    src_w: usize,
    y_idx: &[usize],
    x_idx: &[usize],
    dst_ptr: *mut T,
) {
    let mut dst_ptr = dst_ptr;
    for &sy in y_idx {
        unsafe {
            let row_src = src.add(sy * src_w * C);
            for &sx in x_idx {
                let pix_src = row_src.add(sx * C);
                ptr::copy_nonoverlapping(pix_src, dst_ptr, C);
                dst_ptr = dst_ptr.add(C);
            }
        }
    }
}

fn resize_nearest<T: Copy + Default + Send + Sync>(
    src: &[T],
    src_w: usize,
    src_h: usize,
    channels: usize,
    dst_w: usize,
    dst_h: usize,
) -> Vec<T> {
    let len = dst_w * dst_h * channels;
    let mut dst: Vec<T> = Vec::with_capacity(len);
    
    let y_idx: Vec<usize> = (0..dst_h)
        .map(|y| ((y * src_h + dst_h / 2) / dst_h).min(src_h - 1))
        .collect();
    
    let x_idx: Vec<usize> = (0..dst_w)
        .map(|x| ((x * src_w + dst_w / 2) / dst_w).min(src_w - 1))
        .collect();

    let dst_ptr = dst.as_mut_ptr();

    match channels {
        1 => resize_nn_const::<T, 1>(
            src.as_ptr(), src_w,
            &y_idx, &x_idx, dst_ptr
        ),
        3 => resize_nn_const::<T, 3>(
            src.as_ptr(), src_w,
            &y_idx, &x_idx, dst_ptr
        ),
        4 => resize_nn_const::<T, 4>(
            src.as_ptr(), src_w,
            &y_idx, &x_idx, dst_ptr
        ),
        _ => panic!("Unsupported channel count"),
    }
    
    unsafe {
        dst.set_len(len);
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
    use crate::utility::calculate_dst_size;
    use pepecore::enums::ImgColor;
    use std::path::PathBuf;

    #[test]
    fn test_nearest_neighbour() {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");

        let input_path = PathBuf::from(manifest_dir).join("tests").join("assets").join("test.png");

        assert!(input_path.exists(), "Test image not found at {:?}", input_path);

        let src_img = pepecore::ops::read::read::read_in_path(&input_path, ImgColor::DYNAMIC).expect("Read image failed");
        let (src_height, src_width, _) = src_img.shape();

        let start = std::time::Instant::now();

        let dst_size = calculate_dst_size(src_height, src_width, None, None, Some(0.25));
        let dst_img = nearest_neighbour(&src_img, dst_size.0, dst_size.1).expect("Nearest neighbour resize failed");

        println!("Time elapsed: {}µs", start.elapsed().as_micros());

        let output_path = PathBuf::from(manifest_dir).join("tests").join("output");

        std::fs::create_dir_all(&output_path).expect("Failed to create output directory");

        pepecore::ops::save::save::svec_save(dst_img, &output_path.join("test_nearest_neighbour.png")).expect("Save failed");
    }
}
