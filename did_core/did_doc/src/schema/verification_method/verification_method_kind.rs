use did_parser_nom::DidUrl;
use serde::{Deserialize, Serialize};

use super::VerificationMethod;

// Either a set of verification methods maps or DID URLs
// https://www.w3.org/TR/did-core/#did-document-properties
#[allow(clippy::large_enum_variant)]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum VerificationMethodKind {
    Resolved(VerificationMethod),
    Resolvable(DidUrl),
}
