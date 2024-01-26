use crate::cl::{BlindedCredentialSecrets, BlindedCredentialSecretsCorrectnessProof};
use crate::error::{Result, ValidationError};
use crate::invalid;
use crate::utils::validation::{Validatable, LEGACY_DID_IDENTIFIER};

use super::{cred_def::CredentialDefinitionId, nonce::Nonce};

#[derive(Debug, Deserialize, Serialize)]
pub struct CredentialRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    entropy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    prover_did: Option<String>,
    cred_def_id: CredentialDefinitionId,
    pub blinded_ms: BlindedCredentialSecrets,
    pub blinded_ms_correctness_proof: BlindedCredentialSecretsCorrectnessProof,
    pub nonce: Nonce,
}

impl Validatable for CredentialRequest {
    fn validate(&self) -> std::result::Result<(), ValidationError> {
        self.cred_def_id.validate()?;

        match &self.entropy {
            Some(_) => {
                if self.prover_did.is_some() {
                    Err(invalid!("Prover did and entropy must not both be supplied"))
                } else {
                    Ok(())
                }
            }
            None => {
                if self.cred_def_id.is_legacy_cred_def_identifier() {
                    if let Some(prover_did) = self.prover_did.clone() {
                        if LEGACY_DID_IDENTIFIER.captures(&prover_did).is_some() {
                            Ok(())
                        } else {
                            Err(invalid!("Prover did was supplied, not valid"))
                        }
                    } else {
                        Err(invalid!(
                            "Legacy identifiers used but no entropy or prover did was supplied"
                        ))
                    }
                } else {
                    Err(invalid!("entropy is required"))
                }
            }
        }?;
        Ok(())
    }
}

impl CredentialRequest {
    pub fn new(
        entropy: Option<&str>,
        prover_did: Option<&str>,
        cred_def_id: CredentialDefinitionId,
        blinded_ms: BlindedCredentialSecrets,
        blinded_ms_correctness_proof: BlindedCredentialSecretsCorrectnessProof,
        nonce: Nonce,
    ) -> Result<Self> {
        let s = Self {
            entropy: entropy.map(ToOwned::to_owned),
            prover_did: prover_did.map(ToOwned::to_owned),
            cred_def_id,
            blinded_ms,
            blinded_ms_correctness_proof,
            nonce,
        };
        s.validate()?;
        Ok(s)
    }

    pub fn entropy(&self) -> Result<String> {
        self.entropy.clone().map_or_else(
            || {
                self.prover_did
                    .clone()
                    .ok_or_else(|| err_msg!("Entropy or prover did must be supplied"))
            },
            Result::Ok,
        )
    }
}

// #[derive(Debug, Deserialize, Serialize)]
// pub struct CredentialRequestMetadata {
//     pub link_secret_blinding_data: CredentialSecretsBlindingFactors,
//     pub nonce: Nonce,
//     pub link_secret_name: String,
// }
//
// impl Validatable for CredentialRequestMetadata {}
//
// #[cfg(test)]
// mod cred_req_tests {
//     use crate::{
//         data_types::{
//             cred_def::{CredentialDefinition, CredentialKeyCorrectnessProof, SignatureType},
//             cred_offer::CredentialOffer,
//             link_secret::LinkSecret,
//             schema::AttributeNames,
//         },
//         issuer::{create_credential_definition, create_credential_offer, create_schema},
//         prover::create_credential_request,
//         types::CredentialDefinitionConfig,
//     };
//
//     use super::*;
//
//     const NEW_IDENTIFIER: &str = "mock:uri";
//     const LEGACY_DID_IDENTIFIER: &str = "DXoTtQJNtXtiwWaZAK3rB1";
//     const LEGACY_SCHEMA_IDENTIFIER: &str = "DXoTtQJNtXtiwWaZAK3rB1:2:example:1.0";
//     const LEGACY_CRED_DEF_IDENTIFIER: &str = "DXoTtQJNtXtiwWaZAK3rB1:3:CL:98153:default";
//
//     const ENTROPY: Option<&str> = Some("entropy");
//     const PROVER_DID: Option<&str> = Some(LEGACY_DID_IDENTIFIER);
//     const LINK_SECRET_ID: &str = "link:secret:id";
//
//     fn cred_def() -> Result<(CredentialDefinition, CredentialKeyCorrectnessProof)> {
//         let issuer_id = "sample:uri".try_into()?;
//         let schema_id = "schema:id".try_into()?;
//         let credential_definition_issuer_id = "sample:id".try_into()?;
//         let attr_names = AttributeNames::from(vec!["name".to_owned(), "age".to_owned()]);
//
//         let schema = create_schema("schema:name", "1.0", issuer_id, attr_names)?;
//         let cred_def = create_credential_definition(
//             schema_id,
//             &schema,
//             credential_definition_issuer_id,
//             "default",
//             SignatureType::CL,
//             CredentialDefinitionConfig {
//                 support_revocation: true,
//             },
//         )?;
//
//         Ok((cred_def.0, cred_def.2))
//     }
//
//     fn link_secret() -> LinkSecret {
//         LinkSecret::new().unwrap()
//     }
//
//     fn credential_offer(
//         correctness_proof: CredentialKeyCorrectnessProof,
//         is_legacy: bool,
//     ) -> Result<CredentialOffer> {
//         if is_legacy {
//             create_credential_offer(
//                 LEGACY_SCHEMA_IDENTIFIER.try_into()?,
//                 LEGACY_CRED_DEF_IDENTIFIER.try_into()?,
//                 &correctness_proof,
//             )
//         } else {
//             create_credential_offer(
//                 NEW_IDENTIFIER.try_into()?,
//                 NEW_IDENTIFIER.try_into()?,
//                 &correctness_proof,
//             )
//         }
//     }
//
//     #[test]
//     fn create_credential_request_with_valid_input() -> Result<()> {
//         let (cred_def, correctness_proof) = cred_def()?;
//         let link_secret = link_secret();
//         let credential_offer = credential_offer(correctness_proof, false)?;
//
//         let res = create_credential_request(
//             ENTROPY,
//             None,
//             &cred_def,
//             &link_secret,
//             LINK_SECRET_ID,
//             &credential_offer,
//         );
//
//         assert!(res.is_ok());
//
//         Ok(())
//     }
//
//     #[test]
//     fn create_credential_request_with_valid_input_legacy() -> Result<()> {
//         let (cred_def, correctness_proof) = cred_def()?;
//         let link_secret = link_secret();
//         let credential_offer = credential_offer(correctness_proof, true)?;
//
//         let res = create_credential_request(
//             None,
//             PROVER_DID,
//             &cred_def,
//             &link_secret,
//             LINK_SECRET_ID,
//             &credential_offer,
//         );
//
//         assert!(res.is_ok());
//
//         Ok(())
//     }
//
//     #[test]
//     fn create_credential_request_with_invalid_new_identifiers_and_prover_did() -> Result<()> {
//         let (cred_def, correctness_proof) = cred_def()?;
//         let link_secret = link_secret();
//         let credential_offer = credential_offer(correctness_proof, false)?;
//
//         let res = create_credential_request(
//             None,
//             PROVER_DID,
//             &cred_def,
//             &link_secret,
//             LINK_SECRET_ID,
//             &credential_offer,
//         );
//
//         assert!(res.is_err());
//
//         Ok(())
//     }
//
//     #[test]
//     fn create_credential_request_with_invalid_prover_did_and_entropy() -> Result<()> {
//         let (cred_def, correctness_proof) = cred_def()?;
//         let link_secret = link_secret();
//         let credential_offer = credential_offer(correctness_proof, true)?;
//
//         let res = create_credential_request(
//             ENTROPY,
//             PROVER_DID,
//             &cred_def,
//             &link_secret,
//             LINK_SECRET_ID,
//             &credential_offer,
//         );
//
//         assert!(res.is_err());
//
//         Ok(())
//     }
//
//     #[test]
//     fn create_credential_request_with_invalid_prover_did() -> Result<()> {
//         let (cred_def, correctness_proof) = cred_def()?;
//         let link_secret = link_secret();
//         let credential_offer = credential_offer(correctness_proof, true)?;
//
//         let res = create_credential_request(
//             None,
//             ENTROPY,
//             &cred_def,
//             &link_secret,
//             LINK_SECRET_ID,
//             &credential_offer,
//         );
//
//         assert!(res.is_err());
//
//         Ok(())
//     }
//
//     #[test]
//     fn create_credential_request_with_no_entropy_or_prover_did() -> Result<()> {
//         let (cred_def, correctness_proof) = cred_def()?;
//         let link_secret = link_secret();
//         let credential_offer = credential_offer(correctness_proof, true)?;
//
//         let res = create_credential_request(
//             None,
//             None,
//             &cred_def,
//             &link_secret,
//             LINK_SECRET_ID,
//             &credential_offer,
//         );
//
//         assert!(res.is_err());
//
//         Ok(())
//     }
//
//     #[test]
//     fn create_credential_request_json_contains_entropy() -> Result<()> {
//         let (cred_def, correctness_proof) = cred_def()?;
//         let link_secret = link_secret();
//         let credential_offer = credential_offer(correctness_proof, false)?;
//
//         let res = create_credential_request(
//             ENTROPY,
//             None,
//             &cred_def,
//             &link_secret,
//             LINK_SECRET_ID,
//             &credential_offer,
//         )
//         .unwrap();
//
//         let s = serde_json::to_string(&res)?;
//
//         assert!(s.contains("entropy"));
//
//         Ok(())
//     }
//
//     #[test]
//     fn create_credential_request_json_contains_prover_did_with_legacy_identifiers() -> Result<()> {
//         let (cred_def, correctness_proof) = cred_def()?;
//         let link_secret = link_secret();
//         let credential_offer = credential_offer(correctness_proof, true)?;
//
//         let res = create_credential_request(
//             None,
//             PROVER_DID,
//             &cred_def,
//             &link_secret,
//             LINK_SECRET_ID,
//             &credential_offer,
//         )
//         .unwrap();
//
//         let s = serde_json::to_string(&res)?;
//
//         assert!(s.contains("prover_did"));
//
//         Ok(())
//     }
//
//     #[test]
//     fn create_credential_request_json_contains_entropy_with_legacy_identifiers() -> Result<()> {
//         let (cred_def, correctness_proof) = cred_def()?;
//         let link_secret = link_secret();
//         let credential_offer = credential_offer(correctness_proof, false)?;
//
//         let res = create_credential_request(
//             ENTROPY,
//             None,
//             &cred_def,
//             &link_secret,
//             LINK_SECRET_ID,
//             &credential_offer,
//         )
//         .unwrap();
//
//         let s = serde_json::to_string(&res)?;
//
//         assert!(s.contains("entropy"));
//
//         Ok(())
//     }
// }
