use crate::Error;
use crate::{Interface, IpNet};

pub(crate) trait InterfaceHandleCommonT {
    fn addresses(&self) -> Result<Vec<IpNet>, Error>;
    fn add_address(&self, network: IpNet) -> Result<(), Error>;
    fn remove_address(&self, network: IpNet) -> Result<(), Error>;

    fn mtu(&self) -> Result<u32, Error>;
    fn set_mtu(&self, mtu: u32) -> Result<(), Error>;

    fn name(&self) -> Result<String, Error>;
    fn try_from_name(name: &str) -> Result<Interface, Error>;

    fn index(&self) -> Result<u32, Error>;
    fn try_from_index(index: u32) -> Result<Interface, Error>;

    fn hwaddress(&self) -> Result<[u8; 6], Error>;
}
