use crate::Error;
use ipnet::IpNet;
use nix::ifaddrs::getifaddrs;
use nix::sys::socket::SockAddr::Inet;
use std::ffi::{CStr, CString};
use std::net::IpAddr;

pub(crate) fn indextoname(index: u32) -> Result<String, Error> {
    let mut buf = [0i8; libc::IF_NAMESIZE];
    let ret_buf = unsafe { libc::if_indextoname(index, buf.as_mut_ptr() as _) };

    if ret_buf.is_null() {
        return Err(Error::InterfaceNotFound);
    }

    match unsafe { CStr::from_ptr(buf.as_ptr()) }.to_str() {
        Ok(s) => Ok(s.to_string()),
        Err(_) => Err(Error::UnexpectedMetadata),
    }
}

pub(crate) fn nametoindex(name: &str) -> Result<u32, Error> {
    let cname = CString::new(name).map_err(|_| Error::InvalidParameter)?;
    match unsafe { libc::if_nametoindex(cname.as_ptr() as _) } {
        0 => Err(Error::InterfaceNotFound),
        n => Ok(n),
    }
}

pub(crate) fn ifaceaddr(interface_name: &str) -> Result<Vec<IpNet>, Error> {
    let mut result = vec![];

    for interface in getifaddrs()?.filter(|x| x.interface_name == interface_name) {
        if interface.address.is_none() || interface.netmask.is_none() {
            continue;
        }

        if let (Some(Inet(address)), Some(Inet(netmask))) = (interface.address, interface.netmask) {
            let prefix_len: u8 = match netmask.ip().to_std() {
                IpAddr::V4(addr) => addr
                    .octets()
                    .iter()
                    .map(|byte| byte.leading_ones() as u8)
                    .sum(),
                IpAddr::V6(addr) => addr
                    .octets()
                    .iter()
                    .map(|byte| byte.leading_ones() as u8)
                    .sum(),
            };

            result.push(
                IpNet::new(address.ip().to_std(), prefix_len)
                    .expect("IP address and netmask converted"),
            );
        }
    }
    Ok(result)
}
