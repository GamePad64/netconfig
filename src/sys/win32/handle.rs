use super::Metadata;
use crate::sys::InterfaceHandle;
use crate::{Error, InterfaceHandleCommonT};
use ipnet::IpNet;
use log::warn;
use std::collections::HashSet;
use std::io;
use std::io::ErrorKind;
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};
use widestring::{U16CStr, U16CString};
use windows::core::{Error as WinError, GUID, HRESULT};
use windows::Win32::NetworkManagement::IpHelper::{
    ConvertInterfaceGuidToLuid, ConvertInterfaceIndexToLuid, ConvertInterfaceLuidToIndex,
    ConvertInterfaceLuidToNameW, ConvertInterfaceNameToLuidW, CreateUnicastIpAddressEntry,
    DeleteUnicastIpAddressEntry, FreeMibTable, GetIfEntry2, GetIpInterfaceEntry,
    GetUnicastIpAddressTable, InitializeUnicastIpAddressEntry, SetIpInterfaceEntry, MIB_IF_ROW2,
    MIB_IPINTERFACE_ROW, MIB_UNICASTIPADDRESS_ROW,
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
    fn luid(&self) -> Result<NET_LUID_LH, Error> {
        let mut luid = NET_LUID_LH::default();

        let code = unsafe { ConvertInterfaceIndexToLuid(self.index, &mut luid) };
        match code.map_err(HRESULT::from) {
            Ok(_) => Ok(luid),
            Err(ERROR_FILE_NOT_FOUND) => Err(Error::InterfaceNotFound),
            Err(e) => Err(WinError::from(e).into()),
        }
    }
}

pub trait InterfaceHandleExt {
    fn try_from_luid(luid: NET_LUID_LH) -> Result<crate::InterfaceHandle, Error>;
    fn try_from_guid(guid: GUID) -> Result<crate::InterfaceHandle, Error>;
    fn try_from_alias(alias: &str) -> Result<crate::InterfaceHandle, Error>;
}

impl InterfaceHandleExt for InterfaceHandle {
    fn try_from_luid(luid: NET_LUID_LH) -> Result<crate::InterfaceHandle, Error> {
        let mut index = 0;
        unsafe { ConvertInterfaceLuidToIndex(&luid, &mut index)? };
        Ok(crate::InterfaceHandle::from_index_unchecked(index))
    }

    fn try_from_guid(guid: GUID) -> Result<crate::InterfaceHandle, Error> {
        let mut luid = NET_LUID_LH::default();
        unsafe { ConvertInterfaceGuidToLuid(&guid, &mut luid)? };
        Self::try_from_luid(luid)
    }

    fn try_from_alias(alias: &str) -> Result<crate::InterfaceHandle, Error> {
        let mut luid = NET_LUID_LH::default();
        let code = unsafe { ConvertInterfaceNameToLuidW(alias, &mut luid) }.map_err(HRESULT::from);
        match code {
            Ok(_) => Self::try_from_luid(luid),
            Err(ERROR_INVALID_NAME) => Err(Error::InterfaceNotFound),
            Err(ERROR_INVALID_PARAMETER) => Err(Error::InvalidParameter),
            Err(e) => Err(WinError::from(e).into()),
        }
    }
}

impl InterfaceHandleExt for crate::InterfaceHandle {
    fn try_from_luid(luid: NET_LUID_LH) -> Result<crate::InterfaceHandle, Error> {
        InterfaceHandle::try_from_luid(luid)
    }

    fn try_from_guid(guid: GUID) -> Result<crate::InterfaceHandle, Error> {
        InterfaceHandle::try_from_guid(guid)
    }

    fn try_from_alias(alias: &str) -> Result<crate::InterfaceHandle, Error> {
        InterfaceHandle::try_from_alias(alias)
    }
}

impl InterfaceHandleCommonT for InterfaceHandle {
    fn metadata(&self) -> Result<crate::Metadata, Error> {
        let mut result = Metadata {
            handle: crate::InterfaceHandle(*self),
            index: self.index,
            ..Default::default()
        };

        // MIB_IF_ROW2 data
        {
            let mut row = MIB_IF_ROW2 {
                InterfaceIndex: self.index,
                ..Default::default()
            };
            unsafe {
                GetIfEntry2(&mut row).map_err(|_| Error::InterfaceNotFound)?;
            }
            result.description = U16CStr::from_slice_truncate(&row.Description)
                .map_err(|_| Error::UnexpectedMetadata)?
                .to_string()
                .map_err(|_| Error::UnexpectedMetadata)?;
            result.alias = U16CStr::from_slice_truncate(&row.Alias)
                .map_err(|_| Error::UnexpectedMetadata)?
                .to_string()
                .map_err(|_| Error::UnexpectedMetadata)?;
            result.guid = row.InterfaceGuid;
            result.index = row.InterfaceIndex;
            result.mtu = row.Mtu;
            result.luid = row.InterfaceLuid;
        }

        // Interface name
        {
            let mut name_buf = vec![0u16; (IF_MAX_STRING_SIZE + 1) as _];
            unsafe { ConvertInterfaceLuidToNameW(&self.luid()?, &mut name_buf) }
                .map_err(|_| Error::UnexpectedMetadata)?;

            result.name = U16CString::from_vec_truncate(name_buf)
                .to_string()
                .map_err(|_| Error::UnexpectedMetadata)?;
        }

        Ok(crate::Metadata(result))
    }

    fn add_ip(&self, network: IpNet) {
        let mut row = MIB_UNICASTIPADDRESS_ROW::default();
        unsafe { InitializeUnicastIpAddressEntry(&mut row as _) };

        row.InterfaceIndex = self.index;
        row.Address = SocketAddr::new(network.addr(), 0).into();
        row.OnLinkPrefixLength = network.prefix_len();

        unsafe {
            CreateUnicastIpAddressEntry(&row).unwrap();
        }
    }

    fn remove_ip(&self, network: IpNet) {
        let mut row = MIB_UNICASTIPADDRESS_ROW::default();
        unsafe { InitializeUnicastIpAddressEntry(&mut row as _) };

        row.InterfaceIndex = self.index;
        row.Address = SocketAddr::new(network.addr(), 0).into();
        row.OnLinkPrefixLength = network.prefix_len();

        unsafe {
            DeleteUnicastIpAddressEntry(&row).unwrap();
        }
    }

    fn get_addresses(&self) -> Result<Vec<IpNet>, Error> {
        let mut table = std::ptr::null_mut();

        unsafe { GetUnicastIpAddressTable(AF_UNSPEC.0 as _, &mut table) }.map_err(Error::from)?;
        let table = scopeguard::guard(table, |table| {
            if !table.is_null() {
                unsafe {
                    FreeMibTable(table as _);
                }
            }
        });

        let mut addresses_set = HashSet::new();

        unsafe {
            for i in 0..(*(*table)).NumEntries as _ {
                let row = &(*(*table)).Table.get_unchecked(i);
                let sockaddr = convert_sockaddr(row.Address);

                if row.InterfaceIndex != self.index {
                    continue;
                }

                addresses_set.insert(
                    IpNet::new(sockaddr.ip(), row.OnLinkPrefixLength)
                        .map_err(|_| Error::UnexpectedMetadata)?,
                );
            }
        }

        Ok(addresses_set.iter().cloned().collect())
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

    fn try_from_index(index: u32) -> Result<crate::InterfaceHandle, Error> {
        let mut luid = NET_LUID_LH::default();
        let code = unsafe { ConvertInterfaceIndexToLuid(index, &mut luid) };
        match code.map_err(HRESULT::from) {
            Ok(_) => Ok(crate::InterfaceHandle::from_index_unchecked(index)),
            Err(ERROR_FILE_NOT_FOUND) => Err(Error::InterfaceNotFound),
            Err(e) => Err(WinError::from(e).into()),
        }
    }

    fn try_from_name(name: &str) -> Result<crate::InterfaceHandle, Error> {
        let mut luid = NET_LUID_LH::default();
        let code = unsafe { ConvertInterfaceNameToLuidW(name, &mut luid) };
        match code.map_err(HRESULT::from) {
            Ok(_) => InterfaceHandle::try_from_luid(luid),
            Err(ERROR_FILE_NOT_FOUND) => Err(Error::InterfaceNotFound),
            Err(e) => Err(WinError::from(e).into()),
        }
    }
}
