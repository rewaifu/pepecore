#[derive(Clone, Copy, Debug)]
pub enum ImgFormat {
    U8,
    F32,
    U16,
    DYNAMIC,
}
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
