use core_foundation::impl_TCFType;
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
    fn get_by_name(name: &str) -> Option<SCNetworkInterface> {
        let interfaces = unsafe {
            CFArray::<SCNetworkInterface>::wrap_under_create_rule(SCNetworkInterfaceCopyAll())
        };

        for interface in interfaces.iter() {
            if let Some(ifname) = interface.name() {
                if ifname == name {
                    return Some(interface.clone());
                }
            }
        }
        None
    }

    pub(crate) fn get_displayname(name: &str) -> Option<String> {
        Self::get_by_name(name)?.displayname()
    }

    pub(crate) fn name(&self) -> Option<String> {
        let ptr = unsafe { SCNetworkInterfaceGetBSDName(self.0) };
        if ptr.is_null() {
            None
        } else {
            unsafe { Some(CFString::wrap_under_get_rule(ptr).to_string()) }
        }
    }

    pub(crate) fn displayname(&self) -> Option<String> {
        let ptr = unsafe { SCNetworkInterfaceGetLocalizedDisplayName(self.0) };
        if ptr.is_null() {
            None
        } else {
            unsafe { Some(CFString::wrap_under_get_rule(ptr).to_string()) }
        }
    }
}
