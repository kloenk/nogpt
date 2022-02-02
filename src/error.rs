use crate::header::GptHeaderType;
use crate::Gpt;
use core::convert::Infallible;

#[derive(err_derive::Error, Debug)]
pub enum GptError {
    #[cfg(any(feature = "std", test, doc))]
    #[error(display = "io: {}", _0)]
    Io(#[source] std::io::Error),

    #[cfg(feature = "alloc")]
    #[error(display = "Failed to reserve memory")]
    Alloc(#[source] alloc::collections::TryReserveError),

    #[error(display = "No allocator provided, cannot read bigger data")]
    NoAllocator,

    #[error(display = "Invalid gpt signature: {}", _0)]
    InvalidSignature(u64),

    #[error(display = "Failed to validate crc. Got {} but expected {}", _0, _1)]
    InvalidCrcHeader(u32, u32),
    #[error(display = "Failed to validate crc. Got {} but expected {}", _0, _1)]
    InvalidCrcParts(u32, u32),

    #[error(
        display = "The LBA {} is invalid, or does not contain the expected data",
        _0
    )]
    InvalidLba(u64),

    #[error(
        display = "The partition table is to short, it should be {} bytes long",
        _0
    )]
    PartitionTableToShort(u32),

    #[error(display = "No gpt table could be found")]
    NoGpt,
}

impl From<Infallible> for GptError {
    fn from(_: Infallible) -> Self {
        unreachable!("Infallible can never happen")
    }
}

#[derive(err_derive::Error)]
pub enum GptParseError<T: Sized> {
    #[error(display = "{}", _0)]
    Error(GptError),

    #[error(display = "{} header is invalid: {}", _1, _2)]
    BrokenHeader(crate::Gpt<T>, crate::header::GptHeaderType, GptError),
}

impl<E, T> From<E> for GptParseError<T>
where
    GptError: From<E>,
    T: Sized,
{
    fn from(error: E) -> Self {
        Self::Error(error.into())
    }
}

impl<T: Sized> core::fmt::Debug for GptParseError<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            GptParseError::Error(e) => core::fmt::Debug::fmt(e, f),
            GptParseError::BrokenHeader(_, h, e) => write!(f, "GptParserError({:?}, {:?})", h, e),
        }
    }
}

impl<T: Sized> Into<GptError> for GptParseError<T> {
    fn into(self) -> GptError {
        match self {
            GptParseError::Error(e) => e,
            GptParseError::BrokenHeader(_, _, e) => e,
        }
    }
}

pub type Result<T, E = GptError> = core::result::Result<T, E>;

pub trait GptRepair<T: Sized> {
    fn fail(self) -> Result<Gpt<T>>;

    // TODO: implement helper methods to help repair a gpt partition
}

impl<T: Sized> GptRepair<T> for Result<Gpt<T>, GptParseError<T>> {
    fn fail(self) -> Result<Gpt<T>> {
        match self {
            Ok(v) => Ok(v),
            Err(GptParseError::Error(e)) => Err(e),
            Err(GptParseError::BrokenHeader(_, _, e)) => Err(e),
        }
    }
}

#[derive(err_derive::Error, Debug)]
pub enum ParseGuidError {
    #[error(display = "Failed to parse block: {}", _0)]
    ParseIntError(#[source] core::num::ParseIntError),

    #[error(display = "Invalid GUID Length")]
    InvalidLength,

    #[error(display = "Invalid GUID Separator")]
    InvalidSeparator,
}
