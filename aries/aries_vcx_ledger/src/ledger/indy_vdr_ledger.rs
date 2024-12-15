use std::{
    fmt::{Debug, Formatter},
    str::FromStr,
    sync::RwLock,
};

use anoncreds_types::{
    data_types::{
        identifiers::{
            cred_def_id::CredentialDefinitionId, issuer_id::IssuerId,
            rev_reg_def_id::RevocationRegistryDefinitionId, schema_id::SchemaId,
        },
        ledger::{
            cred_def::CredentialDefinition, rev_reg::RevocationRegistry,
            rev_reg_def::RevocationRegistryDefinition, rev_reg_delta::RevocationRegistryDelta,
            rev_status_list::RevocationStatusList, schema::Schema,
        },
    },
    utils::conversions::from_revocation_registry_delta_to_revocation_status_list,
};
use aries_vcx_wallet::wallet::base_wallet::BaseWallet;
use async_trait::async_trait;
use did_parser_nom::Did;
pub use indy_ledger_response_parser::GetTxnAuthorAgreementData;
use indy_ledger_response_parser::{
    ResponseParser, RevocationRegistryDeltaInfo, RevocationRegistryInfo,
};
use indy_vdr::{
    self as vdr,
    ledger::{
        identifiers::SchemaId as IndyVdrSchemaId,
        requests::cred_def::CredentialDefinition as IndyVdrCredentialDefinition,
    },
    utils::{did::DidValue, Validatable},
};
use log::{debug, trace};
use public_key::Key;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use time::OffsetDateTime;
use vdr::{
    config::PoolConfig,
    ledger::{
        identifiers::{
            CredentialDefinitionId as IndyVdrCredentialDefinitionId, RevocationRegistryId,
        },
        requests::rev_reg_def::RegistryType,
        RequestBuilder,
    },
    pool::{LedgerType, PreparedRequest},
    utils::Qualifiable,
};
pub use vdr::{
    ledger::constants::{LedgerRole, UpdateRole},
    pool::ProtocolVersion,
};

use super::{
    base_ledger::{
        AnoncredsLedgerRead, AnoncredsLedgerSupport, AnoncredsLedgerWrite, IndyLedgerRead,
        IndyLedgerWrite,
    },
    map_error_not_found_to_none,
    request_submitter::{
        vdr_ledger::{IndyVdrLedgerPool, IndyVdrSubmitter},
        RequestSubmitter,
    },
    response_cacher::{
        in_memory::{InMemoryResponseCacher, InMemoryResponseCacherConfig},
        ResponseCacher,
    },
};
use crate::{
    errors::error::{VcxLedgerError, VcxLedgerResult},
    ledger::{
        base_ledger::{TaaConfigurator, TxnAuthrAgrmtOptions},
        common::verify_transaction_can_be_endorsed,
        type_conversion::Convert,
    },
};

pub type DefaultIndyLedgerRead = IndyVdrLedgerRead<IndyVdrSubmitter, InMemoryResponseCacher>;
pub type DefaultIndyLedgerWrite = IndyVdrLedgerWrite<IndyVdrSubmitter>;

// TODO: Should implement builders for these configs...
// Good first issue?
pub struct IndyVdrLedgerReadConfig<T, V>
where
    T: RequestSubmitter + Send + Sync,
    V: ResponseCacher + Send + Sync,
{
    pub request_submitter: T,
    pub response_parser: ResponseParser,
    pub response_cacher: V,
    pub protocol_version: ProtocolVersion,
}

pub struct IndyVdrLedgerWriteConfig<T>
where
    T: RequestSubmitter + Send + Sync,
{
    pub request_submitter: T,
    pub taa_options: Option<TxnAuthrAgrmtOptions>,
    pub protocol_version: ProtocolVersion,
}

pub struct IndyVdrLedgerRead<T, V>
where
    T: RequestSubmitter + Send + Sync,
    V: ResponseCacher + Send + Sync,
{
    request_submitter: T,
    response_parser: ResponseParser,
    response_cacher: V,
    protocol_version: ProtocolVersion,
}

pub struct IndyVdrLedgerWrite<T>
where
    T: RequestSubmitter + Send + Sync,
{
    request_submitter: T,
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

    pub fn request_builder(&self) -> VcxLedgerResult<RequestBuilder> {
        Ok(RequestBuilder::new(self.protocol_version))
    }

    async fn submit_request(
        &self,
        cache_id: Option<&str>,
        request: PreparedRequest,
    ) -> VcxLedgerResult<String> {
        trace!(
            "submit_request >> Submitting ledger request, cache_id: {cache_id:?}, request: \
             {request:?}"
        );
        let (response, is_from_cache) = match cache_id {
            Some(cache_id) => match self.response_cacher.get(cache_id, None).await? {
                Some(response) => {
                    trace!("submit_request << Returning cached response");
                    (response, true)
                }
                None => {
                    trace!(
                        "submit_request << cache miss, will make ledger request. Response will be \
                         cached."
                    );
                    let response = self.request_submitter.submit(request).await?;
                    self.response_cacher.put(cache_id, response.clone()).await?;
                    (response, false)
                }
            },
            None => {
                trace!("submit_request << caching is disabled for this request");
                let response = self.request_submitter.submit(request).await?;
                (response, false)
            }
        };
        trace!("submit_request << ledger response (is from cache: {is_from_cache}): {response}");
        Ok(response)
    }

    async fn resolve_schema_id_from_seq_no(
        &self,
        seq_no: i32,
        submitter_did: Option<&Did>,
    ) -> VcxLedgerResult<IndyVdrSchemaId> {
        let response = self.get_ledger_txn(seq_no, submitter_did).await?;
        let mut txn_response = self.response_parser.parse_get_txn_response(&response)?;
        let txn = txn_response["txn"].take();

        // mimic acapy & credo-ts behaviour - assumes node protocol >= 1.4

        // check correct tx
        let txn_type = &txn["type"];
        if txn_type != &json!("101") {
            return Err(VcxLedgerError::InvalidLedgerResponse(format!(
                "Expected indy schema transaction type (101), found: {txn_type}"
            )));
        }

        // pull schema identifier parts
        let schema_did = &txn["metadata"]["from"];
        let schema_name = &txn["data"]["data"]["name"];
        let schema_version = &txn["data"]["data"]["version"];
        let (Value::String(did), Value::String(name), Value::String(ver)) =
            (schema_did, schema_name, schema_version)
        else {
            return Err(VcxLedgerError::InvalidLedgerResponse(
                "Could not resolve schema DID, name & version from txn".into(),
            ));
        };

        // construct indy schema ID from parts
        let did = DidValue::new(did, None);
        did.validate()?;
        let schema_id = IndyVdrSchemaId::new(&did, name, ver);
        schema_id.validate()?;
        Ok(schema_id)
    }
}

impl<T> IndyVdrLedgerWrite<T>
where
    T: RequestSubmitter + Send + Sync,
{
    pub fn new(config: IndyVdrLedgerWriteConfig<T>) -> Self {
        Self {
            request_submitter: config.request_submitter,
            taa_options: RwLock::new(None),
            protocol_version: config.protocol_version,
        }
    }

    pub fn request_builder(&self) -> VcxLedgerResult<RequestBuilder> {
        Ok(RequestBuilder::new(self.protocol_version))
    }

    async fn sign_request(
        wallet: &impl BaseWallet,
        did: &Did,
        request: &PreparedRequest,
    ) -> VcxLedgerResult<Vec<u8>> {
        let to_sign = request.get_signature_input()?;
        let signer_verkey = wallet.key_for_did(&did.to_string()).await?;
        let signature = wallet.sign(&signer_verkey, to_sign.as_bytes()).await?;
        Ok(signature)
    }

    async fn sign_and_submit_request(
        &self,
        wallet: &impl BaseWallet,
        submitter_did: &Did,
        request: PreparedRequest,
    ) -> VcxLedgerResult<String> {
        let mut request = request;
        let signature = Self::sign_request(wallet, submitter_did, &request).await?;
        request.set_signature(&signature)?;
        self.request_submitter.submit(request).await
    }
}

impl<T> TaaConfigurator for IndyVdrLedgerWrite<T>
where
    T: RequestSubmitter + Send + Sync,
{
    fn set_txn_author_agreement_options(
        &self,
        taa_options: TxnAuthrAgrmtOptions,
    ) -> VcxLedgerResult<()> {
        let mut m = self.taa_options.write()?;
        *m = Some(taa_options);
        Ok(())
    }

    fn get_txn_author_agreement_options(&self) -> VcxLedgerResult<Option<TxnAuthrAgrmtOptions>> {
        Ok(self.taa_options.read()?.clone())
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

impl<T> Debug for IndyVdrLedgerWrite<T>
where
    T: RequestSubmitter + Send + Sync,
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
    async fn get_attr(&self, target_did: &Did, attr_name: &str) -> VcxLedgerResult<String> {
        debug!("get_attr >> target_did: {target_did}, attr_name: {attr_name}");
        let request = self.request_builder()?.build_get_attrib_request(
            None,
            &target_did.convert(())?,
            Some(attr_name.to_string()),
            None,
            None,
            None,
            None,
        )?;
        let response = self.submit_request(None, request).await?;
        debug!("get_attr << response: {response}");
        Ok(response)
    }

    async fn get_nym(&self, did: &Did) -> VcxLedgerResult<String> {
        debug!("get_nym >> did: {did}");
        let request =
            self.request_builder()?
                .build_get_nym_request(None, &did.convert(())?, None, None)?;
        let response = self.submit_request(None, request).await?;
        debug!("get_nym << response: {response}");
        Ok(response)
    }

    async fn get_txn_author_agreement(&self) -> VcxLedgerResult<Option<String>> {
        debug!("get_txn_author_agreement >>");
        let request = self
            .request_builder()?
            .build_get_txn_author_agreement_request(None, None)?;
        let response = self.submit_request(None, request).await?;
        debug!("get_txn_author_agreement << response: {response}");
        map_error_not_found_to_none(
            self.response_parser
                .parse_get_txn_author_agreement_response(&response),
        )?
        .map(|taa| serde_json::to_string(&taa).map_err(Into::into))
        .transpose()
    }

    async fn get_ledger_txn(
        &self,
        seq_no: i32,
        submitter_did: Option<&Did>,
    ) -> VcxLedgerResult<String> {
        debug!("get_ledger_txn >> seq_no: {seq_no}");
        let identifier = submitter_did.map(|did| did.convert(())).transpose()?;
        let request = self.request_builder()?.build_get_txn_request(
            identifier.as_ref(),
            LedgerType::DOMAIN.to_id(),
            seq_no,
        )?;
        let response = self.submit_request(None, request).await?;
        debug!("get_ledger_txn << response: {response}");
        Ok(response)
    }
}

impl<T> IndyVdrLedgerWrite<T>
where
    T: RequestSubmitter + Send + Sync,
{
    async fn append_txn_author_agreement_to_request(
        &self,
        request: PreparedRequest,
    ) -> VcxLedgerResult<PreparedRequest> {
        let taa_options = (*self.taa_options.read()?).clone();
        if let Some(taa_options) = taa_options {
            let mut request = request;
            let taa_data = self
                .request_builder()?
                .prepare_txn_author_agreement_acceptance_data(
                    Some(&taa_options.text),
                    Some(&taa_options.version),
                    None,
                    &taa_options.mechanism,
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
impl<T> IndyLedgerWrite for IndyVdrLedgerWrite<T>
where
    T: RequestSubmitter + Send + Sync,
{
    async fn publish_nym(
        &self,
        wallet: &impl BaseWallet,
        submitter_did: &Did,
        target_did: &Did,
        verkey: Option<&Key>,
        data: Option<&str>,
        role: Option<&str>,
    ) -> VcxLedgerResult<String> {
        let identifier = submitter_did.convert(())?;
        let dest = target_did.convert(())?;
        let request = self.request_builder()?.build_nym_request(
            &identifier,
            &dest,
            verkey.map(Key::base58),
            data.map(String::from),
            role.map(UpdateRole::from_str).transpose()?,
            None,
            None,
        )?;
        let request = self.append_txn_author_agreement_to_request(request).await?;
        self.sign_and_submit_request(wallet, submitter_did, request)
            .await
    }

    async fn set_endorser(
        &self,
        wallet: &impl BaseWallet,
        submitter_did: &Did,
        request_json: &str,
        endorser: &Did,
    ) -> VcxLedgerResult<String> {
        let mut request = PreparedRequest::from_request_json(request_json)?;
        request.set_endorser(&endorser.convert(())?)?;
        let signature_submitter = Self::sign_request(wallet, submitter_did, &request).await?;
        request.set_multi_signature(&submitter_did.convert(())?, &signature_submitter)?;
        Ok(request.req_json.to_string())
    }

    async fn endorse_transaction(
        &self,
        wallet: &impl BaseWallet,
        endorser_did: &Did,
        request_json: &str,
    ) -> VcxLedgerResult<()> {
        let mut request = PreparedRequest::from_request_json(request_json)?;
        verify_transaction_can_be_endorsed(request_json, endorser_did)?;
        let signature_endorser = Self::sign_request(wallet, endorser_did, &request).await?;
        request.set_multi_signature(&endorser_did.convert(())?, &signature_endorser)?;
        self.request_submitter.submit(request).await.map(|_| ())
    }

    async fn add_attr(
        &self,
        wallet: &impl BaseWallet,
        target_did: &Did,
        attrib_json: &str,
    ) -> VcxLedgerResult<String> {
        let identifier = target_did.convert(())?;
        let dest = target_did.convert(())?;
        let request = self.request_builder()?.build_attrib_request(
            &identifier,
            &dest,
            None,
            Some(&serde_json::from_str::<Value>(attrib_json)?),
            None,
        )?;
        let request = self.append_txn_author_agreement_to_request(request).await?;
        self.sign_and_submit_request(wallet, target_did, request)
            .await
    }

    async fn write_did(
        &self,
        wallet: &impl BaseWallet,
        submitter_did: &Did,
        target_did: &Did,
        target_vk: &Key,
        role: Option<UpdateRole>,
        alias: Option<String>,
    ) -> VcxLedgerResult<String> {
        debug!(
            "write_did >> submitter_did: {submitter_did}, target_did: {target_did}, target_vk: \
             {target_vk:?}, role: {role:?}, alias: {alias:?}"
        );
        let request = self.request_builder()?.build_nym_request(
            &submitter_did.convert(())?,
            &target_did.convert(())?,
            Some(target_vk.base58()),
            alias,
            role,
            None,
            None,
        )?;
        let request = self.append_txn_author_agreement_to_request(request).await?;
        let response = self
            .sign_and_submit_request(wallet, submitter_did, request)
            .await?;
        debug!("write_did << response: {response}");
        return Ok(response);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RevocationRegistryDefinitionAdditionalMetadata {
    pub max_cred_num: usize,
    pub issuer_id: IssuerId,
}

#[async_trait]
impl<T, V> AnoncredsLedgerRead for IndyVdrLedgerRead<T, V>
where
    T: RequestSubmitter + Send + Sync,
    V: ResponseCacher + Send + Sync,
{
    type RevocationRegistryDefinitionAdditionalMetadata =
        RevocationRegistryDefinitionAdditionalMetadata;

    async fn get_schema(
        &self,
        schema_id: &SchemaId,
        _submitter_did: Option<&Did>,
    ) -> VcxLedgerResult<Schema> {
        debug!("get_schema >> schema_id: {schema_id}");
        let request = self
            .request_builder()?
            .build_get_schema_request(None, &schema_id.convert(())?)?;
        let response = self.submit_request(None, request).await?;
        debug!("get_schema << response: {response}");
        let schema = self
            .response_parser
            .parse_get_schema_response(&response, None)?;
        Ok(schema.convert(())?)
    }

    async fn get_cred_def(
        &self,
        cred_def_id: &CredentialDefinitionId,
        submitter_did: Option<&Did>,
    ) -> VcxLedgerResult<CredentialDefinition> {
        debug!("get_cred_def >> cred_def_id: {cred_def_id}");
        let identifier = submitter_did.map(|did| did.convert(())).transpose()?;
        let id = IndyVdrCredentialDefinitionId::from_str(&cred_def_id.to_string())?;
        let request = self
            .request_builder()?
            .build_get_cred_def_request(identifier.as_ref(), &id)?;
        // note: Before we try to create credential definition, we are checking if it already
        //       doesn't exist on the ledger to prevent invalidating the old one.
        //       When we make the first request, it typically doesn't exist, but we don't want to
        //       cache such as result. So caching strategy should perhaps only store data in cache
        //       if ledger response was found / the response is success.
        //       Therefore parsing should happen prior to caching.
        let response = self.submit_request(None, request).await?;
        debug!("get_cred_def << response: {response}");
        let cred_def = self
            .response_parser
            .parse_get_cred_def_response(&response, None)?;

        // extract and map seqNo -> schemaId if required
        let IndyVdrCredentialDefinition::CredentialDefinitionV1(mut cred_def) = cred_def;
        if let Ok(seq_no) = cred_def.schema_id.0.parse::<i32>() {
            cred_def.schema_id = self
                .resolve_schema_id_from_seq_no(seq_no, submitter_did)
                .await?;
        }

        let cred_def = IndyVdrCredentialDefinition::CredentialDefinitionV1(cred_def).convert(())?;

        Ok(cred_def)
    }

    async fn get_rev_reg_def_json(
        &self,
        rev_reg_id: &RevocationRegistryDefinitionId,
    ) -> VcxLedgerResult<(
        RevocationRegistryDefinition,
        RevocationRegistryDefinitionAdditionalMetadata,
    )> {
        debug!("get_rev_reg_def_json >> rev_reg_id: {rev_reg_id}");
        let id = RevocationRegistryId::from_str(&rev_reg_id.to_string())?;
        let request = self
            .request_builder()?
            .build_get_revoc_reg_def_request(None, &id)?;
        let response = self
            .submit_request(Some(&rev_reg_id.to_string()), request)
            .await?;
        debug!("get_rev_reg_def_json << response: {response}");
        let rev_reg_def = self
            .response_parser
            .parse_get_revoc_reg_def_response(&response)?;
        let def = rev_reg_def.convert(())?;

        let meta = RevocationRegistryDefinitionAdditionalMetadata {
            max_cred_num: def.value.max_cred_num as usize,
            issuer_id: def.issuer_id.clone(),
        };

        Ok((def, meta))
    }

    async fn get_rev_reg_delta_json(
        &self,
        rev_reg_id: &RevocationRegistryDefinitionId,
        from: Option<u64>,
        to: Option<u64>,
    ) -> VcxLedgerResult<(RevocationRegistryDelta, u64)> {
        debug!("get_rev_reg_delta_json >> rev_reg_id: {rev_reg_id}, from: {from:?}, to: {to:?}");
        let revoc_reg_def_id = RevocationRegistryId::from_str(&rev_reg_id.to_string())?;

        let from = from.map(|x| x as i64);
        let current_time = OffsetDateTime::now_utc().unix_timestamp();
        let to = to.map_or(current_time, |x| x as i64);

        let request = self.request_builder()?.build_get_revoc_reg_delta_request(
            None,
            &revoc_reg_def_id,
            from,
            to,
        )?;
        let response = self.submit_request(None, request).await?;
        debug!("get_rev_reg_delta_json << response: {response}");

        let RevocationRegistryDeltaInfo {
            revoc_reg_delta,
            timestamp,
            ..
        } = self
            .response_parser
            .parse_get_revoc_reg_delta_response(&response)?;
        Ok((revoc_reg_delta.convert(())?, timestamp))
    }

    async fn get_rev_status_list(
        &self,
        rev_reg_id: &RevocationRegistryDefinitionId,
        timestamp: u64,
        rev_reg_def_meta: Option<&RevocationRegistryDefinitionAdditionalMetadata>,
    ) -> VcxLedgerResult<(RevocationStatusList, u64)> {
        #[allow(deprecated)] // TODO - https://github.com/hyperledger/aries-vcx/issues/1309
        let (delta, entry_time) = self
            .get_rev_reg_delta_json(rev_reg_id, Some(0), Some(timestamp))
            .await?;

        let rev_reg_def_meta = match rev_reg_def_meta {
            Some(x) => x,
            None => &self.get_rev_reg_def_json(rev_reg_id).await?.1,
        };

        let status_list = from_revocation_registry_delta_to_revocation_status_list(
            &delta.value,
            entry_time,
            rev_reg_id,
            rev_reg_def_meta.max_cred_num,
            rev_reg_def_meta.issuer_id.clone(),
        )
        .map_err(|e| {
            VcxLedgerError::InvalidLedgerResponse(format!(
                "received rev status delta could not be translated to status list: {e} {delta:?}"
            ))
        })?;

        Ok((status_list, entry_time))
    }

    async fn get_rev_reg(
        &self,
        rev_reg_id: &RevocationRegistryDefinitionId,
        timestamp: u64,
    ) -> VcxLedgerResult<(RevocationRegistry, u64)> {
        debug!("get_rev_reg >> rev_reg_id: {rev_reg_id}, timestamp: {timestamp}");
        let revoc_reg_def_id = RevocationRegistryId::from_str(&rev_reg_id.to_string())?;

        let request = self.request_builder()?.build_get_revoc_reg_request(
            None,
            &revoc_reg_def_id,
            timestamp.try_into().unwrap(),
        )?;
        let response = self.submit_request(None, request).await?;
        debug!("get_rev_reg << response: {response}");

        let RevocationRegistryInfo {
            revoc_reg,
            timestamp,
            ..
        } = self
            .response_parser
            .parse_get_revoc_reg_response(&response)?;

        Ok((revoc_reg.convert(())?, timestamp))
    }
}

impl<T: RequestSubmitter, V: ResponseCacher> AnoncredsLedgerSupport for IndyVdrLedgerRead<T, V> {
    fn supports_schema(&self, id: &SchemaId) -> bool {
        if id.is_legacy() {
            // unqualified
            return true;
        }
        did_method_is_supported(&id.0)
    }

    fn supports_credential_definition(&self, id: &CredentialDefinitionId) -> bool {
        if id.is_legacy_cred_def_identifier() {
            // unqualified
            return true;
        }
        did_method_is_supported(&id.0)
    }

    fn supports_revocation_registry(&self, id: &RevocationRegistryDefinitionId) -> bool {
        if id.is_legacy() {
            // unqualified
            return true;
        }
        did_method_is_supported(&id.0)
    }
}

fn did_method_is_supported(id: &str) -> bool {
    let is_sov = id.starts_with("did:sov:");
    let is_unqualified = !id.starts_with("did");

    // FUTURE - indy & namespace

    is_sov || is_unqualified
}

#[async_trait]
impl<T> AnoncredsLedgerWrite for IndyVdrLedgerWrite<T>
where
    T: RequestSubmitter + Send + Sync,
{
    async fn publish_schema(
        &self,
        wallet: &impl BaseWallet,
        schema_json: Schema,
        submitter_did: &Did,
        _endorser_did: Option<&Did>,
    ) -> VcxLedgerResult<()> {
        let identifier = submitter_did.convert(())?;
        let mut request = self
            .request_builder()?
            .build_schema_request(&identifier, schema_json.convert(())?)?;
        request = self.append_txn_author_agreement_to_request(request).await?;
        // if let Some(endorser_did) = endorser_did {
        //     request = PreparedRequest::from_request_json(
        //         self.set_endorser(submitter_did, &request.req_json.to_string(), &endorser_did)
        //             .await?,
        //     )?
        // }
        let sign_result = self
            .sign_and_submit_request(wallet, submitter_did, request)
            .await;

        if matches!(sign_result, Err(VcxLedgerError::InvalidLedgerResponse(_))) {
            return Err(VcxLedgerError::DuplicationSchema);
        }
        sign_result.map(|_| ())
    }

    async fn publish_cred_def(
        &self,
        wallet: &impl BaseWallet,
        cred_def_json: CredentialDefinition,
        submitter_did: &Did,
    ) -> VcxLedgerResult<()> {
        let identifier = submitter_did.convert(())?;
        let request = self
            .request_builder()?
            .build_cred_def_request(&identifier, cred_def_json.convert(())?)?;
        let request = self.append_txn_author_agreement_to_request(request).await?;
        self.sign_and_submit_request(wallet, submitter_did, request)
            .await
            .map(|_| ())
    }

    async fn publish_rev_reg_def(
        &self,
        wallet: &impl BaseWallet,
        rev_reg_def: RevocationRegistryDefinition,
        submitter_did: &Did,
    ) -> VcxLedgerResult<()> {
        let identifier = submitter_did.convert(())?;
        let request = self
            .request_builder()?
            .build_revoc_reg_def_request(&identifier, rev_reg_def.convert(())?)?;
        let request = self.append_txn_author_agreement_to_request(request).await?;
        self.sign_and_submit_request(wallet, submitter_did, request)
            .await
            .map(|_| ())
    }

    async fn publish_rev_reg_delta(
        &self,
        wallet: &impl BaseWallet,
        rev_reg_id: &RevocationRegistryDefinitionId,
        rev_reg_entry_json: RevocationRegistryDelta,
        submitter_did: &Did,
    ) -> VcxLedgerResult<()> {
        let identifier = submitter_did.convert(())?;
        let request = self.request_builder()?.build_revoc_reg_entry_request(
            &identifier,
            &RevocationRegistryId::from_str(&rev_reg_id.to_string())?,
            &RegistryType::CL_ACCUM,
            rev_reg_entry_json.convert(())?,
        )?;
        let request = self.append_txn_author_agreement_to_request(request).await?;
        self.sign_and_submit_request(wallet, submitter_did, request)
            .await
            .map(|_| ())
    }
}

pub fn indyvdr_build_ledger_read(
    request_submitter: IndyVdrSubmitter,
    cache_config: InMemoryResponseCacherConfig,
) -> VcxLedgerResult<IndyVdrLedgerRead<IndyVdrSubmitter, InMemoryResponseCacher>> {
    let response_parser = ResponseParser;
    let response_cacher = InMemoryResponseCacher::new(cache_config);

    let config_read = IndyVdrLedgerReadConfig {
        request_submitter,
        response_parser,
        response_cacher,
        protocol_version: ProtocolVersion::Node1_4,
    };
    Ok(IndyVdrLedgerRead::new(config_read))
}

pub fn indyvdr_build_ledger_write(
    request_submitter: IndyVdrSubmitter,
    taa_options: Option<TxnAuthrAgrmtOptions>,
) -> IndyVdrLedgerWrite<IndyVdrSubmitter> {
    let config_write = IndyVdrLedgerWriteConfig {
        request_submitter,
        taa_options,
        protocol_version: ProtocolVersion::Node1_4,
    };
    IndyVdrLedgerWrite::new(config_write)
}

#[derive(Clone)]
pub struct VcxPoolConfig {
    pub genesis_file_path: String,
    pub indy_vdr_config: Option<PoolConfig>,
    pub response_cache_config: Option<InMemoryResponseCacherConfig>,
}

pub fn build_ledger_components(
    pool_config: VcxPoolConfig,
) -> VcxLedgerResult<(DefaultIndyLedgerRead, DefaultIndyLedgerWrite)> {
    let indy_vdr_config = pool_config.indy_vdr_config.unwrap_or_default();
    let cache_config = match pool_config.response_cache_config {
        None => InMemoryResponseCacherConfig::builder()
            .ttl(std::time::Duration::from_secs(60))
            .capacity(1000)?
            .build(),
        Some(cfg) => cfg,
    };

    let ledger_pool =
        IndyVdrLedgerPool::new(pool_config.genesis_file_path, indy_vdr_config, vec![])?;

    let request_submitter = IndyVdrSubmitter::new(ledger_pool);

    let ledger_read = indyvdr_build_ledger_read(request_submitter.clone(), cache_config)?;
    let ledger_write = indyvdr_build_ledger_write(request_submitter, None);

    Ok((ledger_read, ledger_write))
}

#[cfg(test)]
mod unit_tests {
    use mockall::mock;

    use super::*;
    use crate::ledger::response_cacher::noop::NoopResponseCacher;

    mock! {
        pub RequestSubmitter {}
        #[async_trait]
        impl RequestSubmitter for RequestSubmitter {
            async fn submit(&self, request: indy_vdr::pool::PreparedRequest) -> VcxLedgerResult<String>;
        }
    }

    fn dummy_indy_vdr_reader() -> IndyVdrLedgerRead<MockRequestSubmitter, NoopResponseCacher> {
        IndyVdrLedgerRead::new(IndyVdrLedgerReadConfig {
            request_submitter: MockRequestSubmitter::new(),
            response_parser: indy_ledger_response_parser::ResponseParser,
            response_cacher: NoopResponseCacher,
            protocol_version: ProtocolVersion::Node1_4,
        })
    }

    #[test]
    fn test_anoncreds_schema_support() {
        let reader = dummy_indy_vdr_reader();

        // legacy
        assert!(reader.supports_schema(
            &SchemaId::new("7BPMqYgYLQni258J8JPS8K:2:degree schema:46.58.87").unwrap()
        ));
        // qualified sov
        assert!(reader.supports_schema(
            &SchemaId::new("did:sov:7BPMqYgYLQni258J8JPS8K:2:degree schema:46.58.87").unwrap()
        ));
        // qualified cheqd
        assert!(!reader.supports_schema(
            &SchemaId::new(
                "did:cheqd:mainnet:7BPMqYgYLQni258J8JPS8K/resources/\
                 6259d357-eeb1-4b98-8bee-12a8390d3497"
            )
            .unwrap()
        ));
    }

    #[test]
    fn test_anoncreds_cred_def_support() {
        let reader = dummy_indy_vdr_reader();

        // legacy
        assert!(reader.supports_credential_definition(
            &CredentialDefinitionId::new(
                "7BPMqYgYLQni258J8JPS8K:3:CL:70:faber.agent.degree_schema"
            )
            .unwrap()
        ));
        // qualified sov
        assert!(reader.supports_credential_definition(
            &CredentialDefinitionId::new(
                "did:sov:7BPMqYgYLQni258J8JPS8K:3:CL:70:faber.agent.degree_schema"
            )
            .unwrap()
        ));
        // qualified cheqd
        assert!(!reader.supports_credential_definition(
            &CredentialDefinitionId::new(
                "did:cheqd:mainnet:7BPMqYgYLQni258J8JPS8K/resources/\
                 6259d357-eeb1-4b98-8bee-12a8390d3497"
            )
            .unwrap()
        ));
    }

    #[test]
    fn test_anoncreds_rev_reg_support() {
        let reader = dummy_indy_vdr_reader();

        // legacy
        assert!(reader.supports_revocation_registry(
            &RevocationRegistryDefinitionId::new(
                "7BPMqYgYLQni258J8JPS8K:4:7BPMqYgYLQni258J8JPS8K:3:CL:70:faber.agent.\
                 degree_schema:CL_ACCUM:61d5a381-30be-4120-9307-b150b49c203c"
            )
            .unwrap()
        ));
        // qualified sov
        assert!(reader.supports_revocation_registry(
            &RevocationRegistryDefinitionId::new(
                "did:sov:7BPMqYgYLQni258J8JPS8K:4:7BPMqYgYLQni258J8JPS8K:3:CL:70:faber.agent.\
                 degree_schema:CL_ACCUM:61d5a381-30be-4120-9307-b150b49c203c"
            )
            .unwrap()
        ));
        // qualified cheqd
        assert!(!reader.supports_revocation_registry(
            &RevocationRegistryDefinitionId::new(
                "did:cheqd:mainnet:7BPMqYgYLQni258J8JPS8K/resources/\
                 6259d357-eeb1-4b98-8bee-12a8390d3497"
            )
            .unwrap()
        ));
    }
}
