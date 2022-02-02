#![cfg_attr(not(any(feature = "std", test, doc)), no_std)]
#![deny(unsafe_op_in_unsafe_fn)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

extern crate core;

use block_device::BlockDevice;

pub use crate::error::{GptError, GptParseError, GptRepair, Result};
use crate::header::{GptHeader, GptHeaderType};

pub const BLOCK_SIZE: u32 = 512;
//pub const BLOCK_SIZE: u32 = 4096;
pub const DEFAULT_PARTTABLE_SIZE: u32 = 16384;
pub const DEFAULT_PARTTABLE_BLOCKS: u32 = DEFAULT_PARTTABLE_SIZE / BLOCK_SIZE;

macro_rules! read_le_bytes {
    ($in:tt, $size:tt, $pos:expr) => {
        // TODO: remove unwrap
        $size::from_le_bytes(($in[$pos]).try_into().unwrap())
    };

    ($in:tt, $pos:expr) => {
        ($in[$pos]).try_into().unwrap()
    };
}
use crate::error::ParseGuidError;
use crate::part::{DefaultGptTypeGuid, GptPartHeader, GptTypeGuid};
pub(crate) use read_le_bytes; // trick to export to crate

pub mod error;
pub mod header;
pub mod part;
#[cfg(any(feature = "std", test, doc))]
pub mod std;

pub struct Gpt<T> {
    block: T,
    header: GptHeader,
}

impl<T> Gpt<T>
where
    T: BlockDevice,
    GptError: From<T::Error>,
{
    pub fn open(block: T) -> Result<Self, GptParseError<T>> {
        #[cfg(not(feature = "alloc"))]
        let mut buf = [0u8; DEFAULT_PARTTABLE_SIZE as usize];

        #[cfg(feature = "alloc")]
        let mut buf = {
            let mut buf = Vec::with_capacity(DEFAULT_PARTTABLE_SIZE as usize);
            buf.try_reserve_exact(DEFAULT_PARTTABLE_SIZE as usize)?; // Catch allocation errors
            buf.resize(DEFAULT_PARTTABLE_SIZE as usize, 0);
            buf
        };

        // TODO: read address from MBR
        let header_lba = 1;
        block.read(&mut buf, header_lba, 1)?;

        let m_header = GptHeader::parse(&buf)?;

        let p_table_size = m_header.size_of_p_entry * m_header.num_parts;
        #[cfg(not(feature = "alloc"))]
        if p_table_size > DEFAULT_PARTTABLE_SIZE {
            return Err(GptError::NoAllocator.into());
        }

        #[cfg(feature = "alloc")]
        if p_table_size > buf.len() as u32 {
            buf.try_reserve_exact(p_table_size as usize - buf.len())?; // Catch allocation errors
            buf.resize(p_table_size as usize, 0);
        }

        let blocks = if p_table_size > DEFAULT_PARTTABLE_SIZE as u32 {
            p_table_size / BLOCK_SIZE + 1 // TODO: round up properly
        } else {
            DEFAULT_PARTTABLE_BLOCKS
        };

        block.read(&mut buf, m_header.p_entry_lba as usize, blocks as usize)?;

        let m_header_valid = m_header.validate(header_lba as u64, &buf);

        block.read(&mut buf, m_header.other_lba as usize, 1)?;
        let b_header = GptHeader::parse(&buf)?;

        block.read(&mut buf, b_header.p_entry_lba as usize, blocks as usize)?;

        let b_header_valid = b_header.validate(m_header.other_lba as u64, &buf);

        if m_header_valid.is_err() || b_header_valid.is_err() {
            if m_header_valid.is_ok() {
                return Err(GptParseError::BrokenHeader(
                    Self {
                        block,
                        header: m_header,
                    },
                    GptHeaderType::Backup,
                    b_header_valid.unwrap_err(),
                ));
            } else if b_header_valid.is_ok() {
                return Err(GptParseError::BrokenHeader(
                    Self {
                        block,
                        header: b_header,
                    },
                    GptHeaderType::Main,
                    m_header_valid.unwrap_err(),
                ));
            } else {
                return Err(GptError::NoGpt.into());
            }
        }

        Ok(Self {
            block,
            header: m_header,
        })
    }

    pub fn get_partition<PT>(&self, idx: u32) -> Result<GptPartHeader<PT>>
    where
        PT: GptTypeGuid,
        GptError: From<<PT as TryFrom<[u8; 16]>>::Error>,
        GptError: From<<PT as TryInto<[u8; 16]>>::Error>,
    {
        if idx >= self.header.num_parts {
            return Err(GptError::NoGpt);
        }

        #[cfg(not(feature = "alloc"))]
        let mut buf = [0u8; DEFAULT_PARTTABLE_SIZE as usize];

        #[cfg(feature = "alloc")]
        let mut buf = {
            let mut buf = Vec::with_capacity(DEFAULT_PARTTABLE_SIZE as usize);
            buf.try_reserve_exact(DEFAULT_PARTTABLE_SIZE as usize)?; // Catch allocation errors
            buf.resize(DEFAULT_PARTTABLE_SIZE as usize, 0);
            buf
        };

        let p_table_size = self.header.size_of_p_entry * self.header.num_parts;
        #[cfg(not(feature = "alloc"))]
        if p_table_size > DEFAULT_PARTTABLE_SIZE {
            return Err(GptError::NoAllocator.into());
        }

        #[cfg(feature = "alloc")]
        if p_table_size > buf.len() as u32 {
            buf.try_reserve_exact(p_table_size as usize - buf.len())?; // Catch allocation errors
            buf.resize(p_table_size as usize, 0);
        }

        let blocks = if p_table_size > DEFAULT_PARTTABLE_SIZE as u32 {
            p_table_size / BLOCK_SIZE + 1 // TODO: round up properly
        } else {
            DEFAULT_PARTTABLE_BLOCKS
        };

        self.block
            .read(&mut buf, self.header.p_entry_lba as usize, blocks as usize)?;

        let offset: u64 = self.header.size_of_p_entry as u64 * idx as u64;

        GptPartHeader::parse(&buf[offset as usize..])
    }
}

//0FC63DAF-8483-4772-8E79-3D69D8477DE4
#[repr(packed)]
#[derive(Eq, PartialEq, Clone, Copy)]
pub struct GUID {
    data1: u32,
    data2: u16,
    data3: u16,
    data4: [u8; 8],
}

impl GUID {
    /// Create a new guid based on the four data blocks.
    pub const fn new(data1: u32, data2: u16, data3: u16, data4: u64) -> Self {
        Self {
            data1,
            data2,
            data3,
            data4: data4.to_be_bytes(),
        }
    }

    pub fn as_bytes(&self) -> [u8; 16] {
        let mut ret = [0u8; 16];
        let mut i = 0;

        for b in self.data1.to_le_bytes() {
            ret[i] = b;
            i += 1;
        }
        for b in self.data2.to_le_bytes() {
            ret[i] = b;
            i += 1;
        }
        for b in self.data3.to_le_bytes() {
            ret[i] = b;
            i += 1;
        }
        for b in self.data4 {
            ret[i] = b;
            i += 1;
        }

        ret
    }

    pub const UNUSED: Self = Self::new(0, 0, 0, 0);
    pub const ESP: Self = Self::new(0xC12A7328, 0xF81F, 0x11D2, 0xBA4B00A0C93EC93B);
    pub const LEGACY_MBR: Self = Self::new(0x024DEE41, 0x33E7, 0x11D3, 0x9D690008C781F39F);
}

impl TryFrom<&[u8]> for GUID {
    type Error = GptError;

    fn try_from(value: &[u8]) -> core::result::Result<Self, Self::Error> {
        let v: [u8; 16] = value
            .get(0..16)
            .ok_or(GptError::NoGpt)?
            .try_into()
            .map_err(|_| GptError::NoGpt)?;

        Ok(Self::from(v))
    }
}

impl From<[u8; 16]> for GUID {
    fn from(buf: [u8; 16]) -> Self {
        let data1 = read_le_bytes!(buf, u32, 0..4);
        let data2 = read_le_bytes!(buf, u16, 4..6);
        let data3 = read_le_bytes!(buf, u16, 6..8);
        let data4 = read_le_bytes!(buf, 8..16);

        Self {
            data1,
            data2,
            data3,
            data4,
        }
    }
}

impl Into<[u8; 16]> for GUID {
    fn into(self) -> [u8; 16] {
        self.as_bytes()
    }
}

impl core::fmt::Debug for GUID {
    // TODO: optimize to not use for?
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // Cannot format without local copy, because of packed guaranties
        let data1 = self.data1;
        let data2 = self.data2;
        let data3 = self.data3;

        write!(f, "{:08X?}-{:04X?}-{:04X?}-", data1, data2, data3)?;
        for b in &self.data4[..2] {
            write!(f, "{:02X?}", b)?;
        }
        write!(f, "-")?;
        for b in &self.data4[2..] {
            write!(f, "{:02X?}", b)?;
        }
        Ok(())
    }
}

impl core::fmt::Display for GUID {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(self, f)
    }
}

const GUID_SEPARATOR: &'static str = "-";
impl core::str::FromStr for GUID {
    type Err = crate::error::ParseGuidError;

    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        use crate::error::ParseGuidError;

        if s.len() != 36 {
            return Err(ParseGuidError::InvalidLength);
        }
        if &s[8..9] != GUID_SEPARATOR
            || &s[13..14] != GUID_SEPARATOR
            || &s[18..19] != GUID_SEPARATOR
            || &s[23..24] != GUID_SEPARATOR
        {
            return Err(ParseGuidError::InvalidSeparator);
        }

        let data1 = u32::from_str_radix(&s[0..8], 16)?;
        let data2 = u16::from_str_radix(&s[9..13], 16)?;
        let data3 = u16::from_str_radix(&s[14..18], 16)?;
        let mut data4 = [0u8; 8];

        data4[0] = u8::from_str_radix(&s[19..21], 16)?;
        data4[1] = u8::from_str_radix(&s[21..23], 16)?;

        for i in 0..6 {
            data4[i + 2] = u8::from_str_radix(&s[i * 2 + 24..i * 2 + 26], 16)?;
        }

        Ok(Self {
            data1,
            data2,
            data3,
            data4,
        })
    }
}

#[cfg(test)]
mod test {
    use super::GUID;

    #[test]
    fn format_guid() {
        assert_eq!(
            GUID::UNUSED.to_string(),
            "00000000-0000-0000-0000-000000000000"
        );
        assert_eq!(
            GUID::ESP.to_string(),
            "C12A7328-F81F-11D2-BA4B-00A0C93EC93B"
        );
        assert_eq!(
            GUID::LEGACY_MBR.to_string(),
            "024DEE41-33E7-11D3-9D69-0008C781F39F"
        );
    }

    #[test]
    fn parse_guid() {
        assert_eq!(
            "00000000-0000-0000-0000-000000000000"
                .parse::<GUID>()
                .unwrap(),
            GUID::UNUSED
        );

        assert_eq!(
            "C12A7328-F81F-11D2-BA4B-00A0C93EC93B"
                .parse::<GUID>()
                .unwrap(),
            GUID::ESP
        );

        assert_eq!(
            "024DEE41-33E7-11D3-9D69-0008C781F39F"
                .parse::<GUID>()
                .unwrap(),
            GUID::LEGACY_MBR
        );
    }

    #[test]
    fn guid_to_string_parse() {
        assert_eq!(
            GUID::UNUSED.to_string().parse::<GUID>().unwrap(),
            GUID::UNUSED
        );

        assert_eq!(GUID::ESP.to_string().parse::<GUID>().unwrap(), GUID::ESP);

        assert_eq!(
            GUID::LEGACY_MBR.to_string().parse::<GUID>().unwrap(),
            GUID::LEGACY_MBR
        );
    }

    #[test]
    fn guid_to_bytes() {
        assert_eq!(GUID::UNUSED.as_bytes(), [0u8; 16]);
        assert_eq!(
            GUID::ESP.as_bytes(),
            [40, 115, 42, 193, 31, 248, 210, 17, 186, 75, 0, 160, 201, 62, 201, 59]
        );

        assert_eq!(
            GUID::LEGACY_MBR.as_bytes(),
            [65, 238, 77, 2, 231, 51, 211, 17, 157, 105, 0, 8, 199, 129, 243, 159]
        );
    }
}
