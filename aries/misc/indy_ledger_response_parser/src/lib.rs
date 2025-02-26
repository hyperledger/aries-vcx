#[macro_use]
extern crate serde;

extern crate serde_json;

mod domain;
pub mod error;

use anoncreds_clsignatures::RevocationRegistryDelta as ClRevocationRegistryDelta;
pub use domain::author_agreement::GetTxnAuthorAgreementData;
use domain::{author_agreement::GetTxnAuthorAgreementResult, txn::GetTxnReplyResult};
use error::LedgerResponseParserError;
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

pub struct ResponseParser;

impl ResponseParser {
    pub fn parse_get_nym_response(
        &self,
        get_nym_response: &str,
    ) -> Result<NymData, LedgerResponseParserError> {
        let reply: Reply<GetNymReplyResult> = Self::parse_response(get_nym_response)?;

        let nym_data = match reply.result() {
            GetNymReplyResult::GetNymReplyResultV0(res) => {
                let data: GetNymResultDataV0 = res
                    .data
                    .ok_or(LedgerResponseParserError::LedgerItemNotFound("NYM"))
                    .and_then(|data| serde_json::from_str(&data).map_err(Into::into))?;

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
    ) -> Result<Schema, LedgerResponseParserError> {
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
    ) -> Result<CredentialDefinition, LedgerResponseParserError> {
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
                    res.signature_type.to_str(),
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
    ) -> Result<RevocationRegistryDefinition, LedgerResponseParserError> {
        let reply: Reply<GetRevocRegDefReplyResult> =
            Self::parse_response(get_revoc_reg_def_response)?;

        let revoc_reg_def = match reply.result() {
            GetRevocRegDefReplyResult::GetRevocRegDefReplyResultV0(res) => res.data,
            GetRevocRegDefReplyResult::GetRevocRegDefReplyResultV1(res) => res.txn.data,
        };

        Ok(RevocationRegistryDefinition::RevocationRegistryDefinitionV1(revoc_reg_def))
    }

    pub fn parse_get_revoc_reg_response(
        &self,
        get_revoc_reg_response: &str,
    ) -> Result<RevocationRegistryInfo, LedgerResponseParserError> {
        let reply: Reply<GetRevocRegReplyResult> = Self::parse_response(get_revoc_reg_response)?;

        let (revoc_reg_def_id, revoc_reg, timestamp) = match reply.result() {
            GetRevocRegReplyResult::GetRevocRegReplyResultV0(res) => {
                (res.revoc_reg_def_id, res.data, res.txn_time)
            }
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
    ) -> Result<GetTxnAuthorAgreementData, LedgerResponseParserError> {
        let reply: Reply<GetTxnAuthorAgreementResult> = Self::parse_response(taa_response)?;

        let data = match reply.result() {
            GetTxnAuthorAgreementResult::GetTxnAuthorAgreementResultV1(res) => res
                .data
                .ok_or(LedgerResponseParserError::LedgerItemNotFound("TAA"))?,
        };

        Ok(GetTxnAuthorAgreementData {
            text: data.text,
            version: data.version,
            aml: data.aml,
            ratification_ts: data.ratification_ts,
            digest: data.digest,
        })
    }

    pub fn parse_get_revoc_reg_delta_response(
        &self,
        get_revoc_reg_delta_response: &str,
    ) -> Result<RevocationRegistryDeltaInfo, LedgerResponseParserError> {
        let reply: Reply<GetRevocRegDeltaReplyResult> =
            Self::parse_response(get_revoc_reg_delta_response)?;

        let (revoc_reg_def_id, revoc_reg) = match reply.result() {
            GetRevocRegDeltaReplyResult::GetRevocRegDeltaReplyResultV0(res) => {
                (res.revoc_reg_def_id, res.data)
            }
            GetRevocRegDeltaReplyResult::GetRevocRegDeltaReplyResultV1(res) => {
                (res.txn.data.revoc_reg_def_id, res.txn.data.value)
            }
        };

        let revoc_reg_delta = RevocationRegistryDeltaV1 {
            value: serde_json::to_value(ClRevocationRegistryDelta::from_parts(
                revoc_reg.value.accum_from.map(|accum| accum.value).as_ref(),
                &revoc_reg.value.accum_to.value,
                &revoc_reg.value.issued,
                &revoc_reg.value.revoked,
            ))?,
        };

        Ok(RevocationRegistryDeltaInfo {
            revoc_reg_delta: RevocationRegistryDelta::RevocationRegistryDeltaV1(revoc_reg_delta),
            revoc_reg_def_id,
            timestamp: revoc_reg.value.accum_to.txn_time,
        })
    }

    // https://github.com/hyperledger/indy-node/blob/main/docs/source/requests.md#get_txn
    pub fn parse_get_txn_response(
        &self,
        get_txn_response: &str,
    ) -> Result<serde_json::Value, LedgerResponseParserError> {
        let reply: Reply<GetTxnReplyResult> = Self::parse_response(get_txn_response)?;

        let data = match reply.result() {
            GetTxnReplyResult::GetTxnReplyResultV0(res) => {
                res.data.unwrap_or(serde_json::Value::Null)
            }
            GetTxnReplyResult::GetTxnReplyResultV1(res) => res.txn.data,
        };
        Ok(data)
    }

    pub fn parse_response<T>(response: &str) -> Result<Reply<T>, LedgerResponseParserError>
    where
        T: DeserializeOwned + ReplyType + ::std::fmt::Debug,
    {
        // TODO: Distinguish between not found and unexpected response format
        let message: Message<T> = serde_json::from_str(response).map_err(|_| {
            LedgerResponseParserError::LedgerItemNotFound(
                "Structure doesn't correspond to type. Most probably not found",
            )
        })?;

        match message {
            Message::Reject(response) | Message::ReqNACK(response) => Err(
                LedgerResponseParserError::InvalidTransaction(response.reason),
            ),
            Message::Reply(reply) => Ok(reply),
        }
    }
}
