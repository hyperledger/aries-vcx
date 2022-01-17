use indy_api_types::{errors::*};
use async_trait::async_trait;
use cosmrs::tx::SignDoc;
use async_std::sync::Arc;
use indy_utils::crypto::base64;

use crate::domain::{
    vdr::{
        prepared_txn::{
            EndorsementSpec,
            CheqdEndorsementSpec,
            CheqdEndorsement,
        },
        ledger_types::LedgerTypes,
        ping_status::PingStatus,
    },
    cheqd_ledger::cheqd::v1::messages::{
        MsgCreateCredDef,
        MsgCreateSchema,
    },
    cheqd_ledger::{
        cheqd::v1::models::DidTxnParams,
        cosmos_ext::CosmosSignDocExt,
    },
    cheqd_pool::{AddPoolConfig, PoolMode},
};
use crate::controllers::vdr::Ledger;
use crate::services::{
    CheqdPoolService,
    CheqdLedgerService,
};

pub(crate) struct CheqdLedger {
    name: String,
    ledger_service: Arc<CheqdLedgerService>,
    pool_service: Arc<CheqdPoolService>,
}

impl CheqdLedger {
    pub(crate) async fn create(chain_id: &str,
                               rpc_address: &str,
                               ledger_service: Arc<CheqdLedgerService>,
                               pool_service: Arc<CheqdPoolService>) -> IndyResult<CheqdLedger> {
        trace!(
            "create > chain_id {:?} rpc_address {:?}",
            chain_id, rpc_address
        );
        let name = uuid::Uuid::new_v4().to_string();
        let config = AddPoolConfig {
            rpc_address: rpc_address.to_string(),
            chain_id: chain_id.to_string(),
            pool_mode: PoolMode::InMemory
        };
        pool_service.add(&name, &config).await?;
        Ok(CheqdLedger {
            name: name.to_string(),
            ledger_service,
            pool_service,
        })
    }
}

#[async_trait]
impl Ledger for CheqdLedger {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn ledger_type(&self) -> LedgerTypes {
        LedgerTypes::Cheqd
    }

    async fn ping(&self) -> IndyResult<PingStatus> {
        trace!(
            "ping >"
        );
        match self.pool_service.abci_info(&self.name).await {
            Ok(info) => Ok(PingStatus::success(json_string!(info))),
            Err(err) => Ok(PingStatus::fail(err))
        }
    }

    async fn submit_txn(&self, tx_bytes: &[u8], signature: &[u8], endorsement: Option<&str>) -> IndyResult<String> {
        trace!(
            "submit_txn > tx_bytes {:?} signature {:?} endorsement {:?}",
            tx_bytes, signature, endorsement
        );
        let tx_bytes = self.prepare_txn(tx_bytes, signature, endorsement).await?;
        self.submit_raw_txn(&tx_bytes).await
    }

    async fn submit_raw_txn(&self, txn_bytes: &[u8]) -> IndyResult<String> {
        trace!(
            "submit_raw_txn > txn_bytes {:?}",
            txn_bytes
        );
        let response = self.pool_service.broadcast_tx_commit(&self.name, &txn_bytes).await?;
        json_string_result!(response)
    }

    async fn submit_query(&self, query: &str) -> IndyResult<String> {
        trace!(
            "submit_raw_txn > query {:?}",
            query
        );
        self.pool_service.abci_query(&self.name, &query).await
    }

    async fn cleanup(&self) -> IndyResult<()> {
        trace!(
            "cleanup >",
        );
        Ok(())
    }

    async fn build_did_request(&self, txn_params: &str, submitter_did: &str, endorser: Option<&str>) -> IndyResult<(Vec<u8>, Vec<u8>)> {
        trace!(
            "build_did_request > submitter_did {:?} txn_params {:?} endorser {:?}",
            submitter_did, txn_params, endorser
        );
        let did_txn_params: DidTxnParams = serde_json::from_str(txn_params)
            .map_err(|err| err_msg(
                IndyErrorKind::InvalidStructure,
                format!("Unable to parse Cheqd DID TXN params from JSON: {}. Err: {:?}", txn_params, err),
            ))?;

        let message = self.ledger_service.cheqd_build_msg_create_did(&did_txn_params.did,
                                                                     &did_txn_params.verkey)?;

        Ok((message.clone(), message))
    }

    async fn build_resolve_did_request(&self, id: &str) -> IndyResult<String> {
        trace!(
            "build_resolve_did_request > id {:?}",
            id
        );
        self.ledger_service.cheqd_build_query_get_did(id)
    }

    async fn parse_resolve_did_response(&self, response: &str) -> IndyResult<String> {
        trace!(
            "parse_resolve_did_response > response {:?}",
            response
        );
        self.ledger_service.cheqd_parse_query_get_did_resp(response)
    }

    async fn build_schema_request(&self, txn_params: &str, submitter_did: &str, endorser: Option<&str>) -> IndyResult<(Vec<u8>, Vec<u8>)> {
        trace!(
            "build_schema_request > submitter_did {:?} txn_params {:?} endorser {:?}",
            submitter_did, txn_params, endorser
        );
        let txn_params: MsgCreateSchema = serde_json::from_str(txn_params)
            .map_err(|err| err_msg(
                IndyErrorKind::InvalidStructure,
                format!("Unable to parse Cheqd Schema TXN params from JSON: {}. Err: {:?}", txn_params, err),
            ))?;

        let message = self.ledger_service.cheqd_build_msg_create_schema(&submitter_did, txn_params)?;
        Ok((message.clone(), message))
    }

    async fn build_resolve_schema_request(&self, id: &str) -> IndyResult<String> {
        trace!(
            "build_resolve_schema_request > id {:?}",
            id
        );
        self.ledger_service.cheqd_build_query_get_schema(id)
    }

    async fn parse_resolve_schema_response(&self, response: &str) -> IndyResult<String> {
        trace!(
            "parse_resolve_schema_response > response {:?}",
            response
        );
        self.ledger_service.cheqd_parse_query_get_schema_resp(response)
    }

    async fn build_cred_def_request(&self, txn_params: &str, submitter_did: &str, endorser: Option<&str>) -> IndyResult<(Vec<u8>, Vec<u8>)> {
        trace!(
            "build_cred_def_request > submitter_did {:?} txn_params {:?} endorser {:?}",
            submitter_did, txn_params, endorser
        );
        let txn_params: MsgCreateCredDef = serde_json::from_str(txn_params)
            .map_err(|err| err_msg(
                IndyErrorKind::InvalidStructure,
                format!("Unable to parse Cheqd CredDef TXN params from JSON: {}. Err: {:?}", txn_params, err),
            ))?;

        let message = self.ledger_service.cheqd_build_msg_create_cred_def(&submitter_did, txn_params)?;
        Ok((message.clone(), message))
    }

    async fn build_resolve_cred_def_request(&self, id: &str) -> IndyResult<String> {
        trace!(
            "build_resolve_cred_def_request > id {:?}",
            id
        );
        self.ledger_service.cheqd_build_query_get_cred_def(id)
    }

    async fn parse_resolve_cred_def_response(&self, response: &str) -> IndyResult<String> {
        trace!(
            "parse_resolve_cred_def_response > response {:?}",
            response
        );
        self.ledger_service.cheqd_parse_query_get_cred_def_resp(response)
    }

    fn prepare_endorsement_spec(&self, endorser: Option<&str>) -> IndyResult<Option<EndorsementSpec>> {
        trace!(
            "prepare_endorsement_spec > endorser {:?}",
            endorser
        );
        Ok(Some(EndorsementSpec::Cheqd(CheqdEndorsementSpec {})))
    }
}

impl CheqdLedger {
    async fn prepare_txn(&self, tx_bytes: &[u8], signature: &[u8], endorsement: Option<&str>) -> IndyResult<Vec<u8>> {
        trace!(
            "prepare_txn > tx_bytes {:?} signature {:?} endorsement {:?}",
            tx_bytes, signature, endorsement
        );

        let (doc, signature): (SignDoc, Vec<u8>) = {
            if let Some (endorsement) = endorsement {
                // tx_bytes contains internal message -- we need to build Cheqd transaction using endorsement data
                self.prepare_txn_with_endorsement(tx_bytes, signature, endorsement).await?
            } else {
                let doc = SignDoc::from_bytes(tx_bytes)
                    .to_indy(IndyErrorKind::InvalidStructure, "Endorsement information need to be provided for submitting of transactions.")?;
                // tx_bytes already contains Cheqd transaction
                (doc, signature.to_vec())
            }
        };

        self.ledger_service.build_signed_txn(doc, signature)
    }

    async fn prepare_txn_with_endorsement(&self, tx_bytes: &[u8], signature: &[u8], endorsement: &str) -> IndyResult<(SignDoc, Vec<u8>)> {
        let endorsement: CheqdEndorsement = serde_json::from_str(endorsement)
            .map_err(|err| err_msg(
                IndyErrorKind::InvalidStructure,
                format!("Unable to parse Cheqd Endorsement from JSON. Err: {:?}", err),
            ))?;

        let tx_bytes = self.ledger_service.build_signed_message(tx_bytes, &endorsement.txn_author_did, signature)?;

        let (doc, _) =
            self.ledger_service
                .auth_build_tx(
                    &endorsement.chain_id,
                    &endorsement.public_key,
                    &tx_bytes,
                    endorsement.account_number,
                    endorsement.sequence_number,
                    endorsement.max_gas,
                    endorsement.max_coin_amount,
                    &endorsement.max_coin_denom,
                    &endorsement.account_id,
                    endorsement.timeout_height,
                    &endorsement.memo,
                )
                .await?;

        let signature = base64::decode(&endorsement.signature)
            .map_err(|_| {
                err_msg(IndyErrorKind::InvalidState, "Invalid base64 bytes for endorser signature")
            })?;

        Ok((doc, signature))
    }
}
