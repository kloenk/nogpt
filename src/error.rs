use crate::GPT;
use core::convert::Infallible;

#[derive(err_derive::Error, Debug)]
pub enum GPTError {
    #[cfg(any(feature = "std", doc))]
    #[error(display = "io: {}", _0)]
    Io(#[source] std::io::Error),

    #[cfg(any(feature = "alloc", doc))]
    #[error(display = "Failed to reserve memory")]
    TryReserveError(#[source] alloc::collections::TryReserveError),

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

    #[error(display = "Invalid data")]
    InvalidData,

    #[error(display = "Data not long enough")]
    UnexpectedEOF,

    #[error(display = "No gpt table could be found")]
    NoGPT,

    #[error(display = "MBR partition is not valid")]
    InvalidMbr,

    #[error(display = "MBR partitions are overlapping")]
    OverlappingPartitions,
}

impl From<Infallible> for GPTError {
    fn from(_: Infallible) -> Self {
        unreachable!("Infallible can never happen")
    }
}

#[derive(err_derive::Error)]
pub enum GPTParseError<T: Sized> {
    #[error(display = "{}", _0)]
    Error(GPTError),

    #[error(display = "{} header is invalid: {}", _1, _2)]
    BrokenHeader(crate::GPT<T>, crate::header::GptHeaderType, GPTError),
}

impl<E, T> From<E> for GPTParseError<T>
where
    GPTError: From<E>,
    T: Sized,
{
    fn from(error: E) -> Self {
        Self::Error(error.into())
    }
}

impl<T: Sized> core::fmt::Debug for GPTParseError<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            GPTParseError::Error(e) => core::fmt::Debug::fmt(e, f),
            GPTParseError::BrokenHeader(_, h, e) => write!(f, "GptParserError({:?}, {:?})", h, e),
        }
    }
}

impl<T: Sized> Into<GPTError> for GPTParseError<T> {
    fn into(self) -> GPTError {
        match self {
            GPTParseError::Error(e) => e,
            GPTParseError::BrokenHeader(_, _, e) => e,
        }
    }
}

pub type Result<T, E = GPTError> = core::result::Result<T, E>;

pub trait GptRepair<T: Sized> {
    fn fail(self) -> Result<GPT<T>>;

    // TODO: implement helper methods to help repair a gpt partition
}

impl<T: Sized> GptRepair<T> for Result<GPT<T>, GPTParseError<T>> {
    fn fail(self) -> Result<GPT<T>> {
        match self {
            Ok(v) => Ok(v),
            Err(GPTParseError::Error(e)) => Err(e),
            Err(GPTParseError::BrokenHeader(_, _, e)) => Err(e),
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
