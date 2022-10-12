use crate::sys::ifreq::ifreq;
use crate::sys::{dummy_socket, ioctls, InterfaceHandle};
use crate::{Error, Interface};
use advmac::MacAddr6;
use delegate::delegate;
use ipnet::IpNet;
use libc::{AF_INET, AF_INET6, ARPHRD_ETHER};
use log::debug;
use netlink_packet_route::{
    address::Nla as AddressNla, AddressMessage, NetlinkHeader, NetlinkMessage, NetlinkPayload,
    RtnlMessage, NLM_F_DUMP, NLM_F_REQUEST,
};
use netlink_sys::constants::NETLINK_ROUTE;
use netlink_sys::{Socket, SocketAddr};
use std::net::IpAddr;
use std::os::unix::io::AsRawFd;

// Public interface (platform extension)
pub trait InterfaceExt {
    fn set_up(&self, v: bool) -> Result<(), Error>;
    fn set_running(&self, v: bool) -> Result<(), Error>;
    fn set_hwaddress(&self, hwaddress: MacAddr6) -> Result<(), Error>;
}

// Private interface
impl InterfaceHandle {
    pub fn add_address(&self, network: IpNet) -> Result<(), Error> {
        let mut socket = Socket::new(NETLINK_ROUTE)?;
        socket.bind_auto()?;
        socket.connect(&SocketAddr::new(0, 0))?;

        let message = make_address_message(self.index, network);

        let mut req = NetlinkMessage {
            header: NetlinkHeader {
                flags: NLM_F_DUMP | NLM_F_REQUEST,
                ..Default::default()
            },
            payload: NetlinkPayload::from(RtnlMessage::NewAddress(message)),
        };

        req.finalize();

        let mut buf = vec![0; req.header.length as _];
        req.serialize(&mut buf);

        debug!(">>> {:?}", req);
        socket.send(&buf, 0)?;

        Ok(())
    }

    pub fn remove_address(&self, network: IpNet) -> Result<(), Error> {
        let mut socket = Socket::new(NETLINK_ROUTE)?;
        socket.bind_auto()?;
        socket.connect(&SocketAddr::new(0, 0))?;

        let message = make_address_message(self.index, network);

        let mut req = NetlinkMessage {
            header: NetlinkHeader {
                flags: NLM_F_REQUEST,
                ..Default::default()
            },
            payload: NetlinkPayload::from(RtnlMessage::DelAddress(message)),
        };

        req.finalize();

        let mut buf = vec![0; req.header.length as _];
        req.serialize(&mut buf);

        debug!(">>> {:?}", req);
        socket.send(&buf, 0).unwrap();

        Ok(())
    }

    pub fn hwaddress(&self) -> Result<MacAddr6, Error> {
        let mut req = ifreq::new(self.name()?);
        let socket = dummy_socket()?;

        unsafe { ioctls::siocgifhwaddr(socket.as_raw_fd(), &mut req) }?;
        Ok(unsafe { &req.ifr_ifru.ifru_hwaddr.sa_data[0..6] }
            .try_into()
            .unwrap())
    }
}

impl InterfaceExt for Interface {
    delegate! {
        to self.0 {
            fn set_up(&self, v: bool) -> Result<(), Error>;
            fn set_running(&self, v: bool) -> Result<(), Error>;
        }
    }

    fn set_hwaddress(&self, hwaddress: MacAddr6) -> Result<(), Error> {
        let mut req = ifreq::new(self.name()?);
        req.ifr_ifru.ifru_hwaddr = libc::sockaddr {
            sa_family: ARPHRD_ETHER,
            sa_data: unsafe { std::mem::zeroed() },
        };

        unsafe {
            req.ifr_ifru.ifru_hwaddr.sa_data[0..6].copy_from_slice(hwaddress.as_c_slice());
        }

        let socket = dummy_socket()?;

        unsafe { ioctls::siocsifhwaddr(socket.as_raw_fd(), &req) }?;
        Ok(())
    }
}

fn make_address_message(index: u32, network: IpNet) -> AddressMessage {
    let mut message = AddressMessage::default();
    message.header.prefix_len = network.prefix_len();
    message.header.index = index;

    let address_vec = match network.addr() {
        IpAddr::V4(ipv4) => {
            message.header.family = AF_INET as _;
            ipv4.octets().to_vec()
        }
        IpAddr::V6(ipv6) => {
            message.header.family = AF_INET6 as _;
            ipv6.octets().to_vec()
        }
    };

    if network.addr().is_multicast() {
        message.nlas.push(AddressNla::Multicast(address_vec));
    } else if network.addr().is_unspecified() {
        message.nlas.push(AddressNla::Unspec(address_vec));
    } else {
        message.nlas.push(AddressNla::Address(address_vec.clone()));

        if let IpNet::V4(network_v4) = network {
            // for IPv4 the IFA_LOCAL address can be set to the same value as IFA_ADDRESS
            message.nlas.push(AddressNla::Local(address_vec));
            // set the IFA_BROADCAST address as well (IPv6 does not support broadcast)
            message.nlas.push(AddressNla::Broadcast(
                network_v4.broadcast().octets().to_vec(),
            ));
        }
    }

    message
}
