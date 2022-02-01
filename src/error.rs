#[derive(err_derive::Error, Debug)]
pub enum GptError {
    #[cfg(any(feature = "std", test, doc))]
    #[error(display = "io: {}", _0)]
    Io(#[source] std::io::Error),

    #[error(display = "Invalid gpt signature: {}", _0)]
    InvalidSignature(u64),
}

pub type Result<T, E = GptError> = core::result::Result<T, E>;
