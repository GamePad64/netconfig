mod ifacename;
pub mod ifreq;
pub use ifacename::InterfaceName;

use crate::Error;
use ipnet::IpNet;
use libc::ARPHRD_ETHER;
use nix::ifaddrs::getifaddrs;
use nix::sys::socket::AddressFamily::{Inet, Inet6};
use nix::sys::socket::{SockaddrIn, SockaddrIn6, SockaddrLike};
use std::net;
use std::os::unix::io::AsRawFd;

mod ioctls {
    cfg_if::cfg_if! {
        if #[cfg(target_os = "linux")] {
            nix::ioctl_write_ptr_bad!(siocsifmtu, libc::SIOCSIFMTU, super::ifreq::ifreq);
            nix::ioctl_write_ptr_bad!(siocsifflags, libc::SIOCSIFFLAGS, super::ifreq::ifreq);
            nix::ioctl_write_ptr_bad!(siocsifaddr, libc::SIOCSIFADDR, super::ifreq::ifreq);
            nix::ioctl_write_ptr_bad!(siocsifdstaddr, libc::SIOCSIFDSTADDR, super::ifreq::ifreq);
            nix::ioctl_write_ptr_bad!(siocsifbrdaddr, libc::SIOCSIFBRDADDR, super::ifreq::ifreq);
            nix::ioctl_write_ptr_bad!(siocsifnetmask, libc::SIOCSIFNETMASK, super::ifreq::ifreq);
            nix::ioctl_write_ptr_bad!(siocsifhwaddr, libc::SIOCSIFHWADDR, super::ifreq::ifreq);

            nix::ioctl_read_bad!(siocgifmtu, libc::SIOCGIFMTU, super::ifreq::ifreq);
            nix::ioctl_read_bad!(siocgifflags, libc::SIOCGIFFLAGS, super::ifreq::ifreq);
            nix::ioctl_read_bad!(siocgifaddr, libc::SIOCGIFADDR, super::ifreq::ifreq);
            nix::ioctl_read_bad!(siocgifdstaddr, libc::SIOCGIFDSTADDR, super::ifreq::ifreq);
            nix::ioctl_read_bad!(siocgifbrdaddr, libc::SIOCGIFBRDADDR, super::ifreq::ifreq);
            nix::ioctl_read_bad!(siocgifnetmask, libc::SIOCGIFNETMASK, super::ifreq::ifreq);
        } else if #[cfg(target_os = "macos")] {
            nix::ioctl_readwrite!(siocgifmtu, b'i', 51, super::ifreq::ifreq);
            nix::ioctl_write_ptr!(siocsifmtu, b'i', 52, super::ifreq::ifreq);
            nix::ioctl_readwrite!(siocgifflags, b'i', 17, super::ifreq::ifreq);
            nix::ioctl_write_ptr!(siocsifflags, b'i', 16, super::ifreq::ifreq);
            nix::ioctl_write_ptr!(siocaifaddr4, b'i', 26, super::ifreq::ifaliasreq4);
            nix::ioctl_write_ptr!(siocaifaddr6, b'i', 26, super::ifreq::ifaliasreq6);
            nix::ioctl_write_ptr!(siocdifaddr, b'i', 25, super::ifreq::ifreq);
        }
    }
}

pub(crate) fn if_indextoname(index: u32) -> Result<String, Error> {
    let mut buf = InterfaceName::default();
    let ret_buf = unsafe { libc::if_indextoname(index, buf.as_mut_ptr()) };

    if ret_buf.is_null() {
        return Err(Error::InterfaceNotFound);
    }

    String::try_from(&buf).map_err(|_| Error::InvalidParameter)
}

pub(crate) fn if_nametoindex(name: &str) -> Result<u32, Error> {
    let name = InterfaceName::try_from(name).map_err(|_| Error::InvalidParameter)?;

    match unsafe { libc::if_nametoindex(name.as_ptr()) } {
        0 => Err(Error::InterfaceNotFound),
        n => Ok(n),
    }
}

pub(crate) fn if_mtu(name: &str) -> Result<u32, Error> {
    let mut req = ifreq::ifreq::new(name);

    let socket = make_dummy_socket();

    unsafe { ioctls::siocgifmtu(socket.as_raw_fd(), &mut req) }?;
    Ok(unsafe { req.ifr_ifru.ifru_mtu } as u32)
}

pub(crate) fn if_set_mtu(name: &str, mtu: u32) -> Result<(), Error> {
    let mut req = ifreq::ifreq::new(name);
    req.ifr_ifru.ifru_mtu = mtu as libc::c_int;

    let socket = make_dummy_socket();

    unsafe { ioctls::siocsifmtu(socket.as_raw_fd(), &req) }?;
    Ok(())
}

pub(crate) fn if_set_hwaddress(name: &str, hwaddress: [u8; 6]) -> Result<(), Error> {
    let mut req = ifreq::ifreq::new(name);
    req.ifr_ifru.ifru_hwaddr = libc::sockaddr {
        sa_family: ARPHRD_ETHER,
        sa_data: unsafe { std::mem::zeroed() },
    };
    for i in 0..6 {
        unsafe {
            req.ifr_ifru.ifru_hwaddr.sa_data[i] = hwaddress[i] as libc::c_char;
        }
    }

    let socket = make_dummy_socket();

    unsafe { ioctls::siocsifhwaddr(socket.as_raw_fd(), &req) }?;
    Ok(())
}

pub(crate) fn if_flags(name: &str) -> Result<i16, Error> {
    let mut req = ifreq::ifreq::new(name);

    let socket = make_dummy_socket();

    unsafe {
        ioctls::siocgifflags(socket.as_raw_fd(), &mut req)?;
        Ok(req.ifr_ifru.ifru_flags)
    }
}

pub(crate) fn if_set_flags(name: &str, flags: i16) -> Result<i16, Error> {
    let mut req = ifreq::ifreq::new(name);
    req.ifr_ifru.ifru_flags = flags;

    let socket = make_dummy_socket();

    unsafe {
        ioctls::siocsifflags(socket.as_raw_fd(), &req)?;
        Ok(req.ifr_ifru.ifru_flags)
    }
}

pub(crate) fn if_set_flags_masked(name: &str, mask: i16, v: bool) -> Result<i16, Error> {
    let mut flags = if_flags(name)?;
    if v {
        flags |= mask;
    } else {
        flags &= !mask;
    }
    if_set_flags(name, flags)
}

#[cfg(target_os = "macos")]
pub(crate) fn if_add_addr(name: &str, addr: IpNet) -> Result<(), Error> {
    let socket = make_dummy_socket();
    match addr {
        IpNet::V4(addr4) => {
            let ifra_addr = SockaddrIn::from(net::SocketAddrV4::new(addr4.addr(), 0));
            let ifra_broadaddr = SockaddrIn::from(net::SocketAddrV4::new(addr4.broadcast(), 0));
            let ifra_mask = SockaddrIn::from(net::SocketAddrV4::new(addr4.netmask(), 0));

            let req = ifreq::ifaliasreq4 {
                ifra_name: name.parse().unwrap(),
                ifra_addr: *ifra_addr.as_ref(),
                ifra_broadaddr: *ifra_broadaddr.as_ref(),
                ifra_mask: *ifra_mask.as_ref(),
            };

            unsafe {
                ioctls::siocaifaddr4(socket.as_raw_fd(), &req)?;
            }
            Ok(())
        }
        IpNet::V6(addr6) => {
            let ifra_addr = SockaddrIn6::from(net::SocketAddrV6::new(addr6.addr(), 0, 0, 0));
            let ifra_broadaddr =
                SockaddrIn6::from(net::SocketAddrV6::new(addr6.broadcast(), 0, 0, 0));
            let ifra_mask = SockaddrIn6::from(net::SocketAddrV6::new(addr6.netmask(), 0, 0, 0));

            let req = ifreq::ifaliasreq6 {
                ifra_name: name.parse().unwrap(),
                ifra_addr: *ifra_addr.as_ref(),
                ifra_broadaddr: *ifra_broadaddr.as_ref(),
                ifra_mask: *ifra_mask.as_ref(),
            };

            unsafe {
                ioctls::siocaifaddr6(socket.as_raw_fd(), &req)?;
            }
            Ok(())
        }
    }
}

pub(crate) fn if_addr(interface_name: &str) -> Result<Vec<IpNet>, Error> {
    let mut result = vec![];

    for interface in getifaddrs()?.filter(|x| x.interface_name == interface_name) {
        if interface.address.is_none() || interface.netmask.is_none() {
            continue;
        }

        if let (Some(address), Some(netmask)) = (interface.address, interface.netmask) {
            let network = if address.family().unwrap() == Inet && netmask.family().unwrap() == Inet
            {
                let addr: net::Ipv4Addr = address.as_sockaddr_in().unwrap().ip().into();
                let prefix =
                    ipnetwork::ipv4_mask_to_prefix(netmask.as_sockaddr_in().unwrap().ip().into())
                        .unwrap();
                IpNet::new(addr.into(), prefix).unwrap()
            } else if address.family().unwrap() == Inet6 && netmask.family().unwrap() == Inet6 {
                let addr: net::Ipv6Addr = address.as_sockaddr_in6().unwrap().ip();
                let prefix =
                    ipnetwork::ipv6_mask_to_prefix(netmask.as_sockaddr_in6().unwrap().ip())
                        .unwrap();
                IpNet::new(addr.into(), prefix).unwrap()
            } else {
                return Err(Error::UnexpectedMetadata);
            };

            result.push(network);
        }
    }
    Ok(result)
}

pub(crate) fn make_dummy_socket() -> net::UdpSocket {
    net::UdpSocket::bind("[::1]:0").expect("Socket is bound")
}
