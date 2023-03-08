use std::collections::{BTreeMap, HashMap};

use lazy_static::lazy_static;

use crate::message_type::message_family::{
    basic_message::BasicMessageV1_0,
    traits::{ResolveMajorVersion, ResolveMinorVersion, ResolveMsgKind},
};

type RegistryMap = HashMap<&'static str, HashMap<u8, BTreeMap<u8, Vec<&'static str>>>>;

macro_rules! extract_parts {
    ($name:ty) => {
        (
            <<$name as ResolveMsgKind>::Parent as ResolveMinorVersion>::Parent::FAMILY,
            <$name as ResolveMsgKind>::Parent::MAJOR,
            <$name as ResolveMsgKind>::MINOR,
        );
    };
}

fn map_insert(map: &mut RegistryMap, parts: (&'static str, u8, u8)) {
    let (family, major, minor) = parts;
    map.entry(family)
        .or_insert(HashMap::new())
        .entry(major)
        .or_insert(BTreeMap::new());
}

lazy_static! {
    static ref PROTOCOL_REGISTRY: RegistryMap = {
        let mut m = HashMap::new();
        let (family, major, minor) = extract_parts!(BasicMessageV1_0);
        m
    };
}
