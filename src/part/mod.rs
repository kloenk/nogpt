use crate::DefaultGPTTypeGuid::Unknown;
use crate::{read_le_bytes, GPTError, Result, GUID};
use core::cmp;
use core::convert::Infallible;

mod attrs;
pub use attrs::Attributes;

//pub const ESP_GUID_TYPE: GUID = GUID::new()

pub struct GPTPartHeader<T = DefaultGPTTypeGuid, A = Attributes>
where
    T: GPTTypeGuid,
{
    /// Unique ID that defines the purpose and type of this Partition.
    /// A value of zero defines that this partition entry is not being used.
    pub type_guid: T,
    /// GUID that is unique for every partition entry.
    /// Every partition ever created will have a unique GUID.
    /// This GUID must be assigned when the GPT Partition Entry is created.
    pub guid: GUID,

    /// Starting LBA of the partition defined by this entry.
    pub start_lba: u64,
    ///Ending LBA of the partition defined by this entry.
    pub end_lba: u64,

    /// Attribute bits, all bits reserved by `UEFI`
    pub attrs: A,

    /// Null-terminated string containing a human-readable name of the partition.
    pub name: [u16; 36],

    /// String representation of name.
    #[cfg(feature = "alloc")]
    pub name_str: alloc::string::String,
    // reserved
}

impl<T, A> GPTPartHeader<T, A>
where
    T: GPTTypeGuid,
    GPTError: From<<T as TryFrom<[u8; 16]>>::Error>,
    A: TryFrom<u64>,
    GPTError: From<<A as TryFrom<u64>>::Error>,
{
    /// Parse gpt partition header.
    pub fn parse(buf: &[u8]) -> Result<Self> {
        let type_guid: [u8; 16] = read_le_bytes!(buf, 0..16);
        let type_guid: T = type_guid.try_into()?;

        let guid = read_le_bytes!(buf, 16..32);

        let start_lba = read_le_bytes!(buf, u64, 32..40);
        let end_lba = read_le_bytes!(buf, u64, 40..48);

        let attrs = read_le_bytes!(buf, u64, 48..56);
        let attrs = attrs.try_into()?;

        let name_in: [u8; 72] = read_le_bytes!(buf, 56..128);
        // TODO: some faster way?
        let mut name = [0; 36];
        for x in 0..36 {
            name[x] = u16::from_le_bytes([name_in[x * 2], name_in[x * 2 + 1]]);
        }

        #[cfg(feature = "alloc")]
        let name_str = {
            let len = (0..36).take_while(|&i| name[i] != 0).count();
            alloc::string::String::from_utf16_lossy(&name[..len])
        };

        Ok(Self {
            type_guid,
            guid,

            start_lba,
            end_lba,

            attrs,

            name,

            #[cfg(feature = "alloc")]
            name_str,
        })
    }
}

impl<T, A> core::fmt::Debug for GPTPartHeader<T, A>
where
    T: GPTTypeGuid + core::fmt::Debug,
    A: core::fmt::Debug,
{
    #[cfg(feature = "alloc")]
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        fmt.debug_struct("GptPartHeader")
            .field("type_guid", &self.type_guid)
            .field("guid", &self.guid)
            .field("start_lba", &self.start_lba)
            .field("end_lba", &self.end_lba)
            .field("attrs", &self.attrs)
            .field("name", &self.name_str)
            .finish()
    }

    #[cfg(not(feature = "alloc"))]
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let len = (0..36).take_while(|&i| self.name[i] != 0).count();

        fmt.debug_struct("GptPartHeader")
            .field("type_guid", &self.type_guid)
            .field("guid", &self.guid)
            .field("start_lba", &self.start_lba)
            .field("end_lba", &self.end_lba)
            .field("attrs", &self.attrs)
            .field("name", &&self.name[..len])
            .finish()
    }
}

pub trait GPTTypeGuid: TryFrom<[u8; 16]> + TryInto<[u8; 16]> {
    // TODO: add provided function to convert to guid values pretty printed string
}

// TODO: somehow pack to same size as GUID

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum DefaultGPTTypeGuid {
    /// Unused Entry.
    Unused,
    /// EFI System Partition.
    ESP,
    /// Partition containing a legacy MBR
    LegacyMBR,
    Unknown(GUID),
}

impl From<[u8; 16]> for DefaultGPTTypeGuid {
    fn from(value: [u8; 16]) -> Self {
        let value = GUID::from(value);
        value.into()
    }
}

impl Into<[u8; 16]> for DefaultGPTTypeGuid {
    fn into(self) -> [u8; 16] {
        let guid: GUID = self.into();
        guid.into()
    }
}

impl Into<GUID> for DefaultGPTTypeGuid {
    fn into(self) -> GUID {
        match self {
            DefaultGPTTypeGuid::Unused => GUID::UNUSED,
            DefaultGPTTypeGuid::ESP => GUID::ESP,
            DefaultGPTTypeGuid::LegacyMBR => GUID::LEGACY_MBR,
            DefaultGPTTypeGuid::Unknown(v) => v,
        }
    }
}

impl From<GUID> for DefaultGPTTypeGuid {
    fn from(value: GUID) -> Self {
        match value {
            GUID::UNUSED => DefaultGPTTypeGuid::Unused,
            GUID::ESP => DefaultGPTTypeGuid::ESP,
            GUID::LEGACY_MBR => DefaultGPTTypeGuid::LegacyMBR,
            v => DefaultGPTTypeGuid::Unknown(v),
        }
    }
}

impl GPTTypeGuid for DefaultGPTTypeGuid {}
impl GPTTypeGuid for GUID {}

#[cfg(test)]
mod test {
    use crate::{DefaultGPTTypeGuid, GUID};

    #[test]
    fn eq_guid_default_partition_type() {
        let lhs = DefaultGPTTypeGuid::ESP;
        let rhs: DefaultGPTTypeGuid = GUID::ESP.into();

        assert_eq!(lhs, rhs);
    }

    #[test]
    fn guid_default_partition_type_into_guid() {
        let lhs: GUID = DefaultGPTTypeGuid::Unused.into();
        assert_eq!(lhs, GUID::UNUSED);

        let lhs: GUID = DefaultGPTTypeGuid::ESP.into();
        assert_eq!(lhs, GUID::ESP);

        let lhs: GUID = DefaultGPTTypeGuid::LegacyMBR.into();
        assert_eq!(lhs, GUID::LEGACY_MBR);

        let guid: GUID = "6FCC8240-3985-4840-901F-A05E7FD9B69D".parse().unwrap();
        let lhs: GUID = DefaultGPTTypeGuid::Unknown(guid).into();
        assert_eq!(lhs, guid);
    }

    #[test]
    fn guid_default_partition_type_from_guid() {
        let lhs: DefaultGPTTypeGuid = GUID::UNUSED.into();
        assert_eq!(lhs, DefaultGPTTypeGuid::Unused);

        let lhs: DefaultGPTTypeGuid = GUID::ESP.into();
        assert_eq!(lhs, DefaultGPTTypeGuid::ESP);

        let lhs: DefaultGPTTypeGuid = GUID::LEGACY_MBR.into();
        assert_eq!(lhs, DefaultGPTTypeGuid::LegacyMBR);

        let guid: GUID = "6FCC8240-3985-4840-901F-A05E7FD9B69D".parse().unwrap();
        let lhs: DefaultGPTTypeGuid = guid.into();
        assert_eq!(lhs, DefaultGPTTypeGuid::Unknown(guid));
    }
}
