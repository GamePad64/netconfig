use log::warn;
use std::fs;

pub use handle::InterfaceHandleExt;
pub(crate) use metadata::Metadata;
pub use metadata::MetadataExt;

mod handle;
mod ifreq;
mod metadata;

pub(crate) fn list_interfaces() -> Vec<crate::InterfaceHandle> {
    let mut result = vec![];

    for path in fs::read_dir("/sys/class/net").expect("Path is not available") {
        let handle = crate::InterfaceHandle::try_from_name(
            path.unwrap()
                .file_name()
                .to_str()
                .expect("Interface name is invalid"),
        );
        match handle {
            Ok(handle) => result.push(handle),
            Err(e) => warn!("Error during interface list: {e:?}"),
        }
    }
    result
}
