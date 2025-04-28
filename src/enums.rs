#[derive(Clone, Copy, Debug)]
pub enum ImgColor {
    GRAY,
    RGB,
    RGBA,
    GRAYA,
    DYNAMIC,
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
    /// Возвращает, какой вариант enum-а внутри `self`.
    pub fn pixel_type(&self) -> PixelType {
        match self {
            ImgData::U8(_) => PixelType::U8,
            ImgData::U16(_) => PixelType::U16,
            ImgData::F32(_) => PixelType::F32,
        }
    }
}
