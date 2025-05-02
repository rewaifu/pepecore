#[derive(Clone, Copy, Debug)]
pub enum ImgColor {
    GRAY,
    RGB,
    RGBA,
    GRAYA,
    DYNAMIC,
}
#[allow(non_camel_case_types)]
pub enum CVTColor {
    RGB2Gray_2020,
    RGB2Gray_601,
    RGB2Gray_709,
    RGB2YCbCR_2020,
    RGB2YCbCR_601,
    RGB2YCbCR_709,
    YCbCR2RGB_2020,
    YCbCR2RGB_601,
    YCbCR2RGB_709,
    RGB2CMYK,
    CMYK2RGB,
    RGB2BGR,
    BGR2RGB,
    Gray2RGB,
    RGB2Bayer_RGGB,
    RGB2Bayer_BGGR,
    RGB2Bayer_GRBG,
    RGB2Bayer_GBRG,
}
#[derive(Clone, Debug)]
pub enum ImgData {
    F32(Vec<f32>),
    U8(Vec<u8>),
    U16(Vec<u16>),
}
#[derive(Clone, Debug)]
pub enum DotType {
    CIRCLE,
    CROSS,
    ELLIPSE,
    LINE,
    INVLINE,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PixelType {
    U8,
    U16,
    F32,
}

impl ImgData {
    pub fn pixel_type(&self) -> PixelType {
        match self {
            ImgData::U8(_) => PixelType::U8,
            ImgData::U16(_) => PixelType::U16,
            ImgData::F32(_) => PixelType::F32,
        }
    }
}
impl From<Vec<u8>> for ImgData {
    fn from(v: Vec<u8>) -> Self {
        ImgData::U8(v)
    }
}

impl From<Vec<u16>> for ImgData {
    fn from(v: Vec<u16>) -> Self {
        ImgData::U16(v)
    }
}

impl From<Vec<f32>> for ImgData {
    fn from(v: Vec<f32>) -> Self {
        ImgData::F32(v)
    }
}
