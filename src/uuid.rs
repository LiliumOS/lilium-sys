use core::str::FromStr;

#[repr(C)]
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
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

impl Uuid {
    pub const NIL: Uuid = Uuid { minor: 0, major: 0 };
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
