#[derive(Debug)]
pub enum Error {
    TypeMismatch { expected: &'static str, actual: &'static str },
}
