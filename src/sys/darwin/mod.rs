pub use handle::InterfaceHandleExt;
pub(crate) use metadata::Metadata;
pub use metadata::MetadataExt;
use std::collections::HashSet;
mod handle;
mod metadata;
pub(crate) mod scinterface;

pub(crate) fn list_interfaces() -> Vec<crate::InterfaceHandle> {
    let names: Vec<String> = nix::ifaddrs::getifaddrs()
        .unwrap()
        .map(|addr| addr.interface_name)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    let mut result = vec![];
    for name in names {
        result.push(crate::InterfaceHandle::try_from_name(&*name).unwrap())
    }

    result
}
