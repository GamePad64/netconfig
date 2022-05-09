use super::{win_convert, Metadata};
use crate::sys::InterfaceHandle;
use crate::{Error, InterfaceHandleCommonT};
use ipnet::IpNet;
use log::warn;
use std::collections::HashSet;
use std::io;
use std::io::ErrorKind;
use std::net::SocketAddr;
use widestring::{U16CStr, U16CString};
use windows::core::{GUID, PCWSTR};
use windows::Win32::Foundation::{
    ERROR_ACCESS_DENIED, ERROR_FILE_NOT_FOUND, ERROR_INVALID_NAME, ERROR_INVALID_PARAMETER,
    ERROR_NOT_FOUND,
};
use windows::Win32::NetworkManagement::IpHelper::{
    ConvertInterfaceGuidToLuid, ConvertInterfaceIndexToLuid, ConvertInterfaceLuidToIndex,
    ConvertInterfaceLuidToNameW, ConvertInterfaceNameToLuidW, CreateUnicastIpAddressEntry,
    DeleteUnicastIpAddressEntry, FreeMibTable, GetIfEntry2, GetIpInterfaceEntry,
    GetUnicastIpAddressTable, InitializeUnicastIpAddressEntry, SetIpInterfaceEntry, MIB_IF_ROW2,
    MIB_IPINTERFACE_ROW, MIB_UNICASTIPADDRESS_ROW, NET_LUID_LH,
};
use windows::Win32::NetworkManagement::Ndis::NDIS_IF_MAX_STRING_SIZE;
use windows::Win32::Networking::WinSock::{AF_INET, AF_INET6, AF_UNSPEC};

impl InterfaceHandle {
    fn luid(&self) -> Result<NET_LUID_LH, Error> {
        let mut luid = NET_LUID_LH::default();
        let code = unsafe { ConvertInterfaceIndexToLuid(self.index, &mut luid) }
            .map_err(|e| e.win32_error().unwrap());
        match code {
            Ok(_) => Ok(luid),
            Err(ERROR_FILE_NOT_FOUND) => Err(Error::InterfaceNotFound),
            _ => Err(Error::InternalError),
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
        let code = unsafe { ConvertInterfaceLuidToIndex(&luid, &mut index) }
            .map_err(|e| e.win32_error().unwrap());
        match code {
            Ok(_) => Ok(crate::InterfaceHandle(InterfaceHandle { index })),
            _ => Err(Error::InternalError),
        }
    }

    fn try_from_guid(guid: GUID) -> Result<crate::InterfaceHandle, Error> {
        let mut luid = NET_LUID_LH::default();
        let code = unsafe { ConvertInterfaceGuidToLuid(&guid, &mut luid) }
            .map_err(|e| e.win32_error().unwrap());
        match code {
            Ok(_) => Self::try_from_luid(luid),
            _ => Err(Error::InternalError),
        }
    }

    fn try_from_alias(alias: &str) -> Result<crate::InterfaceHandle, Error> {
        let walias = U16CString::from_str(alias).map_err(|_| Error::InvalidParameter)?;

        let mut luid = NET_LUID_LH::default();
        let code = unsafe { ConvertInterfaceNameToLuidW(PCWSTR(walias.as_ptr() as _), &mut luid) }
            .map_err(|e| e.win32_error().unwrap());
        match code {
            Ok(_) => Ok(InterfaceHandle::try_from_luid(luid)?),
            Err(ERROR_INVALID_NAME) => Err(Error::InterfaceNotFound),
            Err(ERROR_INVALID_PARAMETER) => Err(Error::InvalidParameter),
            _ => Err(Error::InternalError),
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
            let mut name_buf = vec![0u16; (NDIS_IF_MAX_STRING_SIZE + 1) as _];
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
        row.Address = win_convert::xSocketAddr(SocketAddr::new(network.addr(), 0)).into();
        row.OnLinkPrefixLength = network.prefix_len();

        unsafe {
            CreateUnicastIpAddressEntry(&row).unwrap();
        }
    }

    fn remove_ip(&self, network: IpNet) {
        let mut row = MIB_UNICASTIPADDRESS_ROW::default();
        unsafe { InitializeUnicastIpAddressEntry(&mut row as _) };

        row.InterfaceIndex = self.index;
        row.Address = win_convert::xSocketAddr(SocketAddr::new(network.addr(), 0)).into();
        row.OnLinkPrefixLength = network.prefix_len();

        unsafe {
            DeleteUnicastIpAddressEntry(&row).unwrap();
        }
    }

    fn get_addresses(&self) -> Result<Vec<IpNet>, Error> {
        let mut table = std::ptr::null_mut();

        unsafe { GetUnicastIpAddressTable(AF_UNSPEC.0 as _, &mut table) }
            .map_err(|_| Error::InternalError)?;
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
                let sockaddr = win_convert::xSocketAddr::from(row.Address);

                if row.InterfaceIndex != self.index {
                    continue;
                }

                addresses_set.insert(
                    IpNet::new(sockaddr.0.ip(), row.OnLinkPrefixLength)
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

            match unsafe { GetIpInterfaceEntry(&mut row).map_err(|e| e.win32_error().unwrap()) } {
                Ok(_) => Ok(()),
                Err(ERROR_FILE_NOT_FOUND) => Err(Error::InterfaceNotFound),
                Err(ERROR_NOT_FOUND) => {
                    warn!("Interface not found with family: {:?}", family);
                    continue;
                }
                _ => Err(Error::InternalError),
            }?;

            row.NlMtu = mtu;

            match unsafe { SetIpInterfaceEntry(&mut row).map_err(|e| e.win32_error().unwrap()) } {
                Ok(_) => Ok(()),
                Err(ERROR_FILE_NOT_FOUND) => Err(Error::InterfaceNotFound),
                Err(ERROR_NOT_FOUND) => {
                    warn!("Interface not found with family: {:?}", family);
                    continue;
                }
                Err(ERROR_ACCESS_DENIED) => {
                    Err(io::Error::from(ErrorKind::PermissionDenied).into())
                }
                Err(_) => Err(Error::InternalError),
            }?;
        }
        Ok(())
    }

    fn try_from_index(index: u32) -> Result<crate::InterfaceHandle, Error> {
        let mut luid = NET_LUID_LH::default();
        let code = unsafe { ConvertInterfaceIndexToLuid(index, &mut luid) }
            .map_err(|e| e.win32_error().unwrap());
        match code {
            Ok(_) => Ok(crate::InterfaceHandle::from_index_unchecked(index)),
            Err(ERROR_FILE_NOT_FOUND) => Err(Error::InterfaceNotFound),
            _ => Err(Error::InternalError),
        }
    }

    fn try_from_name(name: &str) -> Result<crate::InterfaceHandle, Error> {
        let wname = U16CString::from_str(name).map_err(|_| Error::InvalidParameter)?;

        let mut luid = NET_LUID_LH::default();
        let code = unsafe { ConvertInterfaceNameToLuidW(PCWSTR(wname.as_ptr() as _), &mut luid) }
            .map_err(|e| e.win32_error().unwrap());
        match code {
            Ok(_) => Ok(InterfaceHandle::try_from_luid(luid)?),
            Err(ERROR_FILE_NOT_FOUND) => Err(Error::InterfaceNotFound),
            _ => Err(Error::InternalError),
        }
    }
}
