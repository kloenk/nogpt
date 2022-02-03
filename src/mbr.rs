use crate::{GPTError, Result};

#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct MBRPartitionRecord {
    pub boot_indicator: u8,
    pub start_head: u8,
    pub start_sector: u8,
    pub start_track: u8,
    pub os_indicator: u8,
    pub end_head: u8,
    pub end_sector: u8,
    pub end_track: u8,
    pub starting_lba: [u8; 4],
    pub size_in_lba: [u8; 4],
}

impl MBRPartitionRecord {
    /// Check if the partition is to be considered empty.
    ///
    /// A partition is considered empty if the [`Self::os_indicator`] or the [`Self::size_in_lba`]
    /// is 0.
    pub fn is_empty(&self) -> bool {
        self.os_indicator == 0 || self.size_in_lba() == 0
    }

    /// Return the starting logical block as u32.
    pub fn starting_lba(&self) -> u32 {
        u32::from_le_bytes(self.starting_lba)
    }

    /// Return the size in logical blocks as u32.
    pub fn size_in_lba(&self) -> u32 {
        u32::from_le_bytes(self.size_in_lba)
    }

    /// Helper to calculate [`Self::starting_lba`] + [`Self::size_in_lba`] to get the ending lba.
    pub fn ending_lba(&self) -> u32 {
        self.starting_lba() + self.size_in_lba()
    }

    /// Defines a UEFI system partition.
    pub const UEFI_SYSTEM_OS_TYPE: u8 = 0xef;
    /// Is used by a protective MBR to define a fake partition covering the entire disk.
    pub const GPT_PROTECTIVE_OS_TYPE: u8 = 0xee;
}

#[derive(Clone, Debug)]
#[repr(C, packed)]
pub struct MasterBootRecord {
    /// x86 code used on a non-UEFI system to select an MBR partition record and load the first
    /// logical block of that partition . This code shall not be executed on UEFI systems.
    pub bootstrapcode: [u8; 440],
    /// Unique Disk Signature This may be used by the OS to identify the disk from other disks in
    /// the system. This value is always written by the OS and is never written by EFI firmware.
    pub unique_mbr_signature: [u8; 4],
    /// Unknown. This field shall not be used by UEFI firmware.
    pub unknown: [u8; 2],
    /// Array of four legacy MBR partition records [`MBRPartitionRecord`].
    pub partition: [MBRPartitionRecord; 4],
    pub signature: [u8; 2],
}

impl MasterBootRecord {
    /// # Safety
    ///
    /// Behavior is undefined if any of the following conditions are violated:
    ///
    /// * `buf` must be valid for 512 reads.
    ///
    /// * `buf` must be properly aligned.
    ///
    pub unsafe fn from_buf(buf: &[u8]) -> Result<Self> {
        if buf.len() < core::mem::size_of::<Self>() {
            return Err(GPTError::UnexpectedEOF);
        }

        let mut ret: MasterBootRecord = unsafe { core::ptr::read(buf.as_ptr() as _) };
        Ok(ret)
    }

    /// Return the signature as u16
    pub fn signature(&self) -> u16 {
        u16::from_le_bytes(self.signature)
    }

    pub fn verify(&self, last_lba: Option<u32>) -> Result<()> {
        if self.signature() != 0xaa55 {
            return Err(GPTError::InvalidData);
        }

        self.verify_partitions(last_lba)?;

        Ok(())
    }

    pub fn verify_partitions(&self, last_lba: Option<u32>) -> Result<()> {
        // FIXME: 1..3 does not have to be empty, just to lazy right now
        if !self.partition[1].is_empty()
            || !self.partition[2].is_empty()
            || !self.partition[3].is_empty()
        {
            return Err(GPTError::InvalidMbr);
        }
        /*(0..=2)
        .fold([0u8, 1, 2, 3], |order, left| {
            (left + 1..=3).fold(order, |mut order, right| {
                let swap = |mut this: [u8; 4], left, right| {
                    this.swap(left, right);
                    this
                };
                if self.partition[left].starting_lba() > self.partition[right].starting_lba() {
                    swap(order, left, right)
                } else {
                    order
                }
            })
        })
        .iter()
        .take(3)
        .try_for_each(|index| {
            let index = *index as usize;
            let left = self.partition[index];
            let right = self.partition[index + 1];
            if left.is_empty() || right.is_empty() || left.ending_lba() < right.starting_lba() {
                Ok(())
            } else {
                Err(GptError::OverlappingPartitions)
            }
        });*/

        let last_lba_p = self.partition[0].ending_lba();
        if let Some(last_lba) = last_lba {
            if last_lba_p > last_lba {
                return Err(GPTError::InvalidMbr);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::mbr::{MBRPartitionRecord, MasterBootRecord};

    #[test]
    fn size() {
        assert_eq!(core::mem::size_of::<MBRPartitionRecord>(), 16);
        assert_eq!(core::mem::size_of::<MasterBootRecord>(), 512)
    }
}
