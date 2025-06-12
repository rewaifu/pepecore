use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Type mismatch: expected {expected}, found {actual}")]
    TypeMismatch { expected: &'static str, actual: &'static str },
    #[error("No channels available")]
    NoChannelsError,
    #[error("Dimensions out of bounds")]
    OutOfBounds,
}
