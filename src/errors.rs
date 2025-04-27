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
