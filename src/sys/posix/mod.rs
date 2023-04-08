mod handle;
mod ifacename;
pub mod ifreq;

use crate::Error;
pub use ifacename::InterfaceName;

use std::net;

pub(crate) mod ioctls;

pub(crate) fn dummy_socket() -> Result<net::UdpSocket, Error> {
    Ok(net::UdpSocket::bind("0:0")?)
}

pub(crate) fn list_interfaces() -> Result<Vec<crate::Interface>, Error> {
    Ok(nix::net::if_::if_nameindex()?
        .iter()
        .map(|a| crate::Interface::from_index_unchecked(a.index()))
        .collect())
}
