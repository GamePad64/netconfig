use crate::Interface;

#[derive(Debug, Clone)]
pub(crate) struct InterfaceHandle {
    pub(crate) index: u32,
}

impl InterfaceHandle {
    #[allow(unused)]
    fn interface(&self) -> Interface {
        Interface(self.clone())
    }
}

cfg_if::cfg_if! {
    if #[cfg(target_os = "windows")] {
        mod win32;
        pub(crate) use win32::*;
        pub use win32::InterfaceExt;
    } else if #[cfg(target_os = "linux")] {
        mod linux;
        #[allow(unused)]
        pub(crate) use linux::*;
        pub use linux::InterfaceExt;
    } else if #[cfg(target_os = "macos")] {
        mod darwin;
        #[allow(unused)]
        pub(crate) use darwin::*;
        pub use darwin::{InterfaceExt};
    }
}

cfg_if::cfg_if! {
    if #[cfg(unix)] {
        pub mod posix;
        pub(crate) use posix::*;
    }
}
