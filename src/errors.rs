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
