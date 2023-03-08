use std::collections::{BTreeMap, HashMap};

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

use crate::message_type::message_family::{
    basic_message::BasicMessageV1_0,
    connection::ConnectionV1_0,
    cred_issuance::CredentialIssuanceV1_0,
    discover_features::DiscoverFeaturesV1_0,
    notification::NotificationV1_0,
    out_of_band::OutOfBandV1_1,
    present_proof::PresentProofV1_0,
    report_problem::ReportProblemV1_0,
    revocation::RevocationV2_0,
    traits::{ResolveMajorVersion, ResolveMinorVersion, ResolveMsgKind},
    trust_ping::TrustPingV1_0,
};

use super::{actor::Actor, prefix::Prefix};

type RegistryMap = HashMap<&'static str, HashMap<u8, BTreeMap<u8, ProtocolDescriptor>>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolDescriptor {
    pub pid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roles: Option<Vec<Actor>>,
}

impl ProtocolDescriptor {
    pub fn new(pid: String) -> Self {
        Self { pid, roles: None }
    }
}

macro_rules! extract_parts {
    ($name:ty) => {
        (
            <<$name as ResolveMsgKind>::Parent as ResolveMinorVersion>::Parent::FAMILY,
            <$name as ResolveMsgKind>::Parent::MAJOR,
            <$name as ResolveMsgKind>::MINOR,
            <$name as ResolveMsgKind>::Parent::actors().to_vec(),
        )
    };
}

fn map_insert(map: &mut RegistryMap, parts: (&'static str, u8, u8, Vec<Actor>)) {
    let (family, major, minor, actors) = parts;

    let pid = format!("{}/{}/{}.{}", Prefix::DID_COM_ORG_PREFIX, family, major, minor);
    let mut pd = ProtocolDescriptor::new(pid);
    pd.roles = Some(actors);

    map.entry(family)
        .or_insert(HashMap::new())
        .entry(major)
        .or_insert(BTreeMap::new())
        .insert(minor, pd);
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
