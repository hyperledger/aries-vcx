pub use indy_ledger_response_parser::GetTxnAuthorAgreementData;
use indy_ledger_response_parser::{ResponseParser, RevocationRegistryDeltaInfo, RevocationRegistryInfo};
use indy_vdr as vdr;
use std::fmt::{Debug, Formatter};
use std::sync::{Arc, RwLock};
use time::OffsetDateTime;
use vdr::ledger::requests::cred_def::CredentialDefinitionV1;
use vdr::ledger::requests::rev_reg::{RevocationRegistryDelta, RevocationRegistryDeltaV1};
use vdr::ledger::requests::rev_reg_def::{RegistryType, RevocationRegistryDefinition, RevocationRegistryDefinitionV1};
use vdr::ledger::requests::schema::{Schema, SchemaV1};

use async_trait::async_trait;
use serde_json::Value;
use vdr::ledger::identifiers::{CredentialDefinitionId, RevocationRegistryId, SchemaId};
use vdr::ledger::requests::cred_def::CredentialDefinition;
use vdr::ledger::RequestBuilder;
use vdr::pool::{LedgerType, PreparedRequest, ProtocolVersion as VdrProtocolVersion};
use vdr::utils::did::DidValue;
use vdr::utils::Qualifiable;

use crate::ledger::common::verify_transaction_can_be_endorsed;
use crate::errors::error::{AriesVcxCoreError, AriesVcxCoreErrorKind, VcxCoreResult};
use crate::ledger::base_ledger::{TaaConfigurator, TxnAuthrAgrmtOptions};

use super::base_ledger::{AnoncredsLedgerRead, AnoncredsLedgerWrite, IndyLedgerRead, IndyLedgerWrite};
use super::map_error_not_found_to_none;
use super::request_signer::RequestSigner;
use super::request_submitter::RequestSubmitter;
use super::response_cacher::ResponseCacher;

// TODO: Should implement builders for these configs...
// Good first issue?
pub struct IndyVdrLedgerReadConfig<T, V>
where
    T: RequestSubmitter + Send + Sync,
    V: ResponseCacher + Send + Sync,
{
    pub request_submitter: Arc<T>,
    pub response_parser: Arc<ResponseParser>,
    pub response_cacher: Arc<V>,
    pub protocol_version: ProtocolVersion,
}

pub struct IndyVdrLedgerWriteConfig<T, U>
where
    T: RequestSubmitter + Send + Sync,
    U: RequestSigner + Send + Sync,
{
    pub request_signer: Arc<U>,
    pub request_submitter: Arc<T>,
    pub taa_options: Option<TxnAuthrAgrmtOptions>,
    pub protocol_version: ProtocolVersion,
}

pub struct IndyVdrLedgerRead<T, V>
where
    T: RequestSubmitter + Send + Sync,
    V: ResponseCacher + Send + Sync,
{
    request_submitter: Arc<T>,
    response_parser: Arc<ResponseParser>,
    response_cacher: Arc<V>,
    protocol_version: ProtocolVersion,
}

pub struct IndyVdrLedgerWrite<T, U>
where
    T: RequestSubmitter + Send + Sync,
    U: RequestSigner + Send + Sync,
{
    request_signer: Arc<U>,
    request_submitter: Arc<T>,
    taa_options: RwLock<Option<TxnAuthrAgrmtOptions>>,
    protocol_version: ProtocolVersion,
}

impl<T, V> IndyVdrLedgerRead<T, V>
where
    T: RequestSubmitter + Send + Sync,
    V: ResponseCacher + Send + Sync,
{
    pub fn new(config: IndyVdrLedgerReadConfig<T, V>) -> Self {
        Self {
            request_submitter: config.request_submitter,
            response_parser: config.response_parser,
            response_cacher: config.response_cacher,
            protocol_version: config.protocol_version,
        }
    }

    pub fn request_builder(&self) -> VcxCoreResult<RequestBuilder> {
        Ok(RequestBuilder::new(self.protocol_version.0))
    }

    async fn submit_request_cached(&self, id: &str, request: PreparedRequest) -> VcxCoreResult<String> {
        match self.response_cacher.get(id, None).await? {
            Some(response) => Ok(response),
            None => {
                let response = self.request_submitter.submit(request).await?;
                self.response_cacher.put(id, response.clone()).await?;
                Ok(response)
            }
        }
    }
}

impl<T, U> IndyVdrLedgerWrite<T, U>
where
    T: RequestSubmitter + Send + Sync,
    U: RequestSigner + Send + Sync,
{
    pub fn new(config: IndyVdrLedgerWriteConfig<T, U>) -> Self {
        Self {
            request_signer: config.request_signer,
            request_submitter: config.request_submitter,
            taa_options: RwLock::new(None),
            protocol_version: config.protocol_version,
        }
    }

    pub fn request_builder(&self) -> VcxCoreResult<RequestBuilder> {
        Ok(RequestBuilder::new(self.protocol_version.0))
    }

    async fn sign_and_submit_request(&self, submitter_did: &str, request: PreparedRequest) -> VcxCoreResult<String> {
        let mut request = request;
        let signature = self.request_signer.sign(submitter_did, &request).await?;
        request.set_signature(&signature)?;
        self.request_submitter.submit(request).await
    }
}

impl<T, U> TaaConfigurator for IndyVdrLedgerWrite<T, U>
where
    T: RequestSubmitter + Send + Sync,
    U: RequestSigner + Send + Sync,
{
    fn set_txn_author_agreement_options(&self, taa_options: TxnAuthrAgrmtOptions) -> VcxCoreResult<()> {
        let mut m = self.taa_options.write()?;
        *m = Some(taa_options);
        Ok(())
    }
}

impl<T, V> Debug for IndyVdrLedgerRead<T, V>
where
    T: RequestSubmitter + Send + Sync,
    V: ResponseCacher + Send + Sync,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "IndyVdrLedgerRead instance")
    }
}

impl<T, U> Debug for IndyVdrLedgerWrite<T, U>
where
    T: RequestSubmitter + Send + Sync,
    U: RequestSigner + Send + Sync,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "IndyVdrLedgerWrite instance")
    }
}

#[async_trait]
impl<T, V> IndyLedgerRead for IndyVdrLedgerRead<T, V>
where
    T: RequestSubmitter + Send + Sync,
    V: ResponseCacher + Send + Sync,
{
    async fn get_attr(&self, target_did: &str, attr_name: &str) -> VcxCoreResult<String> {
        let dest = DidValue::from_str(target_did)?;
        let request =
            self.request_builder()?
                .build_get_attrib_request(None, &dest, Some(attr_name.to_string()), None, None)?;
        self.request_submitter.submit(request).await
    }

    async fn get_nym(&self, did: &str) -> VcxCoreResult<String> {
        let dest = DidValue::from_str(did)?;
        let request = self.request_builder()?.build_get_nym_request(None, &dest)?;
        self.submit_request_cached(did, request).await
    }

    async fn get_txn_author_agreement(&self) -> VcxCoreResult<Option<String>> {
        let request = self
            .request_builder()?
            .build_get_txn_author_agreement_request(None, None)?;
        let response = self.request_submitter.submit(request).await?;
        map_error_not_found_to_none(self.response_parser.parse_get_txn_author_agreement_response(&response))?
            .map(|taa| serde_json::to_string(&taa).map_err(Into::into))
            .transpose()
    }

    async fn get_ledger_txn(&self, seq_no: i32, submitter_did: Option<&str>) -> VcxCoreResult<String> {
        let identifier = submitter_did.map(DidValue::from_str).transpose()?;
        let request =
            self.request_builder()?
                .build_get_txn_request(identifier.as_ref(), LedgerType::DOMAIN.to_id(), seq_no)?;
        self.request_submitter.submit(request).await
    }
}

impl<T, U> IndyVdrLedgerWrite<T, U>
where
    T: RequestSubmitter + Send + Sync,
    U: RequestSigner + Send + Sync,
{
    async fn append_txn_author_agreement_to_request(&self, request: PreparedRequest) -> VcxCoreResult<PreparedRequest> {
        let taa_options = (*self.taa_options.read()?).clone();
        if let Some(taa_options) = taa_options {
            let mut request = request;
            let taa_data = self.request_builder()?.prepare_txn_author_agreement_acceptance_data(
                Some(&taa_options.text),
                Some(&taa_options.version),
                None,
                &taa_options.aml_label,
                OffsetDateTime::now_utc().unix_timestamp() as u64,
            )?;
            request.set_txn_author_agreement_acceptance(&taa_data)?;
            Ok(request)
        } else {
            Ok(request)
        }
    }
}

#[async_trait]
impl<T, U> IndyLedgerWrite for IndyVdrLedgerWrite<T, U>
where
    T: RequestSubmitter + Send + Sync,
    U: RequestSigner + Send + Sync,
{
    async fn publish_nym(
        &self,
        submitter_did: &str,
        target_did: &str,
        verkey: Option<&str>,
        data: Option<&str>,
        role: Option<&str>,
    ) -> VcxCoreResult<String> {
        let identifier = DidValue::from_str(submitter_did)?;
        let dest = DidValue::from_str(target_did)?;
        let request = self.request_builder()?.build_nym_request(
            &identifier,
            &dest,
            verkey.map(String::from),
            data.map(String::from),
            role.map(String::from),
        )?;
        let request = self.append_txn_author_agreement_to_request(request).await?;
        self.sign_and_submit_request(submitter_did, request).await
    }

    async fn set_endorser(&self, submitter_did: &str, request_json: &str, endorser: &str) -> VcxCoreResult<String> {
        let mut request = PreparedRequest::from_request_json(request_json)?;
        request.set_endorser(&DidValue::from_str(endorser)?)?;
        let signature_submitter = self.request_signer.sign(submitter_did, &request).await?;
        request.set_multi_signature(&DidValue::from_str(submitter_did)?, &signature_submitter)?;
        Ok(request.req_json.to_string())
    }

    async fn endorse_transaction(&self, endorser_did: &str, request_json: &str) -> VcxCoreResult<()> {
        let mut request = PreparedRequest::from_request_json(&request_json)?;
        verify_transaction_can_be_endorsed(request_json, endorser_did)?;
        let signature_endorser = self.request_signer.sign(endorser_did, &request).await?;
        request.set_multi_signature(&DidValue::from_str(endorser_did)?, &signature_endorser)?;
        self.request_submitter.submit(request).await.map(|_| ())
    }

    async fn add_attr(&self, target_did: &str, attrib_json: &str) -> VcxCoreResult<String> {
        let identifier = DidValue::from_str(target_did)?;
        let dest = DidValue::from_str(target_did)?;
        let request = self.request_builder()?.build_attrib_request(
            &identifier,
            &dest,
            None,
            Some(&serde_json::from_str::<Value>(attrib_json)?),
            None,
        )?;
        let request = self.append_txn_author_agreement_to_request(request).await?;
        self.sign_and_submit_request(target_did, request).await
    }
}

#[async_trait]
impl<T, V> AnoncredsLedgerRead for IndyVdrLedgerRead<T, V>
where
    T: RequestSubmitter + Send + Sync,
    V: ResponseCacher + Send + Sync,
{
    async fn get_schema(&self, schema_id: &str, _submitter_did: Option<&str>) -> VcxCoreResult<String> {
        let request = self
            .request_builder()?
            .build_get_schema_request(None, &SchemaId::from_str(schema_id)?)?;
        let response = self.submit_request_cached(schema_id, request).await?;
        let schema = self.response_parser.parse_get_schema_response(&response, None)?;
        Ok(serde_json::to_string(&schema)?)
    }

    async fn get_cred_def(&self, cred_def_id: &str, submitter_did: Option<&str>) -> VcxCoreResult<String> {
        let identifier = submitter_did.map(DidValue::from_str).transpose()?;
        let id = CredentialDefinitionId::from_str(cred_def_id)?;
        let request = self
            .request_builder()?
            .build_get_cred_def_request(identifier.as_ref(), &id)?;
        let response = self.request_submitter.submit(request).await?;
        let cred_def = self.response_parser.parse_get_cred_def_response(&response, None)?;
        Ok(serde_json::to_string(&cred_def)?)
    }

    async fn get_rev_reg_def_json(&self, rev_reg_id: &str) -> VcxCoreResult<String> {
        let id = RevocationRegistryId::from_str(rev_reg_id)?;
        let request = self.request_builder()?.build_get_revoc_reg_def_request(None, &id)?;
        let res = self.submit_request_cached(rev_reg_id, request).await?;
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
        let current_time = OffsetDateTime::now_utc().unix_timestamp() as i64;
        let to = to.map_or(current_time, |x| x as i64);

        let request = self
            .request_builder()?
            .build_get_revoc_reg_delta_request(None, &revoc_reg_def_id, from, to)?;
        let res = self.request_submitter.submit(request).await?;

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
        let res = self.request_submitter.submit(request).await?;

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
}

#[async_trait]
impl<T, U> AnoncredsLedgerWrite for IndyVdrLedgerWrite<T, U>
where
    T: RequestSubmitter + Send + Sync,
    U: RequestSigner + Send + Sync,
{
    async fn publish_schema(
        &self,
        schema_json: &str,
        submitter_did: &str,
        endorser_did: Option<String>,
    ) -> VcxCoreResult<()> {
        let identifier = DidValue::from_str(submitter_did)?;
        let schema_data: SchemaV1 = serde_json::from_str(schema_json)?;
        let mut request = self
            .request_builder()?
            .build_schema_request(&identifier, Schema::SchemaV1(schema_data))?;
        request = self.append_txn_author_agreement_to_request(request).await?;
        // if let Some(endorser_did) = endorser_did {
        //     request = PreparedRequest::from_request_json(
        //         self.set_endorser(submitter_did, &request.req_json.to_string(), &endorser_did)
        //             .await?,
        //     )?
        // }
        let sign_result = self.sign_and_submit_request(submitter_did, request).await;

        if let Err(err) = &sign_result {
            if let AriesVcxCoreErrorKind::InvalidLedgerResponse = err.kind() {
                return Err(AriesVcxCoreError::from_msg(
                    AriesVcxCoreErrorKind::DuplicationSchema,
                    format!("Schema probably already exists, ledger request failed: {:?}", &err),
                ));
            }
        }
        sign_result.map(|_| ())
    }

    async fn publish_cred_def(&self, cred_def_json: &str, submitter_did: &str) -> VcxCoreResult<()> {
        let identifier = DidValue::from_str(submitter_did)?;
        let cred_def_data: CredentialDefinitionV1 = serde_json::from_str(cred_def_json)?;
        let request = self
            .request_builder()?
            .build_cred_def_request(&identifier, CredentialDefinition::CredentialDefinitionV1(cred_def_data))?;
        let request = self.append_txn_author_agreement_to_request(request).await?;
        self.sign_and_submit_request(submitter_did, request).await.map(|_| ())
    }

    async fn publish_rev_reg_def(&self, rev_reg_def: &str, submitter_did: &str) -> VcxCoreResult<()> {
        let identifier = DidValue::from_str(submitter_did)?;
        let rev_reg_def_data: RevocationRegistryDefinitionV1 = serde_json::from_str(rev_reg_def)?;
        let request = self.request_builder()?.build_revoc_reg_def_request(
            &identifier,
            RevocationRegistryDefinition::RevocationRegistryDefinitionV1(rev_reg_def_data),
        )?;
        let request = self.append_txn_author_agreement_to_request(request).await?;
        self.sign_and_submit_request(submitter_did, request).await.map(|_| ())
    }

    async fn publish_rev_reg_delta(
        &self,
        rev_reg_id: &str,
        rev_reg_entry_json: &str,
        submitter_did: &str,
    ) -> VcxCoreResult<()> {
        let identifier = DidValue::from_str(submitter_did)?;
        let rev_reg_delta_data: RevocationRegistryDeltaV1 = serde_json::from_str(rev_reg_entry_json)?;
        let request = self.request_builder()?.build_revoc_reg_entry_request(
            &identifier,
            &RevocationRegistryId::from_str(rev_reg_id)?,
            &RegistryType::CL_ACCUM,
            RevocationRegistryDelta::RevocationRegistryDeltaV1(rev_reg_delta_data),
        )?;
        let request = self.append_txn_author_agreement_to_request(request).await?;
        self.sign_and_submit_request(submitter_did, request).await.map(|_| ())
    }
}

#[derive(Debug)]
pub struct ProtocolVersion(VdrProtocolVersion);

impl Default for ProtocolVersion {
    fn default() -> Self {
        ProtocolVersion(VdrProtocolVersion::Node1_4)
    }
}

impl ProtocolVersion {
    pub fn node_1_3() -> Self {
        ProtocolVersion(VdrProtocolVersion::Node1_3)
    }

    pub fn node_1_4() -> Self {
        ProtocolVersion(VdrProtocolVersion::Node1_4)
    }
}
