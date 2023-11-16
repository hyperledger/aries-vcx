use std::collections::HashMap;

use lazy_static::lazy_static;
use shared::maybe_known::MaybeKnown;

use super::{role::Role, Protocol};
use crate::msg_types::{
    present_proof::PresentProofTypeV2,
    protocols::{
        basic_message::BasicMessageTypeV1,
        connection::ConnectionTypeV1,
        coordinate_mediation::CoordinateMediationTypeV1,
        cred_issuance::{CredentialIssuanceTypeV1, CredentialIssuanceTypeV2},
        discover_features::DiscoverFeaturesTypeV1,
        notification::NotificationTypeV1,
        out_of_band::OutOfBandTypeV1,
        pickup::PickupTypeV2,
        present_proof::PresentProofTypeV1,
        report_problem::ReportProblemTypeV1,
        revocation::RevocationTypeV2,
        routing::RoutingTypeV1,
        signature::SignatureTypeV1,
        trust_ping::TrustPingTypeV1,
    },
};
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
    pub roles: Vec<MaybeKnown<Role>>,
}

/// Extracts the necessary parts for constructing a [`RegistryEntry`] from a protocol minor version.
macro_rules! extract_parts {
    ($name:expr) => {{
        let roles = $crate::msg_types::traits::ProtocolVersion::roles(&$name);
        let protocol = Protocol::from($name);
        let (name, major, minor) = protocol.as_parts();
        (name, major, minor, roles, Protocol::from($name))
    }};
}

fn map_insert(
    map: &mut RegistryMap,
    parts: (&'static str, u8, u8, Vec<MaybeKnown<Role>>, Protocol),
) {
    let (protocol_name, major, minor, roles, protocol) = parts;

    let str_pid = format!(
        "{}/{}/{}.{}",
        Protocol::DID_COM_ORG_PREFIX,
        protocol_name,
        major,
        minor
    );
    let entry = RegistryEntry {
        protocol,
        minor,
        str_pid,
        roles,
    };

    map.entry((protocol_name, major)).or_default().push(entry);
}

lazy_static! {
    /// The protocol registry, used as a baseline for the protocols and versions
    /// that an agent supports along with semver resolution.
    ///
    /// Keys are comprised of the protocol name and major version while
    /// the values are [`RegistryEntry`] instances.
    pub static ref PROTOCOL_REGISTRY: RegistryMap = {
        let mut m = HashMap::new();
        map_insert(&mut m, extract_parts!(RoutingTypeV1::new_v1_0()));
        map_insert(&mut m, extract_parts!(BasicMessageTypeV1::new_v1_0()));
        map_insert(&mut m, extract_parts!(ConnectionTypeV1::new_v1_0()));
        map_insert(&mut m, extract_parts!(SignatureTypeV1::new_v1_0()));
        map_insert(&mut m, extract_parts!(CredentialIssuanceTypeV1::new_v1_0()));
        map_insert(&mut m, extract_parts!(CredentialIssuanceTypeV2::new_v2_0()));
        map_insert(&mut m, extract_parts!(DiscoverFeaturesTypeV1::new_v1_0()));
        map_insert(&mut m, extract_parts!(NotificationTypeV1::new_v1_0()));
        map_insert(&mut m, extract_parts!(OutOfBandTypeV1::new_v1_1()));
        map_insert(&mut m, extract_parts!(PresentProofTypeV1::new_v1_0()));
        map_insert(&mut m, extract_parts!(PresentProofTypeV2::new_v2_0()));
        map_insert(&mut m, extract_parts!(ReportProblemTypeV1::new_v1_0()));
        map_insert(&mut m, extract_parts!(RevocationTypeV2::new_v2_0()));
        map_insert(&mut m, extract_parts!(TrustPingTypeV1::new_v1_0()));
        map_insert(&mut m, extract_parts!(PickupTypeV2::new_v2_0()));
        map_insert(&mut m, extract_parts!(CoordinateMediationTypeV1::new_v1_0()));


        m
    };
}

/// Looks into the protocol registry for (in order):
/// * the exact protocol version requested
/// * the maximum minor version of a protocol less than the minor version requested (e.g: requesting
///   1.7 should yield 1.6).
pub fn get_supported_version(name: &'static str, major: u8, minor: u8) -> Option<u8> {
    PROTOCOL_REGISTRY
        .get(&(name, major))
        .and_then(|v| v.iter().rev().map(|r| r.minor).find(|v| *v <= minor))
}
