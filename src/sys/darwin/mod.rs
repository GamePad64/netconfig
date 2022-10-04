pub use handle::InterfaceHandleExt;
pub(crate) use metadata::Metadata;
pub use metadata::MetadataExt;
use std::collections::HashSet;
mod handle;
mod metadata;
pub(crate) mod scinterface;

pub(crate) fn list_interfaces() -> Vec<crate::InterfaceHandle> {
    nix::net::if_::if_nameindex()
        .unwrap()
        .iter()
        .map(|a| crate::InterfaceHandle::from_index_unchecked(a.index()))
        .collect()
}
