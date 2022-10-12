mod error;
use advmac::MacAddr6;
use delegate::delegate;
pub use error::Error;
pub use ipnet;
use ipnet::IpNet;
use std::collections::HashSet;

pub mod sys;

/// Wrapped interface index.
///
/// Index is chosen, because basically all operating systems use index as an identifier.
/// This struct can be used to manipulate interface parameters, such as IP address and MTU.
#[derive(Debug)]
pub struct Interface(sys::InterfaceHandle);

impl Interface {
    delegate! {
        to self.0 {
            pub fn add_address(&self, network: IpNet) -> Result<(), Error>;
            pub fn remove_address(&self, network: IpNet) -> Result<(), Error>;
            /// Returns array of IP addresses, assigned to this Interface
            pub fn addresses(&self) -> Result<Vec<IpNet>, Error>;

            pub fn mtu(&self) -> Result<u32, Error>;
            pub fn set_mtu(&self, mtu: u32) -> Result<(), Error>;

            pub fn name(&self) -> Result<String, Error>;
            pub fn index(&self) -> Result<u32, Error>;
            /// Returns MAC address, assigned to this Interface
            pub fn hwaddress(&self) -> Result<MacAddr6, Error>;
        }
    }

    pub fn from_index_unchecked(index: u32) -> Self {
        Self(sys::InterfaceHandle { index })
    }

    /// Returns `InterfaceHandle` from given interface index or Error if not found.
    ///
    /// This method checks given index for validity and interface for presence. If you want to get
    /// `InterfaceHandle` without checking interface for presence, use [`from_index_unchecked`](Self::from_index_unchecked).
    pub fn try_from_index(index: u32) -> Result<Self, Error> {
        sys::InterfaceHandle::try_from_index(index)
    }

    /// Returns `InterfaceHandle` from given name or Error if not found.
    ///
    /// On Windows it uses interface name, that is similar to `ethernet_32774`.
    /// If you want to search interface by human-readable name (like `Ethernet 1`), use `try_from_alias`
    pub fn try_from_name(name: &str) -> Result<Self, Error> {
        sys::InterfaceHandle::try_from_name(name)
    }
}

pub fn list_interfaces() -> Result<Vec<Interface>, Error> {
    sys::list_interfaces()
}

pub fn list_addresses() -> Result<Vec<IpNet>, Error> {
    let interfaces = list_interfaces()?;

    let addresses = interfaces
        .iter()
        .flat_map(|iface| iface.addresses())
        .flatten();

    Ok(HashSet::<IpNet>::from_iter(addresses)
        .iter()
        .cloned()
        .collect())
}
