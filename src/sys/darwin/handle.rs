use super::scinterface::SCNetworkInterface;
use super::Metadata;
use crate::sys::posix::{
    if_add_addr, if_addr, if_flags, if_indextoname, if_mtu, if_nametoindex, if_set_flags,
    if_set_mtu,
};
use crate::sys::InterfaceHandle;
use crate::{Error, InterfaceHandleCommonT};
use delegate::delegate;
use ipnet::IpNet;

pub trait InterfaceHandleExt {
    fn set_up(&self, v: bool) -> Result<(), Error>;
    fn set_running(&self, v: bool) -> Result<(), Error>;
}

impl InterfaceHandleExt for crate::InterfaceHandle {
    delegate! {
        to self.0 {
            fn set_up(&self, v: bool) -> Result<(), Error>;
            fn set_running(&self, v: bool) -> Result<(), Error>;
        }
    }
}

impl InterfaceHandle {
    fn name(&self) -> Result<String, Error> {
        if_indextoname(self.index)
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
            mtu: if_mtu(&*name)?,
            ..Default::default()
        };

        Ok(crate::Metadata(metadata))
    }

    fn add_ip(&self, network: IpNet) {
        if_add_addr(&*self.name().unwrap(), network).unwrap()
    }

    fn remove_ip(&self, _network: IpNet) {
        todo!()
    }

    fn get_addresses(&self) -> Result<Vec<IpNet>, Error> {
        if_addr(&*self.name()?)
    }

    fn set_mtu(&self, mtu: u32) -> Result<(), Error> {
        if_set_mtu(&*self.name()?, mtu)
    }

    fn try_from_index(index: u32) -> Result<crate::InterfaceHandle, Error> {
        if_indextoname(index).map(|_| crate::InterfaceHandle::from_index_unchecked(index))
    }

    fn try_from_name(name: &str) -> Result<crate::InterfaceHandle, Error> {
        if_nametoindex(name).map(crate::InterfaceHandle::from_index_unchecked)
    }
}

impl InterfaceHandleExt for InterfaceHandle {
    fn set_up(&self, v: bool) -> Result<(), Error> {
        if_set_flags_masked(&*self.name()?, libc::IFF_UP as i16, v).map(|_| ())
    }

    fn set_running(&self, v: bool) -> Result<(), Error> {
        if_set_flags_masked(&*self.name()?, libc::IFF_RUNNING as i16, v).map(|_| ())
    }
}
