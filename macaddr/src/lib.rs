#![cfg_attr(not(feature = "std"), no_std)]
use core::fmt::{Debug, Display, Formatter};
use core::str::FromStr;
use rand::Rng;
#[cfg(feature = "std")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use snafu::Snafu;

#[repr(transparent)]
#[derive(Default, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct MacAddr6([u8; 6]);

#[derive(Eq, PartialEq, Debug, Snafu)]
pub enum Error {
    #[snafu(display("invalid MAC address"))]
    InvalidMac,
}

impl MacAddr6 {
    pub fn random() -> Self {
        let mut result = Self::default();
        rand::rngs::OsRng.fill(result.0.as_mut_slice());
        result
    }

    pub fn set_local(&mut self, v: bool) {
        if v {
            self.0[0] |= 0b0000_0010;
        } else {
            self.0[0] &= !0b0000_0010;
        }
    }

    pub const fn is_local(&self) -> bool {
        (self.0[0] & 0b0000_0010) != 0
    }

    pub fn set_multicast(&mut self, v: bool) {
        if v {
            self.0[0] |= 0b0000_0001;
        } else {
            self.0[0] &= !0b0000_0001;
        }
    }

    pub const fn is_multicast(&self) -> bool {
        (self.0[0] & 0b0000_0001) != 0
    }

    fn write_delimited(&self, f: &mut Formatter<'_>, sep: &str) -> core::fmt::Result {
        write!(
            f,
            "{:02X}{sep}{:02X}{sep}{:02X}{sep}{:02X}{sep}{:02X}{sep}{:02X}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5]
        )
    }

    /// Returns Organizationally unique identifier
    pub fn oui(&self) -> [u8; 3] {
        self.0[..3].try_into().unwrap()
    }

    /// Sets Organizationally unique identifier
    pub fn set_oui(&mut self, oui: [u8; 3]) {
        self.0[..3].copy_from_slice(&oui);
    }

    pub fn as_array(&self) -> [u8; 6] {
        self.0
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    pub fn as_c_slice(&self) -> &[core::ffi::c_char] {
        unsafe { &*(self.as_slice() as *const _ as *const [core::ffi::c_char]) }
    }
}

impl Display for MacAddr6 {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        self.write_delimited(f, ":")
    }
}

impl Debug for MacAddr6 {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        self.write_delimited(f, ":")
    }
}

impl From<[u8; 6]> for MacAddr6 {
    fn from(arr: [u8; 6]) -> Self {
        Self(arr)
    }
}

impl TryFrom<&[u8]> for MacAddr6 {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Ok(Self(value.try_into().map_err(|_| Error::InvalidMac)?))
    }
}

impl TryFrom<&[core::ffi::c_char]> for MacAddr6 {
    type Error = Error;

    fn try_from(value: &[core::ffi::c_char]) -> Result<Self, Self::Error> {
        Self::try_from(unsafe { &*(value as *const _ as *const [u8]) })
    }
}

impl TryFrom<&str> for MacAddr6 {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::from_str(value)
    }
}

#[cfg(feature = "std")]
impl TryFrom<String> for MacAddr6 {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(&value)
    }
}

impl FromStr for MacAddr6 {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut result = Self::default();
        let result_buf = result.0.as_mut();

        if s.len() == 12 {
            hex::decode_to_slice(s, result_buf).map_err(|_| Error::InvalidMac)?;
            Ok(result)
        } else if s.len() == 17 {
            if !s.is_ascii() {
                return Err(Error::InvalidMac);
            }

            let sep = s.chars().nth(2).ok_or(Error::InvalidMac)?;
            if sep != ':' && sep != '-' {
                // Invalid separator
                return Err(Error::InvalidMac);
            }

            if s[2..].chars().step_by(3).any(|x| x != sep) {
                // Inconsistent separator
                return Err(Error::InvalidMac);
            }

            for (i, s) in s.as_bytes().chunks(3).enumerate() {
                result_buf[i] =
                    u8::from_str_radix(unsafe { core::str::from_utf8_unchecked(&s[0..2]) }, 16)
                        .map_err(|_| Error::InvalidMac)?;
            }

            Ok(result)
        } else {
            Err(Error::InvalidMac)
        }
    }
}

#[cfg(feature = "std")]
impl Serialize for MacAddr6 {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.to_string().serialize(s)
    }
}

#[cfg(feature = "std")]
impl<'de> Deserialize<'de> for MacAddr6 {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        MacAddr6::from_str(&String::deserialize(d)?).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod test {
    use crate::{Error, MacAddr6};
    use core::str::FromStr;
    #[cfg(feature = "std")]
    use serde::{Deserialize, Serialize};

    #[cfg(feature = "std")]
    #[test]
    fn test_format() {
        let mac = MacAddr6::from([0x11, 0x22, 0x03, 0x00, 0x50, 0x6A]);
        assert_eq!(mac.to_string(), "11:22:03:00:50:6A")
    }

    #[test]
    fn test_parse() {
        let mac = MacAddr6::from([0x11, 0x22, 0x03, 0x00, 0x50, 0x6A]);
        assert_eq!(mac, "11:22:03:00:50:6A".parse().unwrap());
        assert_eq!(mac, "11-22-03-00-50-6A".parse().unwrap());
        assert_eq!(mac, "11220300506A".parse().unwrap());

        // Inconsistent separators
        assert_eq!(
            MacAddr6::from_str("11-22:03:00:50:6A"),
            Err(Error::InvalidMac)
        );

        // Invalid length
        assert_eq!(
            MacAddr6::from_str("1122:03:00:50:6A"),
            Err(Error::InvalidMac)
        );
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_serde() {
        #[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
        struct S {
            pub mac: MacAddr6,
        }
        let s = S {
            mac: MacAddr6::from([0x11, 0x22, 0x03, 0x00, 0x50, 0x6A]),
        };
        let serialized = serde_json::to_string(&s).unwrap();
        assert_eq!(serialized, r#"{"mac":"11:22:03:00:50:6A"}"#);
        let parsed: S = serde_json::from_str(&serialized).unwrap();
        assert_eq!(parsed, s);
    }

    #[test]
    fn test_flags_roundtrip() {
        let mut addr = MacAddr6::default();
        assert!(!addr.is_local());
        assert!(!addr.is_multicast());

        addr.set_multicast(true);
        assert!(!addr.is_local());
        assert!(addr.is_multicast());

        addr.set_local(true);
        assert!(addr.is_local());
        assert!(addr.is_multicast());

        addr.set_multicast(false);
        assert!(addr.is_local());
        assert!(!addr.is_multicast());

        addr.set_local(false);
        assert!(!addr.is_local());
        assert!(!addr.is_multicast());
    }
}
