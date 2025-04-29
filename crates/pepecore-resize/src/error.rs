#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Image has no channels")]
    NoChannelsError,
}
