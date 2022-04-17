use crate::{Error, Metadata};
use crate::{InterfaceHandle, IpNet};

pub(crate) trait MetadataCommonT {
    fn name(&self) -> String;
    fn handle(&self) -> InterfaceHandle;
    fn mtu(&self) -> u32;
    fn index(&self) -> u32;
}

pub(crate) trait InterfaceHandleCommonT {
    fn metadata(&self) -> Result<Metadata, Error>;
    fn add_ip(&self, network: IpNet);
    fn remove_ip(&self, network: IpNet);
    fn get_addresses(&self) -> Result<Vec<IpNet>, Error>;
    fn set_mtu(&self, mtu: u32) -> Result<(), Error>;

    fn try_from_index(index: u32) -> Result<crate::InterfaceHandle, Error>;
    fn try_from_name(name: &str) -> Result<crate::InterfaceHandle, Error>;
}
