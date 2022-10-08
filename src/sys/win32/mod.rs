use windows::Win32::Networking::WinSock::AF_UNSPEC;

use crate::sys::mib_table::MibTable;
use crate::{Error, Interface};
pub use handle::InterfaceExt;

mod handle;
pub(crate) mod mib_table;

pub(crate) fn list_interfaces() -> Result<Vec<Interface>, Error> {
    MibTable::GetIpInterfaceTable(&AF_UNSPEC)?
        .as_slice()
        .iter()
        .map(|row| Interface::try_from_index(row.InterfaceIndex))
        .collect()
}
