#![allow(non_snake_case)]
use crate::Error;
use std::marker::PhantomData;
use windows::Win32::NetworkManagement::IpHelper::{
    FreeMibTable, GetIpInterfaceTable, GetUnicastIpAddressTable, MIB_IPINTERFACE_ROW,
    MIB_IPINTERFACE_TABLE, MIB_UNICASTIPADDRESS_ROW, MIB_UNICASTIPADDRESS_TABLE,
};
use windows::Win32::Networking::WinSock::ADDRESS_FAMILY;

pub struct MibTable<'a, T, R> {
    table: *mut T,
    phantom: PhantomData<&'a R>,
}

impl<'a, T, R> Default for MibTable<'a, T, R> {
    fn default() -> Self {
        Self {
            table: std::ptr::null_mut(),
            phantom: Default::default(),
        }
    }
}

impl<'a> MibTable<'a, MIB_UNICASTIPADDRESS_TABLE, MIB_UNICASTIPADDRESS_ROW> {
    pub fn GetUnicastIpAddressTable(family: &ADDRESS_FAMILY) -> Result<Self, Error> {
        let mut result = Self::default();
        unsafe { GetUnicastIpAddressTable(family.0 as _, &mut result.table)? }
        Ok(result)
    }

    pub fn as_slice(&self) -> &'a [MIB_UNICASTIPADDRESS_ROW] {
        unsafe {
            let table = self.table.as_ref().unwrap();
            std::slice::from_raw_parts(table.Table.as_ptr(), table.NumEntries as _)
        }
    }
}

impl<'a> MibTable<'a, MIB_IPINTERFACE_TABLE, MIB_IPINTERFACE_ROW> {
    pub fn GetIpInterfaceTable(family: &ADDRESS_FAMILY) -> Result<Self, Error> {
        let mut result = Self::default();
        unsafe { GetIpInterfaceTable(family.0 as _, &mut result.table)? }
        Ok(result)
    }

    pub fn as_slice(&self) -> &'a [MIB_IPINTERFACE_ROW] {
        unsafe {
            let table = self.table.as_ref().unwrap();
            std::slice::from_raw_parts(table.Table.as_ptr(), table.NumEntries as _)
        }
    }
}

impl<'a, T, R> Drop for MibTable<'a, T, R> {
    fn drop(&mut self) {
        if !self.table.is_null() {
            unsafe {
                FreeMibTable(self.table as _);
            }
        }
    }
}
