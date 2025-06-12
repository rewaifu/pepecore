use pepecore::enums::CVTColor;
use pepecore::enums::ImgColor;
use pyo3::pyclass;

#[pyclass(name = "ImgColor")]
#[derive(Clone, Copy)]
pub enum ColorMode {
    GRAY,
    RGB,
    RGBA,
    GRAYA,
    DYNAMIC,
}

impl From<ColorMode> for ImgColor {
    fn from(value: ColorMode) -> Self {
        match value {
            ColorMode::GRAY => ImgColor::GRAY,
            ColorMode::RGB => ImgColor::RGB,
            ColorMode::RGBA => ImgColor::RGBA,
            ColorMode::GRAYA => ImgColor::GRAYA,
            ColorMode::DYNAMIC => ImgColor::DYNAMIC,
        }
    }
}

#[pyclass]
#[derive(Clone, Copy)]
pub enum ImgFormat {
    F32,
    U8,
    U16,
    DYNAMIC,
}

#[pyclass(name = "CVTColor")]
#[derive(Clone, Copy)]
#[allow(non_camel_case_types)]
pub enum ColorCVT {
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

impl From<ColorCVT> for CVTColor {
    fn from(value: ColorCVT) -> Self {
        match value {
            ColorCVT::RGB2Gray_2020 => CVTColor::RGB2Gray_2020,
            ColorCVT::RGB2Gray_601 => CVTColor::RGB2Gray_601,
            ColorCVT::RGB2Gray_709 => CVTColor::RGB2Gray_709,
            ColorCVT::RGB2YCbCR_2020 => CVTColor::RGB2YCbCR_2020,
            ColorCVT::RGB2YCbCR_601 => CVTColor::RGB2YCbCR_601,
            ColorCVT::RGB2YCbCR_709 => CVTColor::RGB2YCbCR_709,
            ColorCVT::YCbCR2RGB_2020 => CVTColor::YCbCR2RGB_2020,
            ColorCVT::YCbCR2RGB_601 => CVTColor::YCbCR2RGB_601,
            ColorCVT::YCbCR2RGB_709 => CVTColor::YCbCR2RGB_709,
            ColorCVT::RGB2CMYK => CVTColor::RGB2CMYK,
            ColorCVT::CMYK2RGB => CVTColor::CMYK2RGB,
            ColorCVT::RGB2BGR => CVTColor::RGB2BGR,
            ColorCVT::BGR2RGB => CVTColor::BGR2RGB,
            ColorCVT::Gray2RGB => CVTColor::Gray2RGB,
        }
    }
}
