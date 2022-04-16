use std::fs;

pub(crate) use handle::InterfaceHandle;
pub use handle::InterfaceHandleExt;
pub(crate) use metadata::Metadata;

mod handle;
mod ifreq;
mod metadata;

pub(crate) fn list_interfaces() -> Vec<crate::InterfaceHandle> {
    let mut result = vec![];

    for path in fs::read_dir("/sys/class/net").expect("Path is not available") {
        let handle = InterfaceHandle::from_name(path.unwrap().file_name().to_str().unwrap());
        result.push(crate::InterfaceHandle(handle));
    }
    result
}
