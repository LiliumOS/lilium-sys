use core::str::FromStr;

#[repr(C, align(16))]
#[derive(Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "bytemuck", bytemuck::Zeroable, bytemuck::Pod)]
pub struct Uuid {
    pub minor: u64,
    pub major: u64,
}

impl FromStr for Uuid {
    type Err = TryParseUuidError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        try_parse_uuid(s)
    }
}

impl core::fmt::Debug for Uuid {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if f.alternate() {
            f.write_str("{")?;
        }
        let minor_lo48 = self.minor & ((1 << 48) - 1);
        let minor_hi16 = self.minor >> 48;
        let major_lo16 = self.major & ((1 << 16) - 1);
        let major_mid16 = (self.major >> 16) & ((1 << 16) - 1);
        let major_hi32 = self.major >> 32;

        f.write_fmt(format_args!(
            "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
            major_hi32, major_mid16, major_lo16, minor_hi16, minor_lo48
        ))?;

        if f.alternate() {
            f.write_str("}")?;
        }
        Ok(())
    }
}

impl core::fmt::Display for Uuid {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if f.alternate() {
            f.write_str("{")?;
        }
        let minor_lo48 = self.minor & ((1 << 48) - 1);
        let minor_hi16 = self.minor >> 48;
        let major_lo16 = self.major & ((1 << 16) - 1);
        let major_mid16 = (self.major >> 16) & ((1 << 16) - 1);
        let major_hi32 = self.major >> 32;

        f.write_fmt(format_args!(
            "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
            major_hi32, major_mid16, major_lo16, minor_hi16, minor_lo48
        ))?;

        if f.alternate() {
            f.write_str("}")?;
        }
        Ok(())
    }
}

impl core::fmt::LowerHex for Uuid {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if f.alternate() {
            f.write_str("{")?;
        }
        let minor_lo48 = self.minor & ((1 << 48) - 1);
        let minor_hi16 = self.minor >> 48;
        let major_lo16 = self.major & ((1 << 16) - 1);
        let major_mid16 = (self.major >> 16) & ((1 << 16) - 1);
        let major_hi32 = self.major >> 32;

        f.write_fmt(format_args!(
            "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
            major_hi32, major_mid16, major_lo16, minor_hi16, minor_lo48
        ))?;

        if f.alternate() {
            f.write_str("}")?;
        }
        Ok(())
    }
}

impl core::fmt::UpperHex for Uuid {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if f.alternate() {
            f.write_str("{")?;
        }
        let minor_lo48 = self.minor & ((1 << 48) - 1);
        let minor_hi16 = self.minor >> 48;
        let major_lo16 = self.major & ((1 << 16) - 1);
        let major_mid16 = (self.major >> 16) & ((1 << 16) - 1);
        let major_hi32 = self.major >> 32;

        f.write_fmt(format_args!(
            "{:08X}-{:04X}-{:04X}-{:04X}-{:012X}",
            major_hi32, major_mid16, major_lo16, minor_hi16, minor_lo48
        ))?;

        if f.alternate() {
            f.write_str("}")?;
        }
        Ok(())
    }
}

impl Uuid {
    pub const NIL: Uuid = Uuid { minor: 0, major: 0 };
    pub const FULL: Uuid = Uuid {
        minor: !0,
        major: !0,
    };
}

const fn to_hexdig(c: u8) -> Option<u64> {
    if b'0' <= c && c <= b'9' {
        return Some((c - b'0') as u64);
    } else if b'A' <= c && c <= b'F' {
        return Some((c - b'A') as u64 + 10);
    } else if b'a' <= c && c <= b'f' {
        return Some((c - b'a') as u64 + 10);
    } else {
        return None;
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum TryParseUuidError {
    InvalidLen(usize),
    InvalidDigit(usize),
    UnexpectedChar(usize),
}

pub const fn try_parse_uuid(st: &str) -> Result<Uuid, TryParseUuidError> {
    use TryParseUuidError::*;
    let st = match st.as_bytes() {
        [b'{', rest @ .., b'}'] => rest,
        x => x,
    };
    if st.len() != 36 {
        return Err(InvalidLen(st.len()));
    }

    let mut i = 0;

    let mut major = 0u64;
    let mut minor = 0u64;

    while i < 8 {
        let c = st[i];

        if let Some(dig) = to_hexdig(c) {
            major <<= 4;
            major |= (dig as u64);
        } else {
            return Err(InvalidDigit(i));
        }
        i += 1;
    }

    if st[i] != b'-' {
        return Err(UnexpectedChar(i));
    }

    i += 1;

    while i < 13 {
        let c = st[i];

        if let Some(dig) = to_hexdig(c) {
            major <<= 4;
            major |= (dig as u64);
        } else {
            return Err(InvalidDigit(i));
        }
        i += 1;
    }

    if st[i] != b'-' {
        return Err(UnexpectedChar(i));
    }

    i += 1;

    while i < 18 {
        let c = st[i];

        if let Some(dig) = to_hexdig(c) {
            major <<= 4;
            major |= (dig as u64);
        } else {
            return Err(InvalidDigit(i));
        }
        i += 1;
    }

    if st[i] != b'-' {
        return Err(UnexpectedChar(i));
    }

    i += 1;

    while i < 23 {
        let c = st[i];

        if let Some(dig) = to_hexdig(c) {
            minor <<= 4;
            minor |= (dig as u64);
        } else {
            return Err(InvalidDigit(i));
        }
        i += 1;
    }

    if st[i] != b'-' {
        return Err(UnexpectedChar(i));
    }

    i += 1;

    while i < 36 {
        let c = st[i];

        if let Some(dig) = to_hexdig(c) {
            minor <<= 4;
            minor |= (dig as u64);
        } else {
            return Err(InvalidDigit(i));
        }
        i += 1;
    }

    Ok(Uuid { major, minor })
}

pub const fn parse_uuid(st: &str) -> Uuid {
    match try_parse_uuid(st) {
        Ok(uuid) => uuid,
        Err(TryParseUuidError::InvalidLen(_)) => panic!("Invalid UUID length"),
        Err(TryParseUuidError::InvalidDigit(_)) => {
            panic!("Encountered an invalid character where a hex digit is expected")
        }
        Err(TryParseUuidError::UnexpectedChar(_)) => {
            panic!("Encountered an invalid character in the UUID")
        }
    }
}
