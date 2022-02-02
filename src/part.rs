use crate::DefaultGptTypeGuid::Unknown;
use crate::{read_le_bytes, GptError, Result, GUID};
use core::cmp;
use core::convert::Infallible;

//pub const ESP_GUID_TYPE: GUID = GUID::new()

pub struct GptPartHeader<T = DefaultGptTypeGuid>
where
    T: GptTypeGuid,
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

    // TODO: attrs type
    /// Attribute bits, all bits reserved by `UEFI`
    pub attrs: u64,

    /// Null-terminated string containing a human-readable name of the partition.
    pub name: [u16; 36],

    /// String representation of name.
    #[cfg(feature = "alloc")]
    pub name_str: alloc::string::String,
    // reserved
}

impl<T> GptPartHeader<T>
where
    T: GptTypeGuid,
    GptError: From<<T as TryFrom<[u8; 16]>>::Error>,
{
    /// Parse gpt partition header.
    pub fn parse(buf: &[u8]) -> Result<Self> {
        let type_guid: [u8; 16] = read_le_bytes!(buf, 0..16);
        let type_guid: T = type_guid.try_into()?;

        let guid = read_le_bytes!(buf, 16..32);

        let start_lba = read_le_bytes!(buf, u64, 32..40);
        let end_lba = read_le_bytes!(buf, u64, 40..48);

        let attrs = read_le_bytes!(buf, u64, 48..56);

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

impl<T> core::fmt::Debug for GptPartHeader<T>
where
    T: GptTypeGuid + core::fmt::Debug,
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

pub trait GptTypeGuid: TryFrom<[u8; 16]> + TryInto<[u8; 16]> {
    // TODO: add provided function to convert to guid values pretty printed string
}

// TODO: somehow pack to same size as GUID

#[derive(Debug)]
pub enum DefaultGptTypeGuid {
    /// Unused Entry.
    Unused,
    /// EFI System Partition.
    ESP,
    /// Partition containing a legacy MBR
    LegacyMBR,
    Unknown(GUID),
}

impl TryFrom<[u8; 16]> for DefaultGptTypeGuid {
    type Error = Infallible;

    fn try_from(value: [u8; 16]) -> core::result::Result<Self, Self::Error> {
        let value = GUID::from(value);
        Ok(match value {
            GUID::UNUSED => DefaultGptTypeGuid::Unused,
            GUID::ESP => DefaultGptTypeGuid::ESP,
            GUID::LEGACY_MBR => DefaultGptTypeGuid::LegacyMBR,
            v => DefaultGptTypeGuid::Unknown(v),
        })
    }
}

impl TryInto<[u8; 16]> for DefaultGptTypeGuid {
    type Error = GptError;

    fn try_into(self) -> core::result::Result<[u8; 16], Self::Error> {
        Ok(match self {
            DefaultGptTypeGuid::Unused => GUID::ESP.into(),
            DefaultGptTypeGuid::ESP => GUID::ESP.into(),
            DefaultGptTypeGuid::LegacyMBR => GUID::LEGACY_MBR.into(),
            DefaultGptTypeGuid::Unknown(v) => v.into(),
        })
    }
}

impl GptTypeGuid for DefaultGptTypeGuid {}
impl GptTypeGuid for GUID {}
