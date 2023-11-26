use std::fmt::Display;

use serde::{Deserialize, Serialize};

pub mod didcommv1;
pub mod didcommv2;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum ServiceType {
    #[serde(rename = "endpoint")]
    AIP1,
    #[serde(rename = "did-communication")]
    DIDCommV1,
    #[serde(rename = "DIDCommMessaging")]
    DIDCommV2,
    #[serde(rename = "IndyAgent")]
    Legacy,
}

impl Display for ServiceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceType::AIP1 => write!(f, "endpoint"),
            ServiceType::DIDCommV1 => write!(f, "did-communication"),
            // Interop note: AFJ useses DIDComm, Acapy uses DIDCommMessaging
            // Not matching spec:
            // * did:sov method - https://sovrin-foundation.github.io/sovrin/spec/did-method-spec-template.html#crud-operation-definitions
            // Matching spec:
            // * did:peer method - https://identity.foundation/peer-did-method-spec/#multi-key-creation
            // * did core - https://www.w3.org/TR/did-spec-registries/#didcommmessaging
            // * didcommv2 - https://identity.foundation/didcomm-messaging/spec/#service-endpoint
            ServiceType::DIDCommV2 => write!(f, "DIDCommMessaging"),
            ServiceType::Legacy => write!(f, "IndyAgent"),
        }
    }
}
