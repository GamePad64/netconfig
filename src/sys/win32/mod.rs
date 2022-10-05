use windows::Win32::NetworkManagement::IpHelper::{FreeMibTable, GetIpInterfaceTable};
use windows::Win32::Networking::WinSock::AF_UNSPEC;

use crate::{Error, Interface};
pub use handle::InterfaceExt;

mod handle;

pub(crate) fn list_interfaces() -> Result<Vec<Interface>, Error> {
    let mut table = std::ptr::null_mut();

    unsafe { GetIpInterfaceTable(AF_UNSPEC.0 as _, &mut table)? };
    let table = scopeguard::guard(table, |table| {
        if !table.is_null() {
            unsafe {
                FreeMibTable(table as _);
            }
        }
    });

    let rows = unsafe {
        let table = table.as_ref().unwrap();
        std::slice::from_raw_parts(table.Table.as_ptr(), table.NumEntries as _)
    };

    rows.iter()
        .map(|row| Interface::try_from_index(row.InterfaceIndex))
        .collect()
}
