use crate::errors::DecodeError;
use crate::ops::read::decode;
use jpeg_encoder::{Encoder as JpegEncoder, EncodingError as JpegEncodingError};
use pepecore_array::SVec;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    SVecError(#[from] pepecore_array::error::Error),

    #[error(transparent)]
    DecodeError(#[from] DecodeError),

    #[error(transparent)]
    JpegEncodingError(#[from] JpegEncodingError),
}

pub use jpeg_encoder::ColorType as JpegColorType;
pub use jpeg_encoder::SamplingFactor as JpegSamplingFactor;

#[derive(Debug)]
pub struct JpegEncodeOptions {
    pub quality: u8,
    pub progressive: bool,
    pub sampling_factor: JpegSamplingFactor,
}

impl Default for JpegEncodeOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl JpegEncodeOptions {
    pub fn new() -> Self {
        Self {
            quality: 100,
            progressive: true,
            sampling_factor: JpegSamplingFactor::R_4_2_0,
        }
    }

    pub fn set_quality(&mut self, quality: u8) {
        self.quality = quality;
    }

    pub fn set_progressive(&mut self, progressive: bool) {
        self.progressive = progressive;
    }

    pub fn set_sampling_factor(&mut self, sampling_factor: JpegSamplingFactor) {
        self.sampling_factor = sampling_factor;
    }
}

#[derive(Debug, Default)]
pub struct Encoder;

impl Encoder {
    pub fn encode_jpeg(img: &mut SVec, options: JpegEncodeOptions) -> Result<SVec, Error> {
        let (h, w, c) = img.shape();
        img.as_u8();

        let img_data = img.get_data::<u8>()?;

        let mut buffer = Vec::with_capacity(img_data.len());

        let mut encoder = JpegEncoder::new(&mut buffer, options.quality);

        encoder.set_progressive(options.progressive);
        encoder.set_sampling_factor(options.sampling_factor);

        let color_type = match c {
            Some(4) => JpegColorType::Rgba,
            Some(3) => JpegColorType::Rgb,
            Some(1) | None => JpegColorType::Luma,
            _ => unreachable!(),
        };

        encoder.encode(img_data, w as u16, h as u16, color_type)?;

        let result = match color_type {
            JpegColorType::Luma => decode::img_gray_decode(&buffer),
            JpegColorType::Rgb => decode::img_rgb_decode(&buffer),
            JpegColorType::Rgba => decode::img_rgba_decode(&buffer),
            _ => unreachable!(),
        }?;

        Ok(result)
    }
}
