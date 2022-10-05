use crate::sys::InterfaceHandle;
use crate::{Error, Interface, InterfaceHandleCommonT};
use ipnet::IpNet;
use log::warn;
use std::collections::HashSet;
use std::io::{self, ErrorKind};
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};
use widestring::U16CString;
use windows::core::{Error as WinError, GUID, HRESULT, HSTRING};
use windows::Win32::NetworkManagement::IpHelper::{
    ConvertInterfaceGuidToLuid, ConvertInterfaceIndexToLuid, ConvertInterfaceLuidToAlias,
    ConvertInterfaceLuidToGuid, ConvertInterfaceLuidToIndex, ConvertInterfaceLuidToNameW,
    ConvertInterfaceNameToLuidW, CreateUnicastIpAddressEntry, DeleteUnicastIpAddressEntry,
    FreeMibTable, GetIfEntry2, GetIpInterfaceEntry, GetUnicastIpAddressTable,
    InitializeUnicastIpAddressEntry, SetIpInterfaceEntry, MIB_IF_ROW2, MIB_IPINTERFACE_ROW,
    MIB_UNICASTIPADDRESS_ROW,
};
use windows::Win32::NetworkManagement::Ndis::{IF_MAX_STRING_SIZE, NET_LUID_LH};
use windows::Win32::Networking::WinSock::{
    ADDRESS_FAMILY, AF_INET, AF_INET6, AF_UNSPEC, SOCKADDR_INET,
};

const ERROR_ACCESS_DENIED: HRESULT = windows::Win32::Foundation::ERROR_ACCESS_DENIED.to_hresult();
const ERROR_FILE_NOT_FOUND: HRESULT = windows::Win32::Foundation::ERROR_FILE_NOT_FOUND.to_hresult();
const ERROR_INVALID_NAME: HRESULT = windows::Win32::Foundation::ERROR_INVALID_NAME.to_hresult();
const ERROR_INVALID_PARAMETER: HRESULT =
    windows::Win32::Foundation::ERROR_INVALID_PARAMETER.to_hresult();
const ERROR_NOT_FOUND: HRESULT = windows::Win32::Foundation::ERROR_NOT_FOUND.to_hresult();

fn convert_sockaddr(sa: SOCKADDR_INET) -> SocketAddr {
    unsafe {
        match ADDRESS_FAMILY(sa.si_family as _) {
            AF_INET => SocketAddr::new(
                Ipv4Addr::from(sa.Ipv4.sin_addr).into(),
                u16::from_be(sa.Ipv4.sin_port),
            ),
            AF_INET6 => SocketAddr::new(
                Ipv6Addr::from(sa.Ipv6.sin6_addr).into(),
                u16::from_be(sa.Ipv6.sin6_port),
            ),
            _ => panic!("Invalid address family"),
        }
    }
}

impl InterfaceHandle {
    fn luid(&self) -> Result<u64, Error> {
        let mut luid = NET_LUID_LH::default();

        let code = unsafe { ConvertInterfaceIndexToLuid(self.index, &mut luid) };
        match code.map_err(HRESULT::from) {
            Ok(_) => Ok(unsafe { luid.Value }),
            Err(ERROR_FILE_NOT_FOUND) => Err(Error::InterfaceNotFound),
            Err(e) => Err(WinError::from(e).into()),
        }
    }

    fn mib_if_row2(&self) -> Result<MIB_IF_ROW2, Error> {
        let mut row = MIB_IF_ROW2 {
            InterfaceIndex: self.index,
            ..Default::default()
        };
        unsafe {
            GetIfEntry2(&mut row).map_err(|_| Error::InterfaceNotFound)?;
        }
        Ok(row)
    }

    fn net_luid_lh(&self) -> Result<NET_LUID_LH, Error> {
        Ok(NET_LUID_LH {
            Value: self.luid()?,
        })
    }
}

pub trait InterfaceExt {
    fn try_from_luid(luid: u64) -> Result<Interface, Error>;
    fn try_from_guid(guid: u128) -> Result<Interface, Error>;
    fn try_from_alias(alias: &str) -> Result<Interface, Error>;

    fn luid(&self) -> Result<u64, Error>;
    fn guid(&self) -> Result<u128, Error>;
    fn index(&self) -> Result<u32, Error>;
    fn alias(&self) -> Result<String, Error>;
    fn description(&self) -> Result<String, Error>;
}

impl InterfaceExt for Interface {
    fn try_from_luid(luid: u64) -> Result<Interface, Error> {
        let luid = NET_LUID_LH { Value: luid };
        let mut index = 0;
        unsafe { ConvertInterfaceLuidToIndex(&luid, &mut index)? };
        Ok(Self::from_index_unchecked(index))
    }

    fn try_from_guid(guid: u128) -> Result<Interface, Error> {
        let mut luid = NET_LUID_LH::default();
        unsafe { ConvertInterfaceGuidToLuid(&GUID::from_u128(guid), &mut luid)? };
        Self::try_from_luid(unsafe { luid.Value })
    }

    fn try_from_alias(alias: &str) -> Result<Interface, Error> {
        let mut luid = NET_LUID_LH::default();
        let alias = HSTRING::from(alias);
        let code = unsafe { ConvertInterfaceNameToLuidW(&alias, &mut luid) }.map_err(HRESULT::from);
        match code {
            Ok(_) => Self::try_from_luid(unsafe { luid.Value }),
            Err(ERROR_INVALID_NAME) => Err(Error::InterfaceNotFound),
            Err(ERROR_INVALID_PARAMETER) => Err(Error::InvalidParameter),
            Err(e) => Err(WinError::from(e).into()),
        }
    }

    fn luid(&self) -> Result<u64, Error> {
        let mut luid = NET_LUID_LH::default();

        let code = unsafe { ConvertInterfaceIndexToLuid(self.index()?, &mut luid) };
        match code.map_err(HRESULT::from) {
            Ok(_) => Ok(unsafe { luid.Value }),
            Err(ERROR_FILE_NOT_FOUND) => Err(Error::InterfaceNotFound),
            Err(e) => Err(WinError::from(e).into()),
        }
    }

    fn guid(&self) -> Result<u128, Error> {
        let mut guid = GUID::zeroed();
        let code = unsafe { ConvertInterfaceLuidToGuid(&self.0.net_luid_lh()?, &mut guid) };
        match code.map_err(HRESULT::from) {
            Ok(_) => Ok(guid.into()),
            Err(ERROR_FILE_NOT_FOUND) => Err(Error::InterfaceNotFound),
            Err(e) => Err(WinError::from(e).into()),
        }
    }

    fn index(&self) -> Result<u32, Error> {
        Ok(self.0.index)
    }

    fn alias(&self) -> Result<String, Error> {
        let mut alias_buf = vec![0u16; (IF_MAX_STRING_SIZE + 1) as _];
        let code = unsafe { ConvertInterfaceLuidToAlias(&self.0.net_luid_lh()?, &mut alias_buf) };

        match code.map_err(HRESULT::from) {
            Ok(_) => Ok(U16CString::from_vec_truncate(alias_buf).to_string()?),
            Err(ERROR_FILE_NOT_FOUND) => Err(Error::InterfaceNotFound),
            Err(e) => Err(WinError::from(e).into()),
        }
    }

    fn description(&self) -> Result<String, Error> {
        Ok(
            U16CString::from_vec_truncate(self.0.mib_if_row2()?.Description.to_vec())
                .to_string()?,
        )
    }
}

impl InterfaceHandleCommonT for InterfaceHandle {
    fn addresses(&self) -> Result<Vec<IpNet>, Error> {
        let mut table = std::ptr::null_mut();

        unsafe { GetUnicastIpAddressTable(AF_UNSPEC.0 as _, &mut table)? };
        let table = scopeguard::guard(table, |table| {
            if !table.is_null() {
                unsafe {
                    FreeMibTable(table as _);
                }
            }
        });

        let rows = unsafe {
            std::slice::from_raw_parts((*(*table)).Table.as_ptr(), (*(*table)).NumEntries as _)
        };

        let address_set: Result<HashSet<IpNet>, Error> = rows
            .iter()
            .filter(|row| row.InterfaceIndex == self.index)
            .map(|row| {
                IpNet::new(convert_sockaddr(row.Address).ip(), row.OnLinkPrefixLength)
                    .map_err(|_| Error::UnexpectedMetadata)
            })
            .collect();

        Ok(address_set?.into_iter().collect())
    }

    fn add_address(&self, network: IpNet) -> Result<(), Error> {
        let mut row = MIB_UNICASTIPADDRESS_ROW::default();
        unsafe { InitializeUnicastIpAddressEntry(&mut row as _) };

        row.InterfaceIndex = self.index;
        row.Address = SocketAddr::new(network.addr(), 0).into();
        row.OnLinkPrefixLength = network.prefix_len();

        unsafe { Ok(CreateUnicastIpAddressEntry(&row)?) }
    }

    fn remove_address(&self, network: IpNet) -> Result<(), Error> {
        let mut row = MIB_UNICASTIPADDRESS_ROW::default();
        unsafe { InitializeUnicastIpAddressEntry(&mut row as _) };

        row.InterfaceIndex = self.index;
        row.Address = SocketAddr::new(network.addr(), 0).into();
        row.OnLinkPrefixLength = network.prefix_len();

        unsafe { Ok(DeleteUnicastIpAddressEntry(&row)?) }
    }

    fn mtu(&self) -> Result<u32, Error> {
        Ok(self.mib_if_row2()?.Mtu)
    }

    fn set_mtu(&self, mtu: u32) -> Result<(), Error> {
        for family in [AF_INET, AF_INET6] {
            let mut row = MIB_IPINTERFACE_ROW {
                Family: family.0 as _,
                InterfaceIndex: self.index,
                ..Default::default()
            };

            let code = unsafe { GetIpInterfaceEntry(&mut row) };
            match code.map_err(HRESULT::from) {
                Ok(_) => Ok(()),
                Err(ERROR_FILE_NOT_FOUND) => Err(Error::InterfaceNotFound),
                Err(ERROR_NOT_FOUND) => {
                    warn!("Interface not found with family: {:?}", family);
                    continue;
                }
                Err(e) => Err(WinError::from(e).into()),
            }?;

            row.NlMtu = mtu;

            let code = unsafe { SetIpInterfaceEntry(&mut row) };
            match code.map_err(HRESULT::from) {
                Ok(_) => Ok(()),
                Err(ERROR_FILE_NOT_FOUND) => Err(Error::InterfaceNotFound),
                Err(ERROR_NOT_FOUND) => {
                    warn!("Interface not found with family: {:?}", family);
                    continue;
                }
                Err(ERROR_ACCESS_DENIED) => {
                    Err(io::Error::from(ErrorKind::PermissionDenied).into())
                }
                Err(e) => Err(WinError::from(e).into()),
            }?;
        }
        Ok(())
    }

    fn name(&self) -> Result<String, Error> {
        let mut name_buf = vec![0u16; (IF_MAX_STRING_SIZE + 1) as _];
        let code = unsafe { ConvertInterfaceLuidToNameW(&self.net_luid_lh()?, &mut name_buf) };

        match code.map_err(HRESULT::from) {
            Ok(_) => Ok(U16CString::from_vec_truncate(name_buf).to_string()?),
            Err(ERROR_FILE_NOT_FOUND) => Err(Error::InterfaceNotFound),
            Err(e) => Err(WinError::from(e).into()),
        }
    }

    fn try_from_name(name: &str) -> Result<Interface, Error> {
        let mut luid = NET_LUID_LH::default();
        let name = HSTRING::from(name);
        let code = unsafe { ConvertInterfaceNameToLuidW(&name, &mut luid) };
        match code.map_err(HRESULT::from) {
            Ok(_) => Interface::try_from_luid(unsafe { luid.Value }),
            Err(ERROR_FILE_NOT_FOUND) => Err(Error::InterfaceNotFound),
            Err(e) => Err(WinError::from(e).into()),
        }
    }

    fn index(&self) -> Result<u32, Error> {
        Ok(self.index)
    }

    fn try_from_index(index: u32) -> Result<Interface, Error> {
        let mut luid = NET_LUID_LH::default();
        let code = unsafe { ConvertInterfaceIndexToLuid(index, &mut luid) };
        match code.map_err(HRESULT::from) {
            Ok(_) => Ok(Interface::from_index_unchecked(index)),
            Err(ERROR_FILE_NOT_FOUND) => Err(Error::InterfaceNotFound),
            Err(e) => Err(WinError::from(e).into()),
        }
    }
}
