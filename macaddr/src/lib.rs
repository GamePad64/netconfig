use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;
use thiserror::Error as ThisError;

#[repr(transparent)]
#[derive(Default, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct MacAddr6(pub [u8; 6]);

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("mac address is invalid")]
    InvalidMac,
}

impl MacAddr6 {
    pub fn random() -> Self {
        Self(rand::random())
    }

    pub fn set_local(&mut self, v: bool) -> MacAddr6 {
        if v {
            self.0[0] |= 0b0000_0010;
        } else {
            self.0[0] &= !0b0000_0010;
        }
        *self
    }

    pub const fn is_local(&self) -> bool {
        (self.0[0] & 0b0000_0010) != 0
    }

    pub fn set_multicast(&mut self, v: bool) -> MacAddr6 {
        if v {
            self.0[0] |= 0b0000_0001;
        } else {
            self.0[0] &= !0b0000_0001;
        }
        *self
    }

    pub const fn is_multicast(&self) -> bool {
        (self.0[0] & 0b0000_0001) != 0
    }

    fn write_delimited(&self, sep: &str) -> String {
        format!(
            "{:02X}{sep}{:02X}{sep}{:02X}{sep}{:02X}{sep}{:02X}{sep}{:02X}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5]
        )
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    pub fn as_c_slice(&self) -> &[std::os::raw::c_char] {
        unsafe { &*(self.as_slice() as *const _ as *const [std::os::raw::c_char]) }
    }
}

impl Display for MacAddr6 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.write_delimited(":"))
    }
}

impl Debug for MacAddr6 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.write_delimited(":"))
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

impl TryFrom<&[std::os::raw::c_char]> for MacAddr6 {
    type Error = Error;

    fn try_from(value: &[std::os::raw::c_char]) -> Result<Self, Self::Error> {
        Self::try_from(unsafe { &*(value as *const _ as *const [u8]) })
    }
}

impl FromStr for MacAddr6 {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref MAC_SEMI_RE: Regex = Regex::new(r#"^([[:xdigit:]]{2}):([[:xdigit:]]{2}):([[:xdigit:]]{2}):([[:xdigit:]]{2}):([[:xdigit:]]{2}):([[:xdigit:]]{2})$"#).unwrap();
            static ref MAC_DASH_RE: Regex = Regex::new(r#"^([[:xdigit:]]{2})-([[:xdigit:]]{2})-([[:xdigit:]]{2})-([[:xdigit:]]{2})-([[:xdigit:]]{2})-([[:xdigit:]]{2})$"#).unwrap();
        }

        let mut mac_hex = s.to_string();
        mac_hex = MAC_SEMI_RE.replace(&mac_hex, "$1$2$3$4$5$6").into();
        mac_hex = MAC_DASH_RE.replace(&mac_hex, "$1$2$3$4$5$6").into();

        if mac_hex.len() != 12 {
            return Err(Error::InvalidMac);
        }

        Ok(Self(
            hex::decode(mac_hex)
                .map_err(|_| Error::InvalidMac)?
                .try_into()
                .map_err(|_| Error::InvalidMac)?,
        ))
    }
}

impl Serialize for MacAddr6 {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.to_string().serialize(s)
    }
}

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
    use crate::MacAddr6;
    use serde::{Deserialize, Serialize};

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
    }

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
