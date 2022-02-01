use block_device::BlockDevice;

use crate::{GptError, Result};

const EFI_SIGNATURE: u64 = 0x5452415020494645;

macro_rules! read_le_bytes {
    ($in:tt, $size:tt, $pos:expr) => {
        // TODO: remove unwrap
        $size::from_le_bytes(($in[$pos]).try_into().unwrap())
    };
}

#[derive(Debug)]
pub struct GptHeader {
    pub rev: u32,
    pub size: u32,
    //crc32: u32,
    pub my_lba: u64,
    pub other_lba: u64,

    pub first_lba: u64,
    pub last_lba: u64,

    pub guid: [u8; 16],

    pub p_entry_lba: u64,

    pub num_parts: u32,
    pub size_of_p_entries: u32,
    pub p_crc32: u32,
}

impl GptHeader {
    pub fn parse(buf: &[u8]) -> Result<Self> {
        let sig = read_le_bytes!(buf, u64, 0..8);
        if sig != EFI_SIGNATURE {
            return Err(GptError::InvalidSignature(sig));
        }

        let rev = read_le_bytes!(buf, u32, 8..12);
        let size = read_le_bytes!(buf, u32, 12..16);
        let crc32 = read_le_bytes!(buf, u32, 16..20);

        let my_lba = read_le_bytes!(buf, u64, 24..32);
        let other_lba = read_le_bytes!(buf, u64, 32..40);

        let first_lba = read_le_bytes!(buf, u64, 40..48);
        let last_lba = read_le_bytes!(buf, u64, 48..56);

        // TODO: remove unwrap
        let guid = buf[56..72].try_into().unwrap();

        let p_entry_lba = read_le_bytes!(buf, u64, 72..80);

        let num_parts = read_le_bytes!(buf, u32, 80..84);
        let size_of_p_entries = read_le_bytes!(buf, u32, 84..88);
        let p_crc32 = read_le_bytes!(buf, u32, 88..92);

        Ok(Self {
            rev,
            size,
            //crc32,
            my_lba,
            other_lba,

            first_lba,
            last_lba,

            guid,

            p_entry_lba,

            num_parts,
            size_of_p_entries,
            p_crc32,
        })
    }
}
