use crate::MetadataCommonT;

#[derive(Default)]
pub(crate) struct Metadata {
    pub(crate) handle: crate::InterfaceHandle,

    pub(crate) name: String,
    pub(crate) mtu: u32,
    pub(crate) index: u32,
    pub(crate) alias: String,
}

impl MetadataCommonT for Metadata {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn handle(&self) -> crate::InterfaceHandle {
        self.handle
    }

    fn mtu(&self) -> u32 {
        self.mtu
    }

    fn index(&self) -> u32 {
        self.index
    }
}

pub trait MetadataExt {
    fn alias(&self) -> String;
}
impl MetadataExt for crate::Metadata {
    fn alias(&self) -> String {
        self.0.alias.clone()
    }
}
