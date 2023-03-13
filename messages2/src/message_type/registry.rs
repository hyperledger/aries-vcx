use std::collections::{BTreeMap, HashMap};

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

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

use super::{actor::Actor, MessageFamily};

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

    pub fn as_pid_parts(&self) -> (&str, Option<u8>, Option<u8>) {
        let skip_slash = match self.pid {
            _ if self.pid.starts_with(MessageFamily::DID_COM_ORG_PREFIX) => Some(3),
            _ if self.pid.starts_with(MessageFamily::DID_SOV_PREFIX) => Some(1),
            _ => None,
        };

        let Some(skip_slash) = skip_slash else {
            return (&self.pid, None, None);
        };

        let mut iter = self.pid.split('/').skip(skip_slash);
        let (Some(family), Some(version)) = (iter.next(), iter.next()) else {
            return (&self.pid, None, None);
        };

        let mut version_iter = version.split('.');
        let major = version_iter.next().and_then(|v| v.parse::<u8>().ok());
        let minor = version_iter.next().and_then(|v| v.parse::<u8>().ok());

        let (Some(major), Some(minor)) = (major, minor) else {
            return (&self.pid, None, None);
        };

        (family, Some(major), Some(minor))
    }
}

macro_rules! extract_parts {
    ($name:ty) => {
        (
            <<$name as MinorVersion>::Parent as MajorVersion>::Parent::FAMILY,
            <$name as MinorVersion>::Parent::MAJOR,
            <$name as MinorVersion>::MINOR,
            <$name as MinorVersion>::Parent::actors().to_vec(),
        )
    };
}

fn map_insert(map: &mut RegistryMap, parts: (&'static str, u8, u8, Vec<Actor>)) {
    let (family, major, minor, actors) = parts;

    let pid = format!("{}/{}/{}.{}", MessageFamily::DID_COM_ORG_PREFIX, family, major, minor);
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

pub fn get_supported_version(family: &'static str, major: u8, minor: u8) -> Option<u8> {
    PROTOCOL_REGISTRY
        .get(family)
        .and_then(|m| m.get(&major))
        .and_then(|m| m.keys().rev().find(|v| **v <= minor))
        .copied()
}
