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

/// An entry in the protocol registry.
#[derive(Debug, Clone)]
pub struct RegistryEntry {
    /// The [`Protocol`] instance corresponding to this entry
    pub protocol: Protocol,
    /// The minor version of in numeric (for easier semver resolution),
    pub minor: u8,
    /// A [`String`] representation of the *pid*
    pub str_pid: String,
    /// A [`Vec<Actor>`] representing the roles available in the protocol.
    pub actors: Vec<Actor>,
}

/// Extracts the necessary parts for constructing a [`RegistryEntry`] from a protocol minor version.
macro_rules! extract_parts {
    ($name:ident) => {
        (
            <<$name as MinorVersion>::Parent as MajorVersion>::Parent::FAMILY,
            <$name as MinorVersion>::Parent::MAJOR,
            <$name as MinorVersion>::MINOR,
            <$name as MinorVersion>::Parent::actors().to_vec(),
            Protocol::from($name),
        )
    };
}

fn map_insert(map: &mut RegistryMap, parts: (&'static str, u8, u8, Vec<Actor>, Protocol)) {
    let (protocol_name, major, minor, actors, protocol) = parts;

    let str_pid = format!("{}/{}/{}.{}", Protocol::DID_COM_ORG_PREFIX, protocol_name, major, minor);
    let entry = RegistryEntry {
        protocol,
        minor,
        str_pid,
        actors,
    };

    map.entry((protocol_name, major)).or_insert(Vec::new()).push(entry);
}

lazy_static! {
    /// The protocol registry, used as a baseline for the protocols and versions
    /// that an agent supports along with semver resolution.
    ///
    /// Keys are comprised of the protocol name and major version while
    /// the values are [`RegistryEntry`] instances.
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

/// Looks into the protocol registry for (in order):
/// * the exact protocol version requested
/// * the maximum minor version of a protocol less than the minor version requested (e.g: requesting 1.7 should yield 1.6).
pub fn get_supported_version(name: &'static str, major: u8, minor: u8) -> Option<u8> {
    PROTOCOL_REGISTRY
        .get(&(name, major))
        .and_then(|v| v.iter().rev().map(|r| r.minor).find(|v| *v <= minor))
}
