#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use super::InterfaceName;
use std::fmt::Debug;
use std::mem;

#[repr(C)]
#[derive(Copy, Clone, Default)]
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

impl Default for ifreq_ifru {
    fn default() -> Self {
        unsafe { mem::zeroed() }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
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
pub struct ifaliasreq<SA> {
    pub ifra_name: InterfaceName,
    pub ifra_addr: SA,
    pub ifra_broadaddr: SA,
    pub ifra_mask: SA,
}

pub type ifaliasreq4 = ifaliasreq<libc::sockaddr_in>;
pub type ifaliasreq6 = ifaliasreq<libc::sockaddr_in6>;

impl ifreq {
    pub fn new<T: Into<String>>(name: T) -> Self {
        ifreq {
            ifr_ifrn: InterfaceName::try_from(&*name.into()).unwrap(),
            ..Default::default()
        }
    }
}
