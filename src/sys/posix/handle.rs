use crate::sys::posix::ifreq::ifreq;
use crate::sys::posix::{dummy_socket, ioctls, InterfaceName};
use crate::sys::InterfaceHandle;
use crate::{Error, Interface};
use ipnet::IpNet;
use nix::ifaddrs::getifaddrs;
use nix::net::if_::InterfaceFlags;
use nix::sys::socket::AddressFamily::{Inet, Inet6};
use nix::sys::socket::SockaddrLike;
use std::net::IpAddr;
use std::os::unix::io::AsRawFd;

impl InterfaceHandle {
    pub fn addresses(&self) -> Result<Vec<IpNet>, Error> {
        let mut result = vec![];
        let name = self.name()?;

        for interface in getifaddrs()?.filter(|x| x.interface_name == name) {
            let (Some(address), Some(netmask)) = (interface.address, interface.netmask) else { continue; };

            let (address, netmask) = match (address.family(), netmask.family()) {
                (Some(Inet), Some(Inet)) => (
                    IpAddr::V4(address.as_sockaddr_in().unwrap().ip().into()),
                    IpAddr::V4(netmask.as_sockaddr_in().unwrap().ip().into()),
                ),
                (Some(Inet6), Some(Inet6)) => (
                    IpAddr::V6(address.as_sockaddr_in6().unwrap().ip()),
                    IpAddr::V6(netmask.as_sockaddr_in6().unwrap().ip()),
                ),
                (_, _) => continue,
            };

            let prefix = ipnet::ip_mask_to_prefix(netmask).unwrap();

            result.push(IpNet::new(address, prefix).unwrap());
        }
        Ok(result)
    }

    pub fn mtu(&self) -> Result<u32, Error> {
        let mut req = ifreq::new(self.name()?);
        let socket = dummy_socket()?;

        unsafe {
            ioctls::siocgifmtu(socket.as_raw_fd(), &mut req)?;
            Ok(req.ifr_ifru.ifru_mtu as _)
        }
    }

    pub fn set_mtu(&self, mtu: u32) -> Result<(), Error> {
        let mut req = ifreq::new(self.name()?);
        req.ifr_ifru.ifru_mtu = mtu as _;

        let socket = dummy_socket()?;

        unsafe { ioctls::siocsifmtu(socket.as_raw_fd(), &req) }?;
        Ok(())
    }

    pub fn name(&self) -> Result<String, Error> {
        let mut buf = InterfaceName::default();
        let ret_buf = unsafe { libc::if_indextoname(self.index, buf.as_mut_ptr()) };

        if ret_buf.is_null() {
            return Err(Error::InterfaceNotFound);
        }

        buf.try_into().map_err(|_| Error::InvalidParameter)
    }

    pub fn try_from_name(name: &str) -> Result<Interface, Error> {
        let name = InterfaceName::try_from(name).map_err(|_| Error::InvalidParameter)?;

        match unsafe { libc::if_nametoindex(name.as_ptr()) } {
            0 => Err(Error::InterfaceNotFound),
            n => Ok(Interface::from_index_unchecked(n)),
        }
    }

    pub fn try_from_index(index: u32) -> Result<Interface, Error> {
        match nix::net::if_::if_nameindex()?
            .iter()
            .find(|if_| if_.index() == index)
        {
            Some(_) => Ok(Interface::from_index_unchecked(index)),
            None => Err(Error::InterfaceNotFound),
        }
    }

    pub fn index(&self) -> Result<u32, Error> {
        Ok(self.index)
    }

    pub fn set_up(&self, v: bool) -> Result<(), Error> {
        let mut flags = self.flags()?;
        flags.set(InterfaceFlags::IFF_UP, v);
        self.set_flags(flags)?;
        Ok(())
    }

    pub fn set_running(&self, v: bool) -> Result<(), Error> {
        let mut flags = self.flags()?;
        flags.set(InterfaceFlags::IFF_RUNNING, v);
        self.set_flags(flags)?;
        Ok(())
    }
}

impl InterfaceHandle {
    pub(crate) fn flags(&self) -> Result<InterfaceFlags, Error> {
        let mut req = ifreq::new(self.name()?);
        let socket = dummy_socket()?;

        unsafe {
            ioctls::siocgifflags(socket.as_raw_fd(), &mut req)?;
            Ok(InterfaceFlags::from_bits_truncate(
                req.ifr_ifru.ifru_flags as _,
            ))
        }
    }

    pub(crate) fn set_flags(&self, flags: InterfaceFlags) -> Result<InterfaceFlags, Error> {
        let mut req = ifreq::new(self.name()?);
        req.ifr_ifru.ifru_flags = flags.bits() as _;

        let socket = dummy_socket()?;

        unsafe {
            ioctls::siocsifflags(socket.as_raw_fd(), &req)?;
            Ok(InterfaceFlags::from_bits_truncate(
                req.ifr_ifru.ifru_flags as _,
            ))
        }
    }
}
