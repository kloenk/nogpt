#[cfg(any(feature = "bitflags"))]
use crate::GPTError;

#[cfg(any(feature = "bitflags"))]
bitflags::bitflags! {
    #[repr(transparent)]
    pub struct Attributes: u64 {
        ///If this bit is set, the partition is required for the platform to function.
        /// The owner/creator of the partition indicates that deletion or modification
        /// of the contents can result in loss of platform features or failure for the
        /// platform to boot or operate. The system cannot function normally if this partition
        /// is removed, and it should be considered part of the hardware of the system.
        /// Actions such as running diagnostics, system recovery, or even OS install or
        /// boot could potentially stop working if this partition is removed. Unless OS
        /// software or firmware recognizes this partition, it should never be removed
        /// or modified as the UEFI firmware or platform hardware may become non-functional.
        const REQUIRED = 0b1;
        /// If this bit is set, then firmware must not produce an `EFI_BLOCK_IO_PROTOCOL`
        /// device for this partition.
        const NO_BLOCK_IO = 0b10;
        /// This bit is set aside by this specification to let systems with traditional PC-AT BIOS
        /// firmware implementations inform certain limited, special-purpose software running on
        /// these systems that a GPT partition may be bootable. For systems with firmware
        /// implementations conforming to this specification, the UEFI boot manager must ignore
        /// this bit when selecting a UEFI-compliant application, e.g., an OS loader.
        /// Therefore there is no need for this specification to define the exact meaning of
        /// this bit.
        const LEGACY_BIOS_BOOTABLE = 0b100;
    }
}

#[cfg(not(feature = "bitflags"))]
#[derive(Debug)]
#[repr(transparent)]
pub struct Attributes(pub u64);

#[cfg(not(feature = "bitflags"))]
impl Attributes {
    fn bits(&self) -> u64 {
        self.0
    }
}

#[cfg(feature = "bitflags")]
impl TryFrom<u64> for Attributes {
    type Error = GPTError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        Attributes::from_bits(value).ok_or(GPTError::InvalidData)
    }
}

#[cfg(not(feature = "bitflags"))]
impl From<u64> for Attributes {
    fn from(value: u64) -> Self {
        Attributes(value)
    }
}

impl From<Attributes> for u64 {
    fn from(attr: Attributes) -> u64 {
        attr.bits()
    }
}
