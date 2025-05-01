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
}

#[derive(Clone, Debug)]
pub enum DotType {
    CIRCLE,
    CROSS,
    ELLIPSE,
    LINE,
    INVLINE,
}
