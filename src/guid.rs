use crate::GPTError;
use core::fmt::Debug;

#[repr(C, packed)]
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

    /// Convert guid to byte representation
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

    /// Unused Entry.
    pub const UNUSED: Self = Self::new(0, 0, 0, 0);
    /// EFI System Partition.
    pub const ESP: Self = Self::new(0xC12A7328, 0xF81F, 0x11D2, 0xBA4B00A0C93EC93B);
    /// Partition containing a legacy MBR
    pub const LEGACY_MBR: Self = Self::new(0x024DEE41, 0x33E7, 0x11D3, 0x9D690008C781F39F);
}

impl TryFrom<&[u8]> for GUID {
    type Error = GPTError;

    fn try_from(value: &[u8]) -> core::result::Result<Self, Self::Error> {
        let v: [u8; 16] = value
            .get(0..16)
            .ok_or(GPTError::NoGPT)?
            .try_into()
            .map_err(|_| GPTError::NoGPT)?;

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
    use crate::guid::GUID;

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

    #[test]
    fn test_eq() {
        let lhs = GUID::ESP;
        let rhs = "C12A7328-F81F-11D2-BA4B-00A0C93EC93B"
            .parse::<GUID>()
            .unwrap();

        assert_eq!(lhs, rhs);
    }
}
