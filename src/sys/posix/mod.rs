mod handle;
mod ifacename;
pub mod ifreq;

use crate::Error;
pub use ifacename::InterfaceName;

#[cfg(target_os = "macos")]
use nix::sys::socket::{SockaddrIn, SockaddrIn6};
use std::net;

pub(crate) mod ioctls;

#[cfg(target_os = "macos")]
pub(crate) fn if_add_addr(name: &str, addr: IpNet) -> Result<(), Error> {
    let socket = dummy_socket();
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

pub(crate) fn dummy_socket() -> Result<net::UdpSocket, Error> {
    Ok(net::UdpSocket::bind("[::1]:0")?)
}

pub(crate) fn list_interfaces() -> Result<Vec<crate::Interface>, Error> {
    Ok(nix::net::if_::if_nameindex()?
        .iter()
        .map(|a| crate::Interface::from_index_unchecked(a.index()))
        .collect())
}
