#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use std::ffi::CString;
use std::fmt::Debug;
use std::iter::zip;
use std::mem;
use std::str::FromStr;

pub type IfName = [libc::c_char; libc::IFNAMSIZ as _]; // Null-terminated

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct InterfaceName(pub [libc::c_char; libc::IFNAMSIZ as _]);

impl FromStr for InterfaceName {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut s = s.to_string();
        s.truncate(libc::IFNAMSIZ - 1);
        let name = CString::new(s).unwrap();

        type IfName = [libc::c_char; libc::IFNAMSIZ as _];
        let mut result = IfName::default();
        for (x, y) in zip(result.iter_mut(), name.as_bytes_with_nul().iter()) {
            *x = *y as libc::c_char;
        }
        Ok(Self(result))
    }
}

impl ToString for InterfaceName {
    fn to_string(&self) -> String {
        unsafe { std::ffi::CStr::from_ptr(self.0.as_ptr()) }
            .to_string_lossy()
            .to_string()
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ifreq {
    pub ifr_ifrn: InterfaceName,
    pub ifr_ifru: ifreq_ifru,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union ifreq_ifru {
    pub ifru_addr: libc::sockaddr,
    pub ifru_dstaddr: libc::sockaddr,
    pub ifru_broadaddr: libc::sockaddr,
    pub ifru_netmask: libc::sockaddr,
    pub ifru_hwaddr: libc::sockaddr,
    pub ifru_flags: libc::c_short,
    pub ifru_ivalue: libc::c_int,
    pub ifru_mtu: libc::c_int,
    pub ifru_map: ifmap,
    pub ifru_slave: InterfaceName,
    pub ifru_newname: InterfaceName,
    pub ifru_data: *mut libc::c_char,
    align: [u64; 3usize],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ifmap {
    pub mem_start: libc::c_ulong,
    pub mem_end: libc::c_ulong,
    pub base_addr: libc::c_ushort,
    pub irq: libc::c_uchar,
    pub dma: libc::c_uchar,
    pub port: libc::c_uchar,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ifaliasreq4 {
    pub ifra_name: InterfaceName,
    pub ifra_addr: libc::sockaddr_in,
    pub ifra_broadaddr: libc::sockaddr_in,
    pub ifra_mask: libc::sockaddr_in,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ifaliasreq6 {
    pub ifra_name: InterfaceName,
    pub ifra_addr: libc::sockaddr_in6,
    pub ifra_broadaddr: libc::sockaddr_in6,
    pub ifra_mask: libc::sockaddr_in6,
}

impl ifreq {
    pub fn new<T: Into<String>>(name: T) -> Self {
        let mut req: ifreq = unsafe { mem::zeroed() };

        req.ifr_ifrn = name.into().parse().unwrap();
        req
    }
}
