use std::collections::HashMap;

use lazy_static::lazy_static;

use crate::message_type::message_protocol::{
    basic_message::BasicMessageV1_0,
    connection::ConnectionV1_0,
    cred_issuance::CredentialIssuanceV1_0,
    discover_features::DiscoverFeaturesV1_0,
    notification::NotificationV1_0,
    out_of_band::OutOfBandV1_1,
    present_proof::PresentProofV1_0,
    report_problem::ReportProblemV1_0,
    revocation::RevocationV2_0,
    traits::{MajorVersion, MinorVersion, ProtocolName},
    trust_ping::TrustPingV1_0,
};

use super::{actor::Actor, Protocol};

type RegistryMap = HashMap<(&'static str, u8), Vec<RegistryEntry>>;

#[derive(Debug, Clone)]
pub struct RegistryEntry {
    pub protocol: Protocol,
    pub minor: u8,
    pub str_pid: String,
    pub actors: Vec<Actor>,
}

macro_rules! extract_parts {
    ($name:ident) => {
        (
            <<$name as MinorVersion>::Parent as MajorVersion>::Parent::FAMILY,
            <$name as MinorVersion>::Parent::MAJOR,
            <$name as MinorVersion>::MINOR,
            <$name as MinorVersion>::Parent::actors().to_vec(),
            MessageFamily::from($name),
        )
    };
}

fn map_insert(map: &mut RegistryMap, parts: (&'static str, u8, u8, Vec<Actor>, Protocol)) {
    let (family, major, minor, actors, protocol) = parts;

    let str_pid = format!("{}/{}/{}.{}", Protocol::DID_COM_ORG_PREFIX, family, major, minor);
    let entry = RegistryEntry {
        protocol,
        minor,
        str_pid,
        actors,
    };

    map.entry((family, major)).or_insert(Vec::new()).push(entry);
}

lazy_static! {
    pub static ref PROTOCOL_REGISTRY: RegistryMap = {
        let mut m = HashMap::new();
        map_insert(&mut m, extract_parts!(BasicMessageV1_0));
        map_insert(&mut m, extract_parts!(ConnectionV1_0));
        map_insert(&mut m, extract_parts!(CredentialIssuanceV1_0));
        map_insert(&mut m, extract_parts!(DiscoverFeaturesV1_0));
        map_insert(&mut m, extract_parts!(NotificationV1_0));
        map_insert(&mut m, extract_parts!(OutOfBandV1_1));
        map_insert(&mut m, extract_parts!(PresentProofV1_0));
        map_insert(&mut m, extract_parts!(ReportProblemV1_0));
        map_insert(&mut m, extract_parts!(RevocationV2_0));
        map_insert(&mut m, extract_parts!(TrustPingV1_0));
        m
    };
}

pub fn get_supported_version(family: &'static str, major: u8, minor: u8) -> Option<u8> {
    PROTOCOL_REGISTRY
        .get(&(family, major))
        .and_then(|v| v.iter().rev().map(|r| r.minor).find(|v| *v <= minor))
}
