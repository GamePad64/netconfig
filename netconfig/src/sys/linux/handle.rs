use super::ifreq::{ifreq as InterfaceRequest, siocsifmtu};
use super::Metadata;
use crate::sys::InterfaceHandle;
use crate::{Error, InterfaceHandleCommonT};
use ipnet::IpNet;
use libc::{AF_INET, AF_INET6, NLM_F_MULTI};
use log::{debug, warn};
use netlink_packet_route::address::Nla as AddressNla;
use netlink_packet_route::link::nlas::Nla as LinkNla;
use netlink_packet_route::{
    AddressMessage, LinkMessage, NetlinkHeader, NetlinkMessage, NetlinkPayload, RtnlMessage,
    NLM_F_DUMP, NLM_F_REQUEST,
};
use netlink_sys::constants::NETLINK_ROUTE;
use netlink_sys::{Socket, SocketAddr};
use nix::ifaddrs::getifaddrs;
use nix::sys::socket::SockAddr::Inet;
use std::ffi::CStr;
use std::net;
use std::net::IpAddr;
use std::os::unix::io::AsRawFd;

pub trait InterfaceHandleExt {
    fn try_from_name(name: &str) -> Result<crate::InterfaceHandle, Error>;
}

fn indextoname(index: u32) -> Result<String, Error> {
    let mut buf = vec![0i8; libc::IF_NAMESIZE + 1];
    let ret_buf = unsafe { libc::if_indextoname(index, buf.as_mut_ptr() as _) };

    if ret_buf.is_null() {
        return Err(Error::InterfaceNotFound);
    }

    println!("{buf:?}");

    match unsafe { CStr::from_ptr(buf.as_ptr()) }.to_str() {
        Ok(s) => Ok(s.to_string()),
        Err(_) => Err(Error::UnexpectedMetadata),
    }
}

impl InterfaceHandle {
    fn name(&self) -> Result<String, Error> {
        indextoname(self.index)
    }
}

impl InterfaceHandleCommonT for InterfaceHandle {
    fn metadata(&self) -> Result<crate::Metadata, Error> {
        let mut metadata = Metadata {
            handle: crate::InterfaceHandle(*self),
            index: self.index,
            ..Metadata::default()
        };

        let mut socket = Socket::new(NETLINK_ROUTE).unwrap();
        socket.bind_auto().unwrap();
        socket.connect(&SocketAddr::new(0, 0)).unwrap();

        let mut message = LinkMessage::default();
        message.header.change_mask = 0xffff_ffff;
        message.header.index = metadata.index;

        let mut req = NetlinkMessage {
            header: NetlinkHeader {
                flags: NLM_F_REQUEST,
                ..Default::default()
            },
            payload: NetlinkPayload::from(RtnlMessage::GetLink(message)),
        };

        req.finalize();

        let mut buf = vec![0; req.header.length as usize];
        req.serialize(&mut buf[..]);

        debug!(">>> {:?}", req);
        socket.send(&buf[..], 0).unwrap();

        let mut receive_buffer = vec![0; 4096];
        let mut offset = 0;

        'outer: loop {
            let size = socket.recv(&mut &mut receive_buffer[..], 0).unwrap();

            loop {
                let bytes = &receive_buffer[offset..];
                // Parse the message
                let msg: NetlinkMessage<RtnlMessage> = NetlinkMessage::deserialize(bytes).unwrap();

                match msg.payload {
                    NetlinkPayload::Done => break 'outer,
                    NetlinkPayload::InnerMessage(RtnlMessage::NewLink(entry)) => {
                        debug!("entry: {:?}", entry);
                        for nla in entry.nlas {
                            match nla {
                                LinkNla::Mtu(mtu) => metadata.mtu = mtu,
                                LinkNla::IfName(name) => metadata.name = name,
                                _ => {}
                            }
                        }
                    }
                    NetlinkPayload::Error(err) => {
                        eprintln!("Received a netlink error message: {:?}", err);
                        // return;
                    }
                    _ => {
                        warn!("Unexpected message: {:?}", msg);
                    }
                }

                // Got non-multipart message
                if (msg.header.flags & (NLM_F_MULTI as u16)) == 0 {
                    break 'outer;
                }

                offset += msg.header.length as usize;
                if offset == size || msg.header.length == 0 {
                    offset = 0;
                    break;
                }
            }
        }
        Ok(crate::Metadata(metadata))
    }

    fn add_ip(&self, network: IpNet) {
        let mut socket = Socket::new(NETLINK_ROUTE).unwrap();
        socket.bind_auto().unwrap();
        socket.connect(&SocketAddr::new(0, 0)).unwrap();

        let message = make_address_message(self.index, network);

        let mut req = NetlinkMessage {
            header: NetlinkHeader {
                flags: NLM_F_DUMP | NLM_F_REQUEST,
                ..Default::default()
            },
            payload: NetlinkPayload::from(RtnlMessage::NewAddress(message)),
        };

        req.finalize();

        let mut buf = vec![0; req.header.length as usize];
        req.serialize(&mut buf[..]);

        debug!(">>> {:?}", req);
        socket.send(&buf[..], 0).unwrap();
    }

    fn remove_ip(&self, network: IpNet) {
        let mut socket = Socket::new(NETLINK_ROUTE).unwrap();
        socket.bind_auto().unwrap();
        socket.connect(&SocketAddr::new(0, 0)).unwrap();

        let message = make_address_message(self.index, network);

        let mut req = NetlinkMessage {
            header: NetlinkHeader {
                flags: NLM_F_DUMP | NLM_F_REQUEST,
                ..Default::default()
            },
            payload: NetlinkPayload::from(RtnlMessage::DelAddress(message)),
        };

        req.finalize();

        let mut buf = vec![0; req.header.length as usize];
        req.serialize(&mut buf[..]);

        debug!(">>> {:?}", req);
        socket.send(&buf[..], 0).unwrap();
    }

    fn get_addresses(&self) -> Result<Vec<IpNet>, Error> {
        let mut result = vec![];
        let interface_name = self.name()?;

        for interface in getifaddrs()?.filter(|x| x.interface_name == interface_name) {
            if interface.address.is_none() || interface.netmask.is_none() {
                continue;
            }

            if let (Some(Inet(address)), Some(Inet(netmask))) =
                (interface.address, interface.netmask)
            {
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

    fn set_mtu(&self, mtu: u32) -> Result<(), Error> {
        let mut req = InterfaceRequest::new(&*self.name()?);
        req.ifr_ifru.ifru_mtu = mtu as libc::c_int;

        let socket = make_dummy_socket();

        unsafe { siocsifmtu(socket.as_raw_fd(), &req) }?;
        Ok(())
    }

    fn try_from_index(index: u32) -> Result<crate::InterfaceHandle, Error> {
        let handle = crate::InterfaceHandle::from_index_unchecked(index);
        indextoname(index).map(|_| handle)
    }

    fn try_from_name(name: &str) -> Result<crate::InterfaceHandle, Error> {
        match unsafe { libc::if_nametoindex(name.as_ptr() as _) } {
            0 => Err(Error::InterfaceNotFound),
            n => Ok(crate::InterfaceHandle::from_index_unchecked(n)),
        }
    }
}

fn make_address_message(index: u32, network: IpNet) -> AddressMessage {
    let mut message = AddressMessage::default();
    message.header.prefix_len = network.prefix_len();
    message.header.index = index;

    let address_vec = match network.addr() {
        IpAddr::V4(ipv4) => {
            message.header.family = AF_INET as u8;
            ipv4.octets().to_vec()
        }
        IpAddr::V6(ipv6) => {
            message.header.family = AF_INET6 as u8;
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

fn make_dummy_socket() -> net::UdpSocket {
    net::UdpSocket::bind("[::1]:0").expect("Socket is bound")
}
