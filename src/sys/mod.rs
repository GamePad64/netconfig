#[derive(Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub(crate) struct InterfaceHandle {
    pub(crate) index: u32,
}

cfg_if::cfg_if! {
    if #[cfg(target_os = "windows")] {
        mod win32;
        pub(crate) use win32::*;
        pub use win32::InterfaceExt;
    } else if #[cfg(target_os = "linux")] {
        mod linux;
        pub(crate) use linux::*;
        pub use linux::{InterfaceHandleExt, MetadataExt};
    } else if #[cfg(target_os = "macos")] {
        mod darwin;
        pub(crate) use darwin::*;
        pub use darwin::{InterfaceHandleExt, MetadataExt};
    }
}

cfg_if::cfg_if! {
    if #[cfg(unix)] {
        pub mod posix;
    }
}
