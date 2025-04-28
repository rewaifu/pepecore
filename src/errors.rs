use thiserror::Error;

#[derive(Error, Debug)]
pub enum DecodeError {
    #[error("PsdDecode Error: {0}")]
    PsdDecodingError(String),
    #[error("ImgDecode Error: {0}")]
    ImgDecodingError(String),
    #[error("File open Error: {0}")]
    FileOpenError(String),
}
#[derive(Error, Debug)]
pub enum SaveError {
    #[error("Rgb save Error: {0}")]
    RGBSaveError(String),
    #[error("Gray save Error: {0}")]
    GraySaveError(String),
    #[error("Unsupported Channel Save Error: channel - {0}")]
    UnsupportedChannelSaveError(String),
}

#[derive(Debug)]
pub enum SVecError {
    TypeMismatch { expected: &'static str, actual: &'static str },
}
#[derive(Debug, Error)]
pub enum HalftoneError {
    #[error("Failed to get mutable image data: {0}")]
    GetDataError(String),

    #[error("Image has no channels")]
    NoChannelsError,

    #[error("Mismatch between number of dot sizes ({0}) and number of channels ({1})")]
    DotSizeMismatch(usize, usize),

    #[error("dot_circle returned invalid data: {0}")]
    DotCircleError(String),
}
