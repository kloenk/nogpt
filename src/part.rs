use crate::DefaultGptTypeGuid::Unknown;
use crate::{read_le_bytes, GptError, Result, GUID};
use core::cmp;
use std::convert::Infallible;

//pub const ESP_GUID_TYPE: GUID = GUID::new()

pub struct GptPartHeader<T = DefaultGptTypeGuid>
where
    T: GptTypeGuid,
    GptError: From<<T as TryFrom<[u8; 16]>>::Error>,
    GptError: From<<T as TryInto<[u8; 16]>>::Error>,
{
    pub type_guid: T,
    pub guid: GUID,

    pub start_lba: u64,
    pub end_lba: u64,

    // TODO: attrs type
    pub attrs: u64,

    pub name: [u16; 36],
    // reserved
}

impl<T> GptPartHeader<T>
where
    T: GptTypeGuid,
    GptError: From<<T as TryFrom<[u8; 16]>>::Error>,
    GptError: From<<T as TryInto<[u8; 16]>>::Error>,
{
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

        Ok(Self {
            type_guid,
            guid,

            start_lba,
            end_lba,

            attrs,

            name,
        })
    }
}

impl<T> core::fmt::Debug for GptPartHeader<T>
where
    T: GptTypeGuid + core::fmt::Debug,
    GptError: From<<T as TryFrom<[u8; 16]>>::Error>,
    GptError: From<<T as TryInto<[u8; 16]>>::Error>,
{
    #[cfg(feature = "alloc")]
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let len = (0..36).take_while(|&i| self.name[i] != 0).count();
        let name = alloc::string::String::from_utf16_lossy(&self.name[..len]);

        fmt.debug_struct("GptPartHeader")
            .field("type_guid", &self.type_guid)
            .field("guid", &self.guid)
            .field("start_lba", &self.start_lba)
            .field("end_lba", &self.end_lba)
            .field("attrs", &self.attrs)
            .field("name", &name)
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
    Unused,
    ESP,
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
