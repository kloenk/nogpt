#![cfg_attr(not(any(feature = "std", test, doc)), no_std)]
#![deny(unsafe_op_in_unsafe_fn)]

use block_device::BlockDevice;
use nom::HexDisplay;

pub use crate::error::{GptError, Result};
use crate::header::GptHeader;

mod error;
mod header;
#[cfg(any(feature = "std", test, doc))]
pub mod std;

pub struct Gpt<T> {
    block: T,
}

impl<T> Gpt<T>
where
    T: BlockDevice,
    GptError: From<T::Error>,
{
    pub fn open(block: T) -> Result<Self> {
        let mut buf = [0u8; 512];

        block.read(&mut buf, 1, 1)?;

        let m_header = GptHeader::parse(&buf)?;
        println!("{:?}", m_header);

        todo!()
    }
}
