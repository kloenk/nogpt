use block_device::BlockDevice;
use crc::{crc32, Hasher32};

use crate::{read_le_bytes, GptError, Result};

const EFI_SIGNATURE: u64 = 0x5452415020494645;

#[derive(Debug)]
pub struct GptHeader {
    pub rev: u32,
    pub size: u32,
    pub crc32: u32,
    pub my_lba: u64,
    pub other_lba: u64,

    pub first_lba: u64,
    pub last_lba: u64,

    pub guid: [u8; 16],

    pub p_entry_lba: u64,

    pub num_parts: u32,
    pub size_of_p_entry: u32,
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
        let size_of_p_entry = read_le_bytes!(buf, u32, 84..88);
        let p_crc32 = read_le_bytes!(buf, u32, 88..92);

        Ok(Self {
            rev,
            size,
            crc32,
            my_lba,
            other_lba,

            first_lba,
            last_lba,

            guid,

            p_entry_lba,

            num_parts,
            size_of_p_entry,
            p_crc32,
        })
    }

    pub fn validate(&self, my_lba: u64, part_table: &[u8]) -> Result<()> {
        if self.my_lba != my_lba {
            return Err(GptError::InvalidLba(self.my_lba));
        }
        self.validate_crc()?;
        self.validate_part_crc(part_table)?;

        Ok(())
    }

    pub fn validate_part_crc(&self, part_table: &[u8]) -> Result<()> {
        let len = (self.num_parts * self.size_of_p_entry) as usize;
        if len > part_table.len() {
            return Err(GptError::PartitionTableToShort(len as u32));
        }

        let mut digest = crc32::Digest::new(crc32::IEEE);
        digest.write(&part_table[0..len]);

        let digest = digest.sum32();
        if digest != self.p_crc32 {
            return Err(GptError::InvalidCrcParts(digest, self.p_crc32));
        }

        Ok(())
    }

    pub fn validate_crc(&self) -> Result<()> {
        let crc_cal = self.calculate_crc();
        if crc_cal != self.crc32 {
            return Err(GptError::InvalidCrcHeader(crc_cal, self.crc32));
        }

        Ok(())
    }

    pub fn calculate_crc(&self) -> u32 {
        let mut digest = crc32::Digest::new(crc32::IEEE);

        digest.write(&EFI_SIGNATURE.to_le_bytes());
        digest.write(&self.rev.to_le_bytes());
        digest.write(&self.size.to_le_bytes());
        digest.write(&[0, 0, 0, 0, 0, 0, 0, 0]);
        digest.write(&self.my_lba.to_le_bytes());
        digest.write(&self.other_lba.to_le_bytes());
        digest.write(&self.first_lba.to_le_bytes());
        digest.write(&self.last_lba.to_le_bytes());
        digest.write(&self.guid);
        digest.write(&self.p_entry_lba.to_le_bytes());
        digest.write(&self.num_parts.to_le_bytes());
        digest.write(&self.size_of_p_entry.to_le_bytes());
        digest.write(&self.p_crc32.to_le_bytes());

        digest.sum32()
    }
}

#[derive(Debug, Copy, Clone)]
pub enum GptHeaderType {
    Main,
    Backup,
}

impl core::fmt::Display for GptHeaderType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            GptHeaderType::Main => write!(f, "main"),
            GptHeaderType::Backup => write!(f, "backup"),
        }
    }
}
