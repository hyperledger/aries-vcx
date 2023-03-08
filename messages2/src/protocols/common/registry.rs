use std::collections::{BTreeMap, HashMap};

use lazy_static::lazy_static;

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

type RegistryMap = HashMap<&'static str, HashMap<u8, BTreeMap<u8, Vec<&'static str>>>>;

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

fn map_insert(map: &mut RegistryMap, parts: (&'static str, u8, u8, Vec<&'static str>)) {
    let (family, major, minor, actors) = parts;

    map.entry(family)
        .or_insert(HashMap::new())
        .entry(major)
        .or_insert(BTreeMap::new())
        .insert(minor, actors);
}

lazy_static! {
    static ref PROTOCOL_REGISTRY: RegistryMap = {
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
