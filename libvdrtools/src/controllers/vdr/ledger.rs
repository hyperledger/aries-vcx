use indy_api_types::{errors::*};
use async_trait::async_trait;

use crate::domain::{
    vdr::{
        prepared_txn::EndorsementSpec,
        ledger_types::LedgerTypes,
        ping_status::PingStatus,
    }
};

#[async_trait]
pub(crate) trait Ledger: Send + Sync {
    // meta
    fn name(&self) -> String;
    fn ledger_type(&self) -> LedgerTypes;

    // general
    async fn ping(&self) -> IndyResult<PingStatus>;
    async fn submit_txn(&self, txn_bytes: &[u8], signature: &[u8], endorsement: Option<&str>) -> IndyResult<String>;
    async fn submit_raw_txn(&self, txn_bytes: &[u8]) -> IndyResult<String>;
    async fn submit_query(&self, request: &str) -> IndyResult<String>;
    async fn cleanup(&self) -> IndyResult<()>;

    // did builders + parser
    async fn build_did_request(&self, txn_params: &str, submitter_did: &str, endorser: Option<&str>) -> IndyResult<(Vec<u8>, Vec<u8>)>;
    async fn build_resolve_did_request(&self, id: &str) -> IndyResult<String>;
    async fn parse_resolve_did_response(&self, response: &str) -> IndyResult<String>;

    // schema builders + parser
    async fn build_schema_request(&self, txn_params: &str, submitter_did: &str, endorser: Option<&str>) -> IndyResult<(Vec<u8>, Vec<u8>)>;
    async fn build_resolve_schema_request(&self, id: &str) -> IndyResult<String>;
    async fn parse_resolve_schema_response(&self, response: &str) -> IndyResult<String>;

    // creddef builders + parser
    async fn build_cred_def_request(&self, txn_params: &str, submitter_did: &str, endorser: Option<&str>) -> IndyResult<(Vec<u8>, Vec<u8>)>;
    async fn build_resolve_cred_def_request(&self, id: &str) -> IndyResult<String>;
    async fn parse_resolve_cred_def_response(&self, response: &str) -> IndyResult<String>;

    // endorsement
    fn prepare_endorsement_spec(&self, endorser: Option<&str>) -> IndyResult<Option<EndorsementSpec>>;

}
