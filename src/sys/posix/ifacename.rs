use delegate::delegate;
use std::ffi::CString;
use std::iter::zip;
use std::str::FromStr;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum InterfaceNameError {
    #[error("interface name is > 16 (null-terminated): {0:?}")]
    NameTooLong(String),
    #[error("NUL byte encountered in name: {0:?}")]
    NulByteEncountered(String),
    #[error("No NUL byte encountered inside InterfaceName: {0:?}")]
    InvalidCString(Vec<libc::c_char>),
    #[error("Invalid Unicode characters inside InterfaceName: {0:?}")]
    InvalidUnicodeString(Vec<libc::c_char>),
}

#[repr(transparent)]
#[derive(Copy, Clone, Debug)]
pub struct InterfaceName([libc::c_char; libc::IFNAMSIZ as _]);

impl Default for InterfaceName {
    fn default() -> Self {
        Self(unsafe { std::mem::zeroed() })
    }
}

impl FromStr for InterfaceName {
    type Err = InterfaceNameError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s)
    }
}

impl TryFrom<&str> for InterfaceName {
    type Error = InterfaceNameError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.len() >= libc::IFNAMSIZ {
            return Err(InterfaceNameError::NameTooLong(value.to_string()));
        }
        let cname = CString::new(value)
            .map_err(|_| InterfaceNameError::NulByteEncountered(value.to_string()))?;

        let mut result = Self::default();
        for (x, y) in zip(result.0.iter_mut(), cname.as_bytes_with_nul().iter()) {
            *x = *y as libc::c_char;
        }
        Ok(result)
    }
}

impl TryFrom<&InterfaceName> for String {
    type Error = InterfaceNameError;

    fn try_from(value: &InterfaceName) -> Result<Self, Self::Error> {
        if !value.is_valid() {
            return Err(InterfaceNameError::InvalidCString(value.0.to_vec()));
        }
        Ok(unsafe { std::ffi::CStr::from_ptr(value.as_ptr()) }
            .to_str()
            .map_err(|_| InterfaceNameError::InvalidUnicodeString(value.0.to_vec()))?
            .to_string())
    }
}

impl InterfaceName {
    pub fn is_valid(&self) -> bool {
        self.0[libc::IFNAMSIZ - 1] == 0
    }

    delegate! {
        to self.0 {
            pub fn as_slice(&self) -> &[libc::c_char];
            pub fn as_mut_slice(&mut self) -> &mut [libc::c_char];
            pub fn as_ptr(&self) -> *const libc::c_char;
            pub fn as_mut_ptr(&mut self) -> *mut libc::c_char;
        }
    }
}
