mod error;
mod traits;
use delegate::delegate;
pub use error::Error;
pub use ipnet;
use ipnet::IpNet;
use std::collections::HashSet;
use traits::{InterfaceHandleCommonT, MetadataCommonT};
pub mod sys;

/// Wrapped interface index.
///
/// Index is chosen, because basically all operating systems use index as an identifier.
/// This struct can be used to manipulate interface parameters, such as IP address and MTU.
#[derive(Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct InterfaceHandle(sys::InterfaceHandle);
pub struct Metadata(sys::Metadata);

impl Metadata {
    delegate! {
        to self.0 {
            pub fn name(&self) -> String;
            pub fn handle(&self) -> InterfaceHandle;
            pub fn mtu(&self) -> u32;
            pub fn index(&self) -> u32;
        }
    }
}

impl InterfaceHandle {
    delegate! {
        to self.0 {
            pub fn metadata(&self) -> Result<Metadata, Error>;
            pub fn add_ip(&self, network: IpNet);
            pub fn remove_ip(&self, network: IpNet);
            pub fn get_addresses(&self) -> Result<Vec<IpNet>, Error>;
            pub fn set_mtu(&self, mtu: u32) -> Result<(), Error>;
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

pub fn list_interfaces() -> Vec<crate::InterfaceHandle> {
    sys::list_interfaces()
}

pub fn list_addresses() -> Vec<IpNet> {
    let interfaces = list_interfaces();

    let addresses = interfaces
        .iter()
        .flat_map(|iface| iface.get_addresses())
        .flatten();

    HashSet::<IpNet>::from_iter(addresses)
        .iter()
        .cloned()
        .collect()
}
