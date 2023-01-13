use std::{collections::HashMap, str::from_utf8};

use crate::utils::crypto::base58::ToBase58;
use async_std::sync::Arc;
use async_trait::async_trait;
use indy_api_types::{errors::*, PoolHandle};
use indy_utils::next_pool_handle;

use crate::controllers::vdr::Ledger;
use crate::domain::{
    anoncreds::credential_definition::{CredentialDefinition, CredentialDefinitionId},
    anoncreds::schema::{Schema, SchemaId},
    crypto::did::DidValue,
    ledger::did::NymTxnParams,
    ledger::request::Request,
    pool::{PoolMode, PoolOpenConfig},
    vdr::{
        ledger_types::DidMethod,
        ping_status::PingStatus,
        prepared_txn::{EndorsementSpec, IndyEndorsement, IndyEndorsementSpec},
        taa_config::TAAConfig,
    },
};
use crate::services::{LedgerService, PoolService};

pub(crate) struct IndyLedger {
    name: String,
    genesis_txn: String,
    handle: PoolHandle,
    taa_config: Option<TAAConfig>,
    ledger_service: Arc<LedgerService>,
    pool_service: Arc<PoolService>,
}

impl IndyLedger {
    pub(crate) fn create(
        genesis_txn: String,
        taa_config: Option<TAAConfig>,
        ledger_service: Arc<LedgerService>,
        pool_service: Arc<PoolService>,
    ) -> IndyResult<IndyLedger> {
        trace!(
            "create > genesis_txn {:?} taa_config {:?}",
            genesis_txn,
            taa_config,
        );
        let name = uuid::Uuid::new_v4().to_string();
        let handle = next_pool_handle();
        Ok(IndyLedger {
            name,
            genesis_txn,
            taa_config,
            handle,
            ledger_service,
            pool_service,
        })
    }
}

#[async_trait]
impl Ledger for IndyLedger {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn ledger_type(&self) -> DidMethod {
        DidMethod::Indy
    }

    async fn ping(&self) -> IndyResult<PingStatus> {
        trace!("create > ping",);

        if self.pool_service.is_pool_opened(self.handle).await {
            match self.pool_service.refresh(self.handle).await {
                Ok(transactions) => Ok(PingStatus::success(transactions)),
                Err(err) => Ok(PingStatus::fail(err)),
            }
        } else {
            let config = PoolOpenConfig {
                pool_mode: PoolMode::InMemory,
                transactions: Some(self.genesis_txn.clone()),
                ..PoolOpenConfig::default()
            };
            match self
                .pool_service
                .open(self.name.to_string(), Some(config), Some(self.handle))
                .await
            {
                Ok((_, transactions)) => Ok(PingStatus::success(transactions)),
                Err(err) => Ok(PingStatus::fail(err)),
            }
        }
    }

    async fn submit_txn(
        &self,
        txn_bytes: &[u8],
        signature: &[u8],
        endorsement: Option<&str>,
    ) -> IndyResult<String> {
        trace!(
            "submit_txn > txn_bytes {:?} signature {:?} endorsement {:?}",
            txn_bytes,
            signature,
            endorsement,
        );
        let transaction = self.txn_from_bytes(txn_bytes)?;
        let transaction =
            self.set_txn_signatures(&transaction, signature, endorsement.as_deref())?;
        self._submit_txn(&transaction).await
    }

    async fn submit_raw_txn(&self, txn_bytes: &[u8]) -> IndyResult<String> {
        trace!("submit_raw_txn > txn_bytes {:?}", txn_bytes,);
        let transaction = self.txn_from_bytes(txn_bytes)?;
        self._submit_txn(&transaction).await
    }

    async fn submit_query(&self, query: &str) -> IndyResult<String> {
        trace!("submit_query > query {:?}", query,);
        self._submit_txn(query).await
    }

    async fn cleanup(&self) -> IndyResult<()> {
        trace!("cleanup >",);
        if self.pool_service.is_pool_opened(self.handle).await {
            self.pool_service.close(self.handle).await?;
        }
        Ok(())
    }

    async fn build_did_request(
        &self,
        txn_params: &str,
        submitter_did: &str,
        endorser: Option<&str>,
    ) -> IndyResult<(Vec<u8>, Vec<u8>)> {
        trace!(
            "build_did_request > submitter_did {:?} txn_params {:?} endorser {:?}",
            submitter_did,
            txn_params,
            endorser,
        );
        let did_params: NymTxnParams = serde_json::from_str(txn_params).map_err(|err| {
            err_msg(
                IndyErrorKind::InvalidStructure,
                format!(
                    "Unable to parse indy DID from param: {}. Err: {:?}",
                    txn_params, err
                ),
            )
        })?;

        let transaction = self.ledger_service.build_nym_request(
            &DidValue(submitter_did.to_string()),
            &did_params.dest,
            did_params.verkey.as_deref(),
            did_params.alias.as_deref(),
            did_params.role.as_deref(),
        )?;
        self.prepare_resolve_request_result(&transaction, endorser)
    }

    async fn build_resolve_did_request(&self, id: &str) -> IndyResult<String> {
        trace!("build_resolve_did_request > id {:?}", id,);
        self.ledger_service
            .build_get_nym_request(None, &DidValue(id.to_string()))
    }

    async fn parse_resolve_did_response(&self, response: &str) -> IndyResult<String> {
        trace!("parse_resolve_did_response > response {:?}", response,);
        self.ledger_service.parse_get_nym_response(&response)
    }

    async fn build_schema_request(
        &self,
        txn_params: &str,
        submitter_did: &str,
        endorser: Option<&str>,
    ) -> IndyResult<(Vec<u8>, Vec<u8>)> {
        trace!(
            "build_schema_request > submitter_did {:?} txn_params {:?} endorser {:?}",
            submitter_did,
            txn_params,
            endorser,
        );
        let schema: Schema = serde_json::from_str(txn_params).map_err(|err| {
            err_msg(
                IndyErrorKind::InvalidStructure,
                format!(
                    "Unable to parse indy Schema from param: {}. Err: {:?}",
                    txn_params, err
                ),
            )
        })?;

        let transaction = self
            .ledger_service
            .build_schema_request(&DidValue(submitter_did.to_string()), schema)?;
        self.prepare_resolve_request_result(&transaction, endorser)
    }

    async fn build_resolve_schema_request(&self, id: &str) -> IndyResult<String> {
        trace!("build_resolve_schema_request > id {:?}", id,);
        self.ledger_service
            .build_get_schema_request(None, &SchemaId(id.to_string()))
    }

    async fn parse_resolve_schema_response(&self, response: &str) -> IndyResult<String> {
        trace!("parse_resolve_schema_response > response {:?}", response,);
        self.ledger_service
            .parse_get_schema_response(&response, None)
            .map(|(_, response)| response)
    }

    async fn build_cred_def_request(
        &self,
        txn_params: &str,
        submitter_did: &str,
        endorser: Option<&str>,
    ) -> IndyResult<(Vec<u8>, Vec<u8>)> {
        trace!(
            "build_cred_def_request > submitter_did {:?} txn_params {:?} endorser {:?}",
            submitter_did,
            txn_params,
            endorser,
        );
        let cred_def: CredentialDefinition = serde_json::from_str(txn_params).map_err(|err| {
            err_msg(
                IndyErrorKind::InvalidStructure,
                format!(
                    "Unable to parse indy CredentialDefinition from param: {}. Err: {:?}",
                    txn_params, err
                ),
            )
        })?;

        let transaction = self
            .ledger_service
            .build_cred_def_request(&DidValue(submitter_did.to_string()), cred_def)?;
        self.prepare_resolve_request_result(&transaction, endorser)
    }

    async fn build_resolve_cred_def_request(&self, id: &str) -> IndyResult<String> {
        trace!("build_resolve_cred_def_request > id {:?}", id,);
        self.ledger_service
            .build_get_cred_def_request(None, &CredentialDefinitionId(id.to_string()))
    }

    async fn parse_resolve_cred_def_response(&self, response: &str) -> IndyResult<String> {
        trace!("parse_resolve_cred_def_response > response {:?}", response,);
        self.ledger_service
            .parse_get_cred_def_response(&response, None)
            .map(|(_, response)| response)
    }

    fn prepare_endorsement_spec(
        &self,
        endorser: Option<&str>,
    ) -> IndyResult<Option<EndorsementSpec>> {
        trace!("prepare_endorsement_spec > endorser {:?}", endorser,);
        match endorser {
            Some(endorser) => Ok(Some(EndorsementSpec::Indy(IndyEndorsementSpec {
                endorser_did: endorser.to_string(),
            }))),
            None => Ok(None),
        }
    }
}

impl IndyLedger {
    fn txn_from_bytes(&self, txn_bytes: &[u8]) -> IndyResult<String> {
        let transaction = from_utf8(txn_bytes)
            .map_err(|err| {
                err_msg(
                    IndyErrorKind::InvalidTransaction,
                    format!("Unable to restore transaction from bytes. Err: {:?}", err),
                )
            })?
            .to_string();
        Ok(transaction)
    }

    fn prepare_resolve_request_result(
        &self,
        transaction: &str,
        endorser: Option<&str>,
    ) -> IndyResult<(Vec<u8>, Vec<u8>)> {
        let transaction = self.append_txn_extra_fields(transaction, endorser)?;
        let transaction_bytes = transaction.as_bytes().to_vec();
        let (bytes_to_sign, _) = self.ledger_service.get_txn_bytes_to_sign(&transaction)?;
        Ok((transaction_bytes, bytes_to_sign))
    }

    async fn _submit_txn(&self, transaction: &str) -> IndyResult<String> {
        let result = self.pool_service.send_tx(self.handle, &transaction).await?;
        validate_txn_response(result)
    }

    fn set_txn_signatures(
        &self,
        transaction: &str,
        signature: &[u8],
        endorsement: Option<&str>,
    ) -> IndyResult<String> {
        trace!(
            "set_txn_signatures > transaction {:?} signature {:?} endorsement {:?}",
            transaction,
            signature,
            endorsement,
        );
        let mut transaction: Request<serde_json::Value> = serde_json::from_str(&transaction)
            .map_err(|err| {
                err_msg(
                    IndyErrorKind::InvalidStructure,
                    format!("Unable to parse indy transaction. Err: {:?}", err),
                )
            })?;

        let identifier = transaction.identifier.as_ref().ok_or(err_msg(
            IndyErrorKind::InvalidStructure,
            "Invalid transaction: `identifier` field is missing in the request.",
        ))?;

        match endorsement {
            None => {
                transaction.signature = Some(signature.to_base58());
            }
            Some(endorsment) => {
                let endorsement: IndyEndorsement =
                    serde_json::from_str(&endorsment).map_err(|err| {
                        err_msg(
                            IndyErrorKind::InvalidStructure,
                            format!("Unable to parse indy endorsement data. Err: {:?}", err),
                        )
                    })?;

                let endorser = transaction.endorser.ok_or(err_msg(
                    IndyErrorKind::InvalidStructure,
                    "Invalid transaction: `endorser` field is missing in the request.",
                ))?;

                let mut signatures: HashMap<String, String> = HashMap::new();
                signatures.insert(identifier.0.to_string(), signature.to_base58());
                signatures.insert(endorser.0.to_string(), endorsement.signature);

                transaction.endorser = Some(endorser);
                transaction.signatures = Some(signatures);
            }
        }
        json_string_result!(transaction)
    }

    fn append_txn_extra_fields(
        &self,
        transaction: &str,
        endorser: Option<&str>,
    ) -> IndyResult<String> {
        trace!(
            "append_txn_extra_fields > transaction {:?} endorser {:?}",
            transaction,
            endorser,
        );
        let mut request: Request<serde_json::Value> =
            serde_json::from_str(&transaction).map_err(|err| {
                err_msg(
                    IndyErrorKind::InvalidStructure,
                    format!("Unable to parse indy transaction. Err: {:?}", err),
                )
            })?;

        self.append_txn_author_agreement_acceptance_to_request(&mut request)?;
        self.append_txn_endorser(&mut request, endorser)?;
        json_string_result!(request)
    }

    fn append_txn_endorser(
        &self,
        transaction: &mut Request<serde_json::Value>,
        endorser: Option<&str>,
    ) -> IndyResult<()> {
        trace!(
            "append_txn_endorser > transaction {:?} endorser {:?}",
            transaction,
            endorser,
        );
        if let Some(endorser) = endorser {
            self.ledger_service
                .append_txn_endorser(transaction, &DidValue(endorser.to_string()).to_short())?;
        }
        Ok(())
    }

    fn append_txn_author_agreement_acceptance_to_request(
        &self,
        transaction: &mut Request<serde_json::Value>,
    ) -> IndyResult<()> {
        trace!(
            "append_txn_author_agreement_acceptance_to_request > transaction {:?}",
            transaction,
        );
        if let Some(ref taa_config) = self.taa_config {
            self.ledger_service
                .append_txn_author_agreement_acceptance_to_request(
                    transaction,
                    taa_config.text.as_deref(),
                    taa_config.version.as_deref(),
                    taa_config.taa_digest.as_deref(),
                    &taa_config.acc_mech_type,
                    taa_config.time,
                )?;
        }
        Ok(())
    }
}

fn validate_txn_response(response: String) -> IndyResult<String> {
    trace!("validate_txn_response >>> {}", response);
    let message: serde_json::Value = serde_json::from_str(&response).to_indy(
        IndyErrorKind::InvalidTransaction,
        "Response is invalid json",
    )?;

    if message["op"].as_str() == Some("REPLY") {
        Ok(response)
    } else {
        let reason = message["reason"]
            .as_str()
            .unwrap_or("no failure reason provided");
        Err(err_msg(
            IndyErrorKind::InvalidTransaction,
            format!("Transaction has been failed: {:?}", reason),
        ))
    }
}
