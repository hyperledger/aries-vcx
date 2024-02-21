use std::collections::HashMap;

use super::pres_request::NonRevokedInterval;
use crate::data_types::identifiers::{cred_def_id::CredentialDefinitionId, schema_id::SchemaId};

/// Data structure representing the credentials in the wallet, which are suitable
/// for presentation against a proof request.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct RetrievedCredentials {
    /// A map of the proof request's requested referents (predicates and attribute referents)
    /// against a list of [RetrievedCredentialForReferent] items which represent credentials
    /// suitable for the given referent.
    #[serde(rename = "attrs", skip_serializing_if = "HashMap::is_empty", default)]
    pub credentials_by_referent: HashMap<String, Vec<RetrievedCredentialForReferent>>,
}

/// Data structure containing information about the credential which is suitable for a given
/// referent (`cred_info`), and the `interval` of non-revocation that was requested in the
/// original proof request (if requested).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RetrievedCredentialForReferent {
    pub cred_info: RetrievedCredentialInfo,
    pub interval: Option<NonRevokedInterval>,
}

/// A convenience data structure showing the metadata details (information) of a credential
/// in a wallet that has been retrieved as being 'suitable' for a proof request referent.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RetrievedCredentialInfo {
    /// The unique identifier of the credential in the wallet
    pub referent: String,
    /// Map of string key values representing all the attributes this credential has
    #[serde(rename = "attrs")]
    pub attributes: HashMap<String, String>,
    pub schema_id: SchemaId,
    pub cred_def_id: CredentialDefinitionId,
    pub rev_reg_id: Option<String>,
    pub cred_rev_id: Option<u32>,
}

/// Data structure presenting the credentials which have been selected for usage
/// in creating a proof presentation in response to a proof request.
///
/// Typically [SelectedCredentials] is constructed by selecting credential items
/// from [RetrievedCredentials] for each referent, however manual construction
/// can be done if required (e.g. if credential data is managed elsewhere).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct SelectedCredentials {
    /// Map of referents (predicate and attribute) from the original proof request
    /// to the credential to use in proving that referent: [SelectedCredentialForReferent].
    #[serde(rename = "attrs", skip_serializing_if = "HashMap::is_empty", default)]
    pub credential_for_referent: HashMap<String, SelectedCredentialForReferent>,
}

/// Data structure nesting further details about the selected credential for a
/// proof request referent. Including the credential details and configuration
/// for tails files if a non-revocation proof is neccessary.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SelectedCredentialForReferent {
    pub credential: SelectedCredentialForReferentCredential,
    /// If wanting to create a non-revocation proof, `tails_dir` should be provided
    /// and point to the absolute file path for a directory containing the tails
    /// file for the credential's revocation registry. Note that the files within this
    /// dir should be pre-downloaded and named by the tailsFileHash (base58), as
    /// specified in the revocation registry definition for the credential.
    pub tails_dir: Option<String>,
}

// NOTE: the only reason this is in a nested data struct is for backwards compatible
// serialization reasons. It is nested as originally it made mapping the
// [RetrievedCredentialForReferent] JSON value into a [SelectedCredentialForReferentCredential] much
// more convenient.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SelectedCredentialForReferentCredential {
    pub cred_info: SelectedCredentialInfo,
}

// NOTE: this type is very similar to [RetrievedCredentialInfo] above,
// with the exception of `revealed` field being added and `attrs` field being removed
/// Data structure with the details of the credential to be used. Can be easily
/// constructed using the [RetrievedCredentials]'s [RetrievedCredentialInfo] items data.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SelectedCredentialInfo {
    /// The unique identifier of the credential in the wallet
    pub referent: String,
    pub schema_id: SchemaId,
    pub cred_def_id: CredentialDefinitionId,
    pub rev_reg_id: Option<String>,
    pub cred_rev_id: Option<u32>,
    /// Whether the raw attribute value/s should be proven and sent to the verifier.
    /// Selecting false will still produce a proof for this credential, but no details
    /// about the attributes values will be revealed.
    /// If [None] is selected, aries-vcx will choose a default.
    /// Selecting a value other than [None] for a credential being used in a predicate
    /// referent proof will have no effect.
    pub revealed: Option<bool>,
}

// Utility method for easily translating between a retrieved credential for referent
// into a selected credential for referent.
impl From<RetrievedCredentialForReferent> for SelectedCredentialForReferentCredential {
    fn from(value: RetrievedCredentialForReferent) -> Self {
        SelectedCredentialForReferentCredential {
            cred_info: SelectedCredentialInfo {
                referent: value.cred_info.referent,
                schema_id: value.cred_info.schema_id,
                cred_def_id: value.cred_info.cred_def_id,
                rev_reg_id: value.cred_info.rev_reg_id,
                cred_rev_id: value.cred_info.cred_rev_id,
                revealed: None, // default as no-preference for revealed
            },
        }
    }
}

impl SelectedCredentials {
    /// Utility builder method for [SelectedCredentials] attribute creds, used to allow easy
    /// translation from items of [RetrievedCredentials] into [SelectedCredentials] items.
    ///
    /// for the given `referent`, the `retrieved_cred` (from [RetrievedCredentials]) is selected for
    /// presentation. `with_tails_dir` should be provided if the `retrieved_cred` should be
    /// presented with a non-revocation proof. `with_tails_dir` should point to the absolute
    /// path of a directory containing the relevant tails file for the credential's revocation
    /// registry.
    pub fn select_credential_for_referent_from_retrieved(
        &mut self,
        referent: String,
        retrieved_cred: RetrievedCredentialForReferent,
        with_tails_dir: Option<String>,
    ) {
        self.credential_for_referent.insert(
            referent,
            SelectedCredentialForReferent {
                credential: SelectedCredentialForReferentCredential::from(retrieved_cred),
                tails_dir: with_tails_dir,
            },
        );
    }
}
