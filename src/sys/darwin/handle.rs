use super::scinterface::SCNetworkInterface;
use super::Metadata;
use crate::sys::posix::{ifaceaddr, indextoname, nametoindex};
use crate::sys::InterfaceHandle;
use crate::{Error, InterfaceHandleCommonT};
use ipnet::IpNet;

pub trait InterfaceHandleExt {}

impl InterfaceHandle {
    fn name(&self) -> Result<String, Error> {
        indextoname(self.index)
    }
}

impl InterfaceHandleCommonT for InterfaceHandle {
    fn metadata(&self) -> Result<crate::Metadata, Error> {
        let name = self.name()?;
        let metadata = Metadata {
            handle: crate::InterfaceHandle(*self),
            index: self.index,
            name: name.clone(),
            alias: SCNetworkInterface::get_displayname(&*name).unwrap_or_else(|| name.clone()),
            ..Default::default()
        };

        Ok(crate::Metadata(metadata))
    }

    fn add_ip(&self, network: IpNet) {
        todo!()
    }

    fn remove_ip(&self, network: IpNet) {
        todo!()
    }

    fn get_addresses(&self) -> Result<Vec<IpNet>, Error> {
        ifaceaddr(&*self.name()?)
    }

    fn set_mtu(&self, mtu: u32) -> Result<(), Error> {
        todo!()
    }

    fn try_from_index(index: u32) -> Result<crate::InterfaceHandle, Error> {
        indextoname(index).map(|_| crate::InterfaceHandle::from_index_unchecked(index))
    }

    fn try_from_name(name: &str) -> Result<crate::InterfaceHandle, Error> {
        nametoindex(name).map(|n| crate::InterfaceHandle::from_index_unchecked(n))
    }
}
