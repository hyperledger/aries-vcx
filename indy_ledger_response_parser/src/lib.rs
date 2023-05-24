#[macro_use]
extern crate serde;

extern crate serde_json;

mod domain;

use domain::author_agreement::GetTxnAuthorAgreementResult;
pub use indy_api_types::{errors, ErrorCode};
use indy_api_types::{
    errors::{err_msg, IndyErrorKind, IndyResult, IndyResultExt},
    IndyError,
};
use indy_vdr::{
    ledger::{
        identifiers::{CredentialDefinitionId, RevocationRegistryId, SchemaId},
        requests::{
            cred_def::{CredentialDefinition, CredentialDefinitionV1},
            rev_reg::{RevocationRegistry, RevocationRegistryDelta, RevocationRegistryDeltaV1},
            rev_reg_def::RevocationRegistryDefinition,
            schema::{Schema, SchemaV1},
        },
    },
    utils::did::DidValue,
};
use serde::de::DeserializeOwned;
// TODO: Can we replace this to get rid of dependency on Ursa
use ursa::cl::RevocationRegistryDelta as UrsaRevocationDelta;

use crate::domain::{
    cred_def::GetCredDefReplyResult,
    did::{GetNymReplyResult, GetNymResultDataV0, NymData},
    response::{Message, Reply, ReplyType},
    rev_reg::{GetRevocRegDeltaReplyResult, GetRevocRegReplyResult},
    rev_reg_def::GetRevocRegDefReplyResult,
    schema::GetSchemaReplyResult,
};

pub struct RevocationRegistryInfo {
    pub revoc_reg: RevocationRegistry,
    pub revoc_reg_def_id: RevocationRegistryId,
    pub timestamp: u64,
}

pub struct RevocationRegistryDeltaInfo {
    pub revoc_reg_delta: RevocationRegistryDelta,
    pub revoc_reg_def_id: RevocationRegistryId,
    pub timestamp: u64,
}

pub struct ResponseParser {}

impl ResponseParser {
    pub fn new() -> Self {
        Self {}
    }

    pub fn parse_get_nym_response(&self, get_nym_response: &str) -> IndyResult<NymData> {
        let reply: Reply<GetNymReplyResult> = Self::parse_response(get_nym_response)?;

        let nym_data = match reply.result() {
            GetNymReplyResult::GetNymReplyResultV0(res) => {
                let data: GetNymResultDataV0 = res
                    .data
                    .ok_or_else(|| IndyError::from_msg(IndyErrorKind::LedgerItemNotFound, format!("Nym not found")))
                    .and_then(|data| {
                        serde_json::from_str(&data).map_err(|err| {
                            IndyError::from_msg(
                                IndyErrorKind::InvalidState,
                                format!("Cannot parse GET_NYM response: {}", err),
                            )
                        })
                    })?;

                NymData {
                    did: data.dest,
                    verkey: data.verkey,
                    role: data.role,
                }
            }
            GetNymReplyResult::GetNymReplyResultV1(res) => NymData {
                did: res.txn.data.did,
                verkey: res.txn.data.verkey,
                role: res.txn.data.role,
            },
        };

        Ok(nym_data)
    }

    pub fn parse_get_schema_response(
        &self,
        get_schema_response: &str,
        method_name: Option<&str>,
    ) -> IndyResult<Schema> {
        let reply: Reply<GetSchemaReplyResult> = Self::parse_response(get_schema_response)?;

        let schema = match reply.result() {
            GetSchemaReplyResult::GetSchemaReplyResultV0(res) => SchemaV1 {
                id: SchemaId::new(
                    &DidValue::new(&res.dest.0, method_name),
                    &res.data.name,
                    &res.data.version,
                ),
                attr_names: res.data.attr_names.into(),
                name: res.data.name,
                version: res.data.version,
                seq_no: Some(res.seq_no),
            },
            GetSchemaReplyResult::GetSchemaReplyResultV1(res) => SchemaV1 {
                id: SchemaId::new(
                    &DidValue::new(&res.txn.data.id, method_name),
                    &res.txn.data.schema_name,
                    &res.txn.data.schema_version,
                ),
                attr_names: res.txn.data.value.attr_names.into(),
                name: res.txn.data.schema_name,
                version: res.txn.data.schema_version,
                seq_no: Some(res.txn_metadata.seq_no),
            },
        };

        Ok(Schema::SchemaV1(schema))
    }

    pub fn parse_get_cred_def_response(
        &self,
        get_cred_def_response: &str,
        method_name: Option<&str>,
    ) -> IndyResult<CredentialDefinition> {
        let reply: Reply<GetCredDefReplyResult> = Self::parse_response(get_cred_def_response)?;

        let cred_def = match reply.result() {
            GetCredDefReplyResult::GetCredDefReplyResultV0(res) => CredentialDefinitionV1 {
                schema_id: SchemaId(res.ref_.to_string()),
                signature_type: res.signature_type,
                tag: res.tag.clone().unwrap_or_default(),
                value: res.data,
                id: CredentialDefinitionId::new(
                    &DidValue::new(&res.origin.0, method_name),
                    &SchemaId(res.ref_.to_string()),
                    &res.signature_type.to_str(),
                    &res.tag.clone().unwrap_or_default(),
                ),
            },
            GetCredDefReplyResult::GetCredDefReplyResultV1(res) => CredentialDefinitionV1 {
                id: res.txn.data.id,
                schema_id: res.txn.data.schema_ref,
                signature_type: res.txn.data.type_,
                tag: res.txn.data.tag,
                value: res.txn.data.public_keys,
            },
        };

        Ok(CredentialDefinition::CredentialDefinitionV1(cred_def))
    }

    pub fn parse_get_revoc_reg_def_response(
        &self,
        get_revoc_reg_def_response: &str,
    ) -> IndyResult<RevocationRegistryDefinition> {
        let reply: Reply<GetRevocRegDefReplyResult> = Self::parse_response(get_revoc_reg_def_response)?;

        let revoc_reg_def = match reply.result() {
            GetRevocRegDefReplyResult::GetRevocRegDefReplyResultV0(res) => res.data,
            GetRevocRegDefReplyResult::GetRevocRegDefReplyResultV1(res) => res.txn.data,
        };

        Ok(RevocationRegistryDefinition::RevocationRegistryDefinitionV1(
            revoc_reg_def,
        ))
    }

    pub fn parse_get_revoc_reg_response(&self, get_revoc_reg_response: &str) -> IndyResult<RevocationRegistryInfo> {
        let reply: Reply<GetRevocRegReplyResult> = Self::parse_response(get_revoc_reg_response)?;

        let (revoc_reg_def_id, revoc_reg, timestamp) = match reply.result() {
            GetRevocRegReplyResult::GetRevocRegReplyResultV0(res) => (res.revoc_reg_def_id, res.data, res.txn_time),
            GetRevocRegReplyResult::GetRevocRegReplyResultV1(res) => (
                res.txn.data.revoc_reg_def_id,
                res.txn.data.value,
                res.txn_metadata.creation_time,
            ),
        };

        Ok(RevocationRegistryInfo {
            revoc_reg: RevocationRegistry::RevocationRegistryV1(revoc_reg),
            revoc_reg_def_id,
            timestamp,
        })
    }

    pub fn parse_get_txn_author_agreement_response(
        &self,
        taa_response: &str,
    ) -> IndyResult<GetTxnAuthorAgreementResult> {
        let reply: Reply<GetTxnAuthorAgreementResult> = Self::parse_response(taa_response)?;
        Ok(reply.result())
    }

    pub fn parse_get_revoc_reg_delta_response(
        &self,
        get_revoc_reg_delta_response: &str,
    ) -> IndyResult<RevocationRegistryDeltaInfo> {
        let reply: Reply<GetRevocRegDeltaReplyResult> = Self::parse_response(get_revoc_reg_delta_response)?;

        let (revoc_reg_def_id, revoc_reg) = match reply.result() {
            GetRevocRegDeltaReplyResult::GetRevocRegDeltaReplyResultV0(res) => (res.revoc_reg_def_id, res.data),
            GetRevocRegDeltaReplyResult::GetRevocRegDeltaReplyResultV1(res) => {
                (res.txn.data.revoc_reg_def_id, res.txn.data.value)
            }
        };

        let revoc_reg_delta = RevocationRegistryDeltaV1 {
            value: serde_json::to_value(UrsaRevocationDelta::from_parts(
                revoc_reg.value.accum_from.map(|accum| accum.value).as_ref(),
                &revoc_reg.value.accum_to.value,
                &revoc_reg.value.issued,
                &revoc_reg.value.revoked,
            ))
            .to_indy(
                IndyErrorKind::InvalidStructure,
                "Cannot convert RevocationRegistryDelta to Value",
            )?,
        };

        Ok(RevocationRegistryDeltaInfo {
            revoc_reg_delta: RevocationRegistryDelta::RevocationRegistryDeltaV1(revoc_reg_delta),
            revoc_reg_def_id,
            timestamp: revoc_reg.value.accum_to.txn_time,
        })
    }

    pub fn parse_response<T>(response: &str) -> IndyResult<Reply<T>>
    where
        T: DeserializeOwned + ReplyType + ::std::fmt::Debug,
    {
        let message: Message<T> = serde_json::from_str(response).to_indy(
            IndyErrorKind::LedgerItemNotFound,
            "Structure doesn't correspond to type. Most probably not found",
        )?; // FIXME: Review how we handle not found

        match message {
            Message::Reject(response) | Message::ReqNACK(response) => Err(err_msg(
                IndyErrorKind::InvalidTransaction,
                format!("Transaction has been failed: {:?}", response.reason),
            )),
            Message::Reply(reply) => Ok(reply),
        }
    }
}
