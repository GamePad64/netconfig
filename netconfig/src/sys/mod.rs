#[derive(Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub(crate) struct InterfaceHandle {
    pub(crate) index: u32,
}

cfg_if::cfg_if! {
    if #[cfg(target_os = "windows")] {
        mod win32;
        pub(crate) use win32::*;
        pub use win32::{InterfaceHandleExt, MetadataExt};
    } else if #[cfg(target_os = "linux")] {
        mod linux;
        pub(crate) use linux::*;
        pub use linux::{InterfaceHandleExt, MetadataExt};
    }
}
