use crate::MetadataCommonT;

pub trait MetadataExt {
    fn luid(&self) -> u64;
    fn guid(&self) -> u128;
    fn index(&self) -> u32;
    fn alias(&self) -> String;
    fn description(&self) -> String;
}

impl MetadataExt for crate::Metadata {
    fn luid(&self) -> u64 {
        self.0.luid
    }

    fn guid(&self) -> u128 {
        self.0.guid
    }

    fn index(&self) -> u32 {
        self.0.index
    }

    fn alias(&self) -> String {
        self.0.alias.clone()
    }

    fn description(&self) -> String {
        self.0.description.clone()
    }
}

#[derive(Default)]
pub(crate) struct Metadata {
    pub(crate) handle: crate::InterfaceHandle,

    pub(crate) luid: u64,
    pub(crate) guid: u128,
    pub(crate) index: u32,
    pub(crate) mtu: u32,
    pub(crate) name: String,
    pub(crate) alias: String,
    pub(crate) description: String,
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
