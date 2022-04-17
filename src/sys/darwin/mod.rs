use core_foundation::impl_TCFType;
pub use handle::InterfaceHandleExt;
pub(crate) use metadata::Metadata;
pub use metadata::MetadataExt;
use std::collections::HashSet;

mod handle;
mod metadata;
pub(crate) mod scinterface;

use core_foundation::{array::CFArray, base::TCFType, string::CFString};
use system_configuration_sys::network_configuration::{
    SCNetworkInterfaceCopyAll, SCNetworkInterfaceGetBSDName,
    SCNetworkInterfaceGetLocalizedDisplayName, SCNetworkInterfaceGetTypeID, SCNetworkInterfaceRef,
};

core_foundation::declare_TCFType!(SCNetworkInterface, SCNetworkInterfaceRef);
core_foundation::impl_TCFType!(
    SCNetworkInterface,
    SCNetworkInterfaceRef,
    SCNetworkInterfaceGetTypeID
);

impl SCNetworkInterface {
    fn name(&self) -> Option<String> {
        let ptr = unsafe { SCNetworkInterfaceGetBSDName(self.0) };
        if ptr.is_null() {
            None
        } else {
            unsafe { Some(CFString::wrap_under_get_rule(ptr).to_string()) }
        }
    }

    fn displayname(&self) -> Option<String> {
        let ptr = unsafe { SCNetworkInterfaceGetLocalizedDisplayName(self.0) };
        if ptr.is_null() {
            None
        } else {
            unsafe { Some(CFString::wrap_under_get_rule(ptr).to_string()) }
        }
    }
}

pub(crate) fn list_interfaces() -> Vec<crate::InterfaceHandle> {
    let interfaces = unsafe {
        CFArray::<SCNetworkInterface>::wrap_under_create_rule(SCNetworkInterfaceCopyAll())
    };
    for interface in interfaces.iter() {
        println!("{:?}", interface.name());
        println!("{:?}", interface.displayname());
    }

    let names: Vec<String> = nix::ifaddrs::getifaddrs()
        .unwrap()
        .map(|addr| addr.interface_name)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    println!("{names:?}");

    let mut result = vec![];
    for name in names {
        result.push(crate::InterfaceHandle::try_from_name(&*name).unwrap())
    }

    println!("{result:?}");

    result
}
