use crate::array::svec::SVec;
use crate::enums::ImgColor;
use crate::errors::DecodeError;
use crate::errors::DecodeError::FileOpenError;
use crate::ops::read::decode::{img_din_decode, img_gray_decode, img_graya_decode, img_rgb_decode, img_rgba_decode, psd_din_decode, psd_gray_decode, psd_graya_decode, psd_rgb_decode, psd_rgba_decode};
use filebuffer::FileBuffer;

pub fn read_in_path(path: &str, img_color: ImgColor) -> Result<SVec, DecodeError> {
    let img_buffer = FileBuffer::open(path).map_err(|e| FileOpenError(format!("Path: {} FileBuffer error: {:?}", path, e)))?;
    Ok(match &img_buffer[..4] {
        [56, 66, 80, 83] => match img_color {
            ImgColor::DYNAMIC => psd_din_decode(&img_buffer)?,
            ImgColor::GRAY => psd_gray_decode(&img_buffer)?,
            ImgColor::RGB => psd_rgb_decode(&img_buffer)?,
            ImgColor::RGBA => psd_rgba_decode(&img_buffer)?,
            ImgColor::GRAYA => psd_graya_decode(&img_buffer)?,
        },
        _ => match img_color {
            ImgColor::DYNAMIC => img_din_decode(&img_buffer)?,
            ImgColor::GRAY => img_gray_decode(&img_buffer)?,
            ImgColor::RGB => img_rgb_decode(&img_buffer)?,
            ImgColor::RGBA => img_rgba_decode(&img_buffer)?,
            ImgColor::GRAYA => img_graya_decode(&img_buffer)?,
        },
    })
}
pub fn read_in_buffer(img_buffer: &[u8], img_color: ImgColor) -> Result<SVec, DecodeError> {
    Ok(match &img_buffer[..4] {
        [56, 66, 80, 83] => match img_color {
            ImgColor::DYNAMIC => psd_din_decode(&img_buffer)?,
            ImgColor::GRAY => psd_gray_decode(&img_buffer)?,
            ImgColor::RGB => psd_rgb_decode(&img_buffer)?,
            ImgColor::RGBA => psd_rgba_decode(&img_buffer)?,
            ImgColor::GRAYA => psd_graya_decode(&img_buffer)?,
        },
        _ => match img_color {
            ImgColor::DYNAMIC => img_din_decode(&img_buffer)?,
            ImgColor::GRAY => img_gray_decode(&img_buffer)?,
            ImgColor::RGB => img_rgb_decode(&img_buffer)?,
            ImgColor::RGBA => img_rgba_decode(&img_buffer)?,
            ImgColor::GRAYA => img_gray_decode(&img_buffer)?,
        },
    })
}
