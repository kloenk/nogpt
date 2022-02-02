use block_device::BlockDevice;
use crc::{crc32, Hasher32};

use crate::{read_le_bytes, GptError, Result, GUID};

const EFI_SIGNATURE: u64 = 0x5452415020494645;
const GPT_REV: u32 = 0x00010000;

#[derive(Debug)]
pub struct GptHeader {
    /// Size in bytes of the GPT Header. The [`Self::size`] must be greater than or equal to
    /// 92 and must be less than or equal to the logical block size.
    pub size: u32,
    /// CRC32 checksum for the GPT Header structure. This value is computed by
    /// setting this field to 0, and computing the 32-bit CRC for [`Self::size`] bytes.
    pub crc32: u32,
    /// The LBA that contains this data structure.
    pub my_lba: u64,
    /// LBA address of the alternate GPT Header.
    pub other_lba: u64,

    /// The first usable logical block that may be used by a partition described by a GUID
    /// Partition Entry.
    pub first_lba: u64,
    /// The last usable logical block that may be used by a partition described by a GUID
    /// Partition Entry.
    pub last_lba: u64,

    /// GUID that can be used to uniquely identify the disk.
    pub guid: GUID,

    /// The starting LBA of the GUID Partition Entry array.
    pub p_entry_lba: u64,

    /// The number of Partition Entries in the GUID Partition Entry array.
    pub num_parts: u32,
    /// The size, in bytes, of each the GUID Partition Entry structures in the GUID Partition Entry
    /// array. This field shall be set to a value of 128 x 2n where n is an integer greater than or
    /// equal to zero (e.g., 128, 256, 512, etc.).
    /// NOTE: Previous versions of this specification allowed any multiple of 8..
    pub size_of_p_entry: u32,

    /// The CRC32 of the GUID Partition Entry array. Starts at [`Self::p_entry_lba`] and is
    /// computed over a byte length of [`Self::num_parts`] * [`Self::size_of_p_entry`].
    pub p_crc32: u32,
}

impl GptHeader {
    /// Read the GPT header from buf and serializes it into this struct.
    /// Checks for `Signature` and `Revision`, but nothing else.
    pub fn parse(buf: &[u8]) -> Result<Self> {
        let sig = read_le_bytes!(buf, u64, 0..8);
        if sig != EFI_SIGNATURE {
            return Err(GptError::InvalidSignature(sig));
        }

        let rev = read_le_bytes!(buf, u32, 8..12);
        if rev != GPT_REV {
            return Err(GptError::InvalidSignature(rev as u64));
        }

        let size = read_le_bytes!(buf, u32, 12..16);
        let crc32 = read_le_bytes!(buf, u32, 16..20);

        let my_lba = read_le_bytes!(buf, u64, 24..32);
        let other_lba = read_le_bytes!(buf, u64, 32..40);

        let first_lba = read_le_bytes!(buf, u64, 40..48);
        let last_lba = read_le_bytes!(buf, u64, 48..56);

        // TODO: remove unwrap
        let guid = buf[56..72].try_into()?;

        let p_entry_lba = read_le_bytes!(buf, u64, 72..80);

        let num_parts = read_le_bytes!(buf, u32, 80..84);
        let size_of_p_entry = read_le_bytes!(buf, u32, 84..88);
        let p_crc32 = read_le_bytes!(buf, u32, 88..92);

        Ok(Self {
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

    /// Check this header for valid data. needs the bits of the partition table as input.
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

    /// Verifies own crc sum
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
        digest.write(&GPT_REV.to_le_bytes());
        digest.write(&self.size.to_le_bytes());
        digest.write(&[0, 0, 0, 0, 0, 0, 0, 0]);
        digest.write(&self.my_lba.to_le_bytes());
        digest.write(&self.other_lba.to_le_bytes());
        digest.write(&self.first_lba.to_le_bytes());
        digest.write(&self.last_lba.to_le_bytes());
        digest.write(&self.guid.as_bytes());
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
