#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ImgColor {
    GRAY,
    RGB,
    RGBA,
    GRAYA,
    DYNAMIC,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
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

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum DotType {
    CIRCLE,
    CROSS,
    ELLIPSE,
    LINE,
    INVLINE,
}
