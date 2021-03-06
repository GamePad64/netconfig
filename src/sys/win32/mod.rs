use windows::Win32::NetworkManagement::IpHelper::{FreeMibTable, GetIpInterfaceTable};
use windows::Win32::Networking::WinSock::AF_UNSPEC;

use crate::sys::InterfaceHandle;
use crate::InterfaceHandleCommonT;
pub use handle::InterfaceHandleExt;
pub(crate) use metadata::Metadata;
pub use metadata::MetadataExt;

mod handle;
mod metadata;

pub(crate) fn list_interfaces() -> Vec<crate::InterfaceHandle> {
    let mut table = std::ptr::null_mut();

    let result = unsafe { GetIpInterfaceTable(AF_UNSPEC.0 as _, &mut table) };
    let table = scopeguard::guard(table, |table| {
        if !table.is_null() {
            unsafe {
                FreeMibTable(table as _);
            }
        }
    });

    unsafe {
        if result.is_ok() {
            let mut result = Vec::with_capacity((*(*table)).NumEntries as _);
            for i in 0..(*(*table)).NumEntries as _ {
                let row = &(*(*table)).Table.get_unchecked(i);
                let handle = InterfaceHandle::try_from_index(row.InterfaceIndex).unwrap();
                result.push(handle);
            }
            result
        } else {
            vec![]
        }
    }
}
