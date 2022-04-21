#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use std::ffi::CString;
use std::fmt::Debug;
use std::mem;

nix::ioctl_write_ptr_bad!(siocsifmtu, libc::SIOCSIFMTU, ifreq);
nix::ioctl_write_ptr_bad!(siocsifflags, libc::SIOCSIFFLAGS, ifreq);
nix::ioctl_write_ptr_bad!(siocsifaddr, libc::SIOCSIFADDR, ifreq);
nix::ioctl_write_ptr_bad!(siocsifdstaddr, libc::SIOCSIFDSTADDR, ifreq);
nix::ioctl_write_ptr_bad!(siocsifbrdaddr, libc::SIOCSIFBRDADDR, ifreq);
nix::ioctl_write_ptr_bad!(siocsifnetmask, libc::SIOCSIFNETMASK, ifreq);

nix::ioctl_read_bad!(siocgifmtu, libc::SIOCGIFMTU, ifreq);
nix::ioctl_read_bad!(siocgifflags, libc::SIOCGIFFLAGS, ifreq);
nix::ioctl_read_bad!(siocgifaddr, libc::SIOCGIFADDR, ifreq);
nix::ioctl_read_bad!(siocgifdstaddr, libc::SIOCGIFDSTADDR, ifreq);
nix::ioctl_read_bad!(siocgifbrdaddr, libc::SIOCGIFBRDADDR, ifreq);
nix::ioctl_read_bad!(siocgifnetmask, libc::SIOCGIFNETMASK, ifreq);

pub(crate) type IfName = [libc::c_char; libc::IFNAMSIZ as _]; // Null-terminated

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ifreq {
    pub ifr_ifrn: IfName,
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
    pub ifru_slave: IfName,
    pub ifru_newname: IfName,
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

impl ifreq {
    fn make_ifname(mut name: String) -> IfName {
        name.truncate(libc::IFNAMSIZ - 1);
        let name = CString::new(name).unwrap();

        IfName::try_from(unsafe { &*(name.as_bytes() as *const _ as *const [i8]) }).unwrap()
    }

    pub fn new<T: Into<String>>(name: T) -> Self {
        let mut req: ifreq = unsafe { mem::zeroed() };

        req.ifr_ifrn = Self::make_ifname(name.into());
        req
    }

    pub fn name(&self) -> String {
        unsafe { std::ffi::CStr::from_ptr(self.ifr_ifrn.as_ptr()) }
            .to_string_lossy()
            .to_string()
    }
}
