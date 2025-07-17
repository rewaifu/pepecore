use fast_image_resize::{FilterType, ResizeAlg};
use pepecore::enums::ImgColor;
use pepecore::enums::{CVTColor, DotType};
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
    RGB2Bayer_BGGR,
    RGB2Bayer_RGGB,
    RGB2Bayer_GBRG,
    RGB2Bayer_GRBG,
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
            ColorCVT::RGB2Bayer_BGGR => CVTColor::RGB2Bayer_BGGR,
            ColorCVT::RGB2Bayer_RGGB => CVTColor::RGB2Bayer_RGGB,
            ColorCVT::RGB2Bayer_GBRG => CVTColor::RGB2Bayer_GBRG,
            ColorCVT::RGB2Bayer_GRBG => CVTColor::RGB2Bayer_GRBG,
        }
    }
}

#[pyclass(name = "DotType")]
#[derive(Clone, Copy)]
pub enum DotTypePy {
    CIRCLE,
    CROSS,
    ELLIPSE,
    LINE,
    INVLINE,
}

impl From<DotTypePy> for DotType {
    fn from(value: DotTypePy) -> Self {
        match value {
            DotTypePy::CIRCLE => DotType::CIRCLE,
            DotTypePy::CROSS => DotType::CROSS,
            DotTypePy::ELLIPSE => DotType::ELLIPSE,
            DotTypePy::LINE => DotType::LINE,
            DotTypePy::INVLINE => DotType::INVLINE,
        }
    }
}

#[derive(Clone)]
#[pyclass]
pub enum TypeNoise {
    PERLIN = 0,
    SIMPLEX = 1,
    OPENSIMPLEX = 2,
    SUPERSIMPLEX = 3,
    PERLINSURFLET = 4,
}
#[derive(Clone)]
#[pyclass]
pub enum ResizesFilter {
    Box,
    Bilinear,
    Hamming,
    CatmullRom,
    Mitchell,
    Gaussian,
    Lanczos3,
}
#[derive(Clone)]
#[pyclass]
pub enum ResizesAlg {
    Nearest(),
    Conv(ResizesFilter),
    Interpolation(ResizesFilter),
    SuperSampling(ResizesFilter, u8),
}

impl From<ResizesFilter> for FilterType {
    fn from(value: ResizesFilter) -> Self {
        match value {
            ResizesFilter::Box => FilterType::Box,
            ResizesFilter::Bilinear => FilterType::Bilinear,
            ResizesFilter::Hamming => FilterType::Hamming,
            ResizesFilter::CatmullRom => FilterType::CatmullRom,
            ResizesFilter::Mitchell => FilterType::Mitchell,
            ResizesFilter::Gaussian => FilterType::Gaussian,
            ResizesFilter::Lanczos3 => FilterType::Lanczos3,
        }
    }
}
impl From<ResizesAlg> for ResizeAlg {
    fn from(value: ResizesAlg) -> Self {
        match value {
            ResizesAlg::Nearest() => ResizeAlg::Nearest,
            ResizesAlg::Conv(filter) => ResizeAlg::Convolution(filter.into()),
            ResizesAlg::Interpolation(filter) => ResizeAlg::Interpolation(filter.into()),
            ResizesAlg::SuperSampling(filter, sampling) => ResizeAlg::SuperSampling(filter.into(), sampling),
        }
    }
}
