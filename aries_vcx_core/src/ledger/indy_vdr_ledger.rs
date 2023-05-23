use indy_ledger_response_parser::{ResponseParser, RevocationRegistryDeltaInfo, RevocationRegistryInfo};
use indy_vdr as vdr;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use time::OffsetDateTime;
use vdr::ledger::requests::cred_def::CredentialDefinitionV1;
use vdr::ledger::requests::rev_reg::{RevocationRegistryDelta, RevocationRegistryDeltaV1};
use vdr::ledger::requests::rev_reg_def::{RegistryType, RevocationRegistryDefinition, RevocationRegistryDefinitionV1};
use vdr::ledger::requests::schema::{Schema, SchemaV1};

use async_trait::async_trait;
use serde_json::Value;
use vdr::ledger::identifiers::{CredentialDefinitionId, RevocationRegistryId, SchemaId};
use vdr::ledger::requests::{author_agreement::TxnAuthrAgrmtAcceptanceData, cred_def::CredentialDefinition};
use vdr::ledger::RequestBuilder;
use vdr::pool::{LedgerType, PreparedRequest, ProtocolVersion};
use vdr::utils::did::DidValue;
use vdr::utils::Qualifiable;

use crate::common::ledger::transactions::verify_transaction_can_be_endorsed;
use crate::errors::error::VcxCoreResult;
use crate::global::author_agreement::get_txn_author_agreement;
use crate::global::settings;

use super::base_ledger::BaseLedger;
use super::request_signer::RequestSigner;
use super::request_submitter::RequestSubmitter;
use super::response_cacher::ResponseCacher;

pub struct IndyVdrLedgerConfig<T, U, V>
where
    T: RequestSubmitter + Send + Sync,
    U: RequestSigner + Send + Sync,
    V: ResponseCacher + Send + Sync,
{
    pub request_signer: Arc<U>,
    pub request_submitter: Arc<T>,
    pub response_parser: Arc<ResponseParser>,
    pub response_cacher: Arc<V>,
}

pub struct IndyVdrLedger<T, U, V>
where
    T: RequestSubmitter + Send + Sync,
    U: RequestSigner + Send + Sync,
    V: ResponseCacher + Send + Sync,
{
    request_signer: Arc<U>,
    request_submitter: Arc<T>,
    response_parser: Arc<ResponseParser>,
    response_cacher: Arc<V>,
}

impl<T, U, V> IndyVdrLedger<T, U, V>
where
    T: RequestSubmitter + Send + Sync,
    U: RequestSigner + Send + Sync,
    V: ResponseCacher + Send + Sync,
{
    pub fn new(config: IndyVdrLedgerConfig<T, U, V>) -> Self {
        Self {
            request_signer: config.request_signer,
            request_submitter: config.request_submitter,
            response_parser: config.response_parser,
            response_cacher: config.response_cacher,
        }
    }

    pub fn request_builder(&self) -> VcxCoreResult<RequestBuilder> {
        // TODO - confirm correct protocol version?
        let v = settings::get_protocol_version();
        let version = ProtocolVersion::from_id(v as u64)?;
        Ok(RequestBuilder::new(version))
    }

    async fn _submit_request_cached(&self, id: &str, request: PreparedRequest) -> VcxCoreResult<String> {
        match self.response_cacher.get(id, None).await? {
            Some(response) => Ok(response),
            None => {
                let response = self.request_submitter.submit(request).await?;
                self.response_cacher.put(id, response.clone()).await?;
                Ok(response)
            }
        }
    }

    async fn _submit_request(&self, request: PreparedRequest) -> VcxCoreResult<String> {
        self.request_submitter.submit(request).await
    }

    async fn _get_request_signature(&self, did: &str, request: &PreparedRequest) -> VcxCoreResult<Vec<u8>> {
        self.request_signer.sign(did, request).await
    }

    async fn _sign_and_submit_request(&self, submitter_did: &str, request: PreparedRequest) -> VcxCoreResult<String> {
        let mut request = request;
        let signature = self._get_request_signature(submitter_did, &request).await?;
        request.set_signature(&signature)?;
        self._submit_request(request).await
    }

    async fn _build_get_cred_def_request(
        &self,
        submitter_did: Option<&str>,
        cred_def_id: &str,
    ) -> VcxCoreResult<PreparedRequest> {
        let identifier = submitter_did.map(DidValue::from_str).transpose()?;
        let id = CredentialDefinitionId::from_str(cred_def_id)?;
        Ok(self
            .request_builder()?
            .build_get_cred_def_request(identifier.as_ref(), &id)?)
    }

    async fn _build_get_attr_request(
        &self,
        submitter_did: Option<&str>,
        target_did: &str,
        attribute_name: &str,
    ) -> VcxCoreResult<PreparedRequest> {
        let identifier = if let Some(did) = submitter_did {
            Some(DidValue::from_str(did)?)
        } else {
            None
        };
        let dest = DidValue::from_str(target_did)?;
        Ok(self.request_builder()?.build_get_attrib_request(
            identifier.as_ref(),
            &dest,
            Some(attribute_name.to_string()),
            None,
            None,
        )?)
    }

    fn _build_attrib_request(
        &self,
        submitter_did: &str,
        target_did: &str,
        attrib_json_str: Option<&str>,
    ) -> VcxCoreResult<PreparedRequest> {
        let identifier = DidValue::from_str(submitter_did)?;
        let dest = DidValue::from_str(target_did)?;
        let attrib_json = if let Some(attrib) = attrib_json_str {
            Some(serde_json::from_str::<Value>(attrib)?)
        } else {
            None
        };

        Ok(self
            .request_builder()?
            .build_attrib_request(&identifier, &dest, None, attrib_json.as_ref(), None)?)
    }

    fn _build_schema_request(&self, submitter_did: &str, schema_data: &str) -> VcxCoreResult<PreparedRequest> {
        let identifier = DidValue::from_str(submitter_did)?;
        let schema_data: SchemaV1 = serde_json::from_str(schema_data)?;
        Ok(self
            .request_builder()?
            .build_schema_request(&identifier, Schema::SchemaV1(schema_data))?)
    }

    fn _build_cred_def_request(&self, submitter_did: &str, cred_def_data: &str) -> VcxCoreResult<PreparedRequest> {
        let identifier = DidValue::from_str(submitter_did)?;
        let cred_def_data: CredentialDefinitionV1 = serde_json::from_str(cred_def_data)?;
        Ok(self
            .request_builder()?
            .build_cred_def_request(&identifier, CredentialDefinition::CredentialDefinitionV1(cred_def_data))?)
    }

    fn _build_rev_reg_def_request(
        &self,
        submitter_did: &str,
        rev_reg_def_data: &str,
    ) -> VcxCoreResult<PreparedRequest> {
        let identifier = DidValue::from_str(submitter_did)?;
        let rev_reg_def_data: RevocationRegistryDefinitionV1 = serde_json::from_str(rev_reg_def_data)?;
        Ok(self.request_builder()?.build_revoc_reg_def_request(
            &identifier,
            RevocationRegistryDefinition::RevocationRegistryDefinitionV1(rev_reg_def_data),
        )?)
    }

    fn _build_rev_reg_delta_request(
        &self,
        submitter_did: &str,
        rev_reg_id: &str,
        rev_reg_delta_data: &str,
    ) -> VcxCoreResult<PreparedRequest> {
        let identifier = DidValue::from_str(submitter_did)?;
        let rev_reg_delta_data: RevocationRegistryDeltaV1 = serde_json::from_str(rev_reg_delta_data)?;
        Ok(self.request_builder()?.build_revoc_reg_entry_request(
            &identifier,
            &RevocationRegistryId::from_str(rev_reg_id)?,
            &RegistryType::CL_ACCUM,
            RevocationRegistryDelta::RevocationRegistryDeltaV1(rev_reg_delta_data),
        )?)
    }

    fn _build_get_txn_request(&self, submitter_did: Option<&str>, seq_no: i32) -> VcxCoreResult<PreparedRequest> {
        let identifier = submitter_did.map(DidValue::from_str).transpose()?;
        Ok(self
            .request_builder()?
            .build_get_txn_request(identifier.as_ref(), LedgerType::DOMAIN.to_id(), seq_no)?)
    }

    fn _build_taa_request(&self, submitter_did: Option<&str>) -> VcxCoreResult<PreparedRequest> {
        let submitter_did = submitter_did.map(DidValue::from_str).transpose()?;
        Ok(self
            .request_builder()?
            .build_get_txn_author_agreement_request(submitter_did.as_ref(), None)?)
    }
}

impl<T, U, V> Debug for IndyVdrLedger<T, U, V>
where
    T: RequestSubmitter + Send + Sync,
    U: RequestSigner + Send + Sync,
    V: ResponseCacher + Send + Sync,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "IndyVdrLedger instance")
    }
}

#[async_trait]
impl<T, U, V> BaseLedger for IndyVdrLedger<T, U, V>
where
    T: RequestSubmitter + Send + Sync,
    U: RequestSigner + Send + Sync,
    V: ResponseCacher + Send + Sync,
{
    async fn sign_and_submit_request(&self, submitter_did: &str, request_json: &str) -> VcxCoreResult<String> {
        let request = PreparedRequest::from_request_json(request_json)?;

        self._sign_and_submit_request(submitter_did, request).await
    }

    async fn submit_request(&self, request_json: &str) -> VcxCoreResult<String> {
        let request = PreparedRequest::from_request_json(request_json)?;
        self._submit_request(request).await
    }

    async fn endorse_transaction(&self, endorser_did: &str, request_json: &str) -> VcxCoreResult<()> {
        let mut request = PreparedRequest::from_request_json(&request_json)?;
        verify_transaction_can_be_endorsed(request_json, endorser_did)?;
        let signature_endorser = self._get_request_signature(endorser_did, &request).await?;
        request.set_multi_signature(&DidValue::from_str(endorser_did)?, &signature_endorser)?;
        self._submit_request(request).await.map(|_| ())
    }

    async fn set_endorser(&self, submitter_did: &str, request_json: &str, endorser: &str) -> VcxCoreResult<String> {
        let mut request = PreparedRequest::from_request_json(request_json)?;
        request.set_endorser(&DidValue::from_str(endorser)?)?;
        let signature_submitter = self._get_request_signature(submitter_did, &request).await?;
        request.set_multi_signature(&DidValue::from_str(submitter_did)?, &signature_submitter)?;
        Ok(request.req_json.to_string())
    }

    async fn get_txn_author_agreement(&self) -> VcxCoreResult<String> {
        let request = self
            .request_builder()?
            .build_get_txn_author_agreement_request(None, None)?;
        self._submit_request(request).await
    }

    async fn get_nym(&self, did: &str) -> VcxCoreResult<String> {
        let dest = DidValue::from_str(did)?;
        let request = self.request_builder()?.build_get_nym_request(None, &dest)?;

        self._submit_request_cached(did, request).await
    }

    async fn publish_nym(
        &self,
        submitter_did: &str,
        target_did: &str,
        verkey: Option<&str>,
        alias: Option<&str>,
        role: Option<&str>,
    ) -> VcxCoreResult<String> {
        let identifier = DidValue::from_str(submitter_did)?;
        let dest = DidValue::from_str(target_did)?;
        let request = self.request_builder()?.build_nym_request(
            &identifier,
            &dest,
            verkey.map(String::from),
            alias.map(String::from),
            role.map(String::from),
        )?;

        self._sign_and_submit_request(submitter_did, request).await
    }

    async fn get_schema(&self, schema_id: &str, _submitter_did: Option<&str>) -> VcxCoreResult<String> {
        let request = self
            .request_builder()?
            .build_get_schema_request(None, &SchemaId::from_str(schema_id)?)?;
        let response = self._submit_request_cached(schema_id, request).await?;
        let schema = self.response_parser.parse_get_schema_response(&response, None)?;
        Ok(serde_json::to_string(&schema)?)
    }

    async fn get_cred_def(&self, cred_def_id: &str, submitter_did: Option<&str>) -> VcxCoreResult<String> {
        let request = self._build_get_cred_def_request(submitter_did, cred_def_id).await?;
        let response = self._submit_request(request).await?;
        let cred_def = self.response_parser.parse_get_cred_def_response(&response, None)?;
        Ok(serde_json::to_string(&cred_def)?)
    }

    async fn get_attr(&self, target_did: &str, attr_name: &str) -> VcxCoreResult<String> {
        let request = self._build_get_attr_request(None, target_did, attr_name).await?;

        self._submit_request(request).await
    }

    async fn add_attr(&self, target_did: &str, attrib_json: &str) -> VcxCoreResult<String> {
        let request = self._build_attrib_request(target_did, target_did, Some(attrib_json))?;
        let request = _append_txn_author_agreement_to_request(request).await?;

        self._sign_and_submit_request(target_did, request).await
    }

    async fn get_rev_reg_def_json(&self, rev_reg_id: &str) -> VcxCoreResult<String> {
        let id = RevocationRegistryId::from_str(rev_reg_id)?;
        let request = self.request_builder()?.build_get_revoc_reg_def_request(None, &id)?;
        let res = self._submit_request_cached(rev_reg_id, request).await?;

        let rev_reg_def = self.response_parser.parse_get_revoc_reg_def_response(&res)?;

        Ok(serde_json::to_string(&rev_reg_def)?)
    }

    async fn get_rev_reg_delta_json(
        &self,
        rev_reg_id: &str,
        from: Option<u64>,
        to: Option<u64>,
    ) -> VcxCoreResult<(String, String, u64)> {
        let revoc_reg_def_id = RevocationRegistryId::from_str(rev_reg_id)?;

        let from = from.map(|x| x as i64);
        let current_time = current_epoch_time();
        let to = to.map_or(current_time, |x| x as i64);

        let request = self
            .request_builder()?
            .build_get_revoc_reg_delta_request(None, &revoc_reg_def_id, from, to)?;
        let res = self._submit_request(request).await?;

        let RevocationRegistryDeltaInfo {
            revoc_reg_def_id,
            revoc_reg_delta,
            timestamp,
        } = self.response_parser.parse_get_revoc_reg_delta_response(&res)?;

        Ok((
            revoc_reg_def_id.to_string(),
            serde_json::to_string(&revoc_reg_delta)?,
            timestamp,
        ))
    }

    async fn get_rev_reg(&self, rev_reg_id: &str, timestamp: u64) -> VcxCoreResult<(String, String, u64)> {
        let revoc_reg_def_id = RevocationRegistryId::from_str(rev_reg_id)?;

        let request = self.request_builder()?.build_get_revoc_reg_request(
            None,
            &revoc_reg_def_id,
            timestamp.try_into().unwrap(),
        )?;
        let res = self._submit_request(request).await?;

        let RevocationRegistryInfo {
            revoc_reg_def_id,
            revoc_reg,
            timestamp,
        } = self.response_parser.parse_get_revoc_reg_response(&res)?;

        Ok((
            revoc_reg_def_id.to_string(),
            serde_json::to_string(&revoc_reg)?,
            timestamp,
        ))
    }

    async fn get_ledger_txn(&self, seq_no: i32, submitter_did: Option<&str>) -> VcxCoreResult<String> {
        let request = self._build_get_txn_request(submitter_did, seq_no)?;
        self._submit_request(request).await
    }

    async fn build_schema_request(&self, submitter_did: &str, schema_json: &str) -> VcxCoreResult<String> {
        let request = self._build_schema_request(submitter_did, schema_json)?;
        let request = _append_txn_author_agreement_to_request(request).await?;
        Ok(request.req_json.to_string())
    }

    async fn publish_schema(
        &self,
        schema_json: &str,
        submitter_did: &str,
        endorser_did: Option<String>,
    ) -> VcxCoreResult<()> {
        let mut request = self._build_schema_request(submitter_did, schema_json)?;
        request = _append_txn_author_agreement_to_request(request).await?;
        if let Some(endorser_did) = endorser_did {
            request = PreparedRequest::from_request_json(
                self.set_endorser(submitter_did, &request.req_json.to_string(), &endorser_did)
                    .await?,
            )?
        }
        self._sign_and_submit_request(submitter_did, request).await.map(|_| ())
    }

    async fn publish_cred_def(&self, cred_def_json: &str, submitter_did: &str) -> VcxCoreResult<()> {
        let request = self._build_cred_def_request(submitter_did, cred_def_json)?;
        let request = _append_txn_author_agreement_to_request(request).await?;
        self._sign_and_submit_request(submitter_did, request).await.map(|_| ())
    }

    async fn publish_rev_reg_def(&self, rev_reg_def: &str, submitter_did: &str) -> VcxCoreResult<()> {
        let request = self._build_rev_reg_def_request(submitter_did, rev_reg_def)?;
        let request = _append_txn_author_agreement_to_request(request).await?;
        self._sign_and_submit_request(submitter_did, request).await.map(|_| ())
    }

    async fn publish_rev_reg_delta(
        &self,
        rev_reg_id: &str,
        rev_reg_entry_json: &str,
        submitter_did: &str,
    ) -> VcxCoreResult<()> {
        let request = self._build_rev_reg_delta_request(submitter_did, rev_reg_id, rev_reg_entry_json)?;
        let request = _append_txn_author_agreement_to_request(request).await?;
        self._sign_and_submit_request(submitter_did, request).await.map(|_| ())
    }
}

fn current_epoch_time() -> i64 {
    OffsetDateTime::now_utc().unix_timestamp() as i64
}

async fn _append_txn_author_agreement_to_request(request: PreparedRequest) -> VcxCoreResult<PreparedRequest> {
    if let Some(taa) = get_txn_author_agreement()? {
        let mut request = request;
        let acceptance = TxnAuthrAgrmtAcceptanceData {
            mechanism: taa.acceptance_mechanism_type,
            // TODO - investigate default digest
            taa_digest: taa.taa_digest.map_or(String::from(""), |v| v),
            time: taa.time_of_acceptance,
        };
        request.set_txn_author_agreement_acceptance(&acceptance)?;

        Ok(request)
    } else {
        Ok(request)
    }
}

// #[cfg(test)]
// mod unit_tests {
//     use std::sync::Arc;

//     use crate::errors::error::{AriesVcxCoreErrorKind, VcxCoreResult};
//     use crate::{
//         common::test_utils::mock_profile,
//         plugins::ledger::{base_ledger::BaseLedger, indy_vdr_ledger::IndyVdrLedgerPool},
//     };

//     use super::IndyVdrLedger;

//     #[tokio::test]
//     #[ignore]
//     async fn test_pool_unimplemented_methods() {
//         // test used to assert which methods are unimplemented currently, can be removed after all methods implemented

//         fn assert_unimplemented<T: std::fmt::Debug>(result: VcxCoreResult<T>) {
//             assert_eq!(result.unwrap_err().kind(), AriesVcxCoreErrorKind::UnimplementedFeature)
//         }

//         let profile = mock_profile();
//         let pool = Arc::new(IndyVdrLedgerPool { runner: None });
//         let ledger: Box<dyn BaseLedger> = Box::new(IndyVdrLedger::new(profile.inject_wallet(), pool));

//         assert_unimplemented(ledger.endorse_transaction("", "").await);
//         assert_unimplemented(ledger.set_endorser("", "", "").await);
//         assert_unimplemented(ledger.get_txn_author_agreement().await);
//         assert_unimplemented(ledger.get_rev_reg("", 0).await);
//         assert_unimplemented(ledger.get_ledger_txn(0, None).await);
//         assert_unimplemented(ledger.build_schema_request("", "").await);
//         assert_unimplemented(ledger.publish_schema("", "", None).await);
//         assert_unimplemented(ledger.publish_cred_def("", "").await);
//         assert_unimplemented(ledger.publish_rev_reg_def("", "").await);
//     }
// }
