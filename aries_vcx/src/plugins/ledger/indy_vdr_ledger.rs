use std::{
    collections::{hash_map::RandomState, HashMap},
    sync::Arc,
};

use async_trait::async_trait;
use indy_vdr as vdr;
use serde_json::Value;
use tokio::sync::oneshot;
use vdr::{
    common::error::VdrError,
    config::PoolConfig as IndyVdrPoolConfig,
    ledger::{
        identifiers::{CredentialDefinitionId, RevocationRegistryId, SchemaId},
        requests::{
            author_agreement::TxnAuthrAgrmtAcceptanceData,
            schema::{AttributeNames, Schema, SchemaV1},
        },
        RequestBuilder,
    },
    pool::{PoolBuilder, PoolRunner, PoolTransactions, PreparedRequest, ProtocolVersion, RequestResult},
    utils::{did::DidValue, Qualifiable},
};

use super::base_ledger::BaseLedger;
use crate::{
    common::primitives::revocation_registry::RevocationRegistryDefinition,
    core::profile::{modular_wallet_profile::LedgerPoolConfig, profile::Profile},
    errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult},
    global::settings,
    utils::{
        author_agreement::get_txn_author_agreement,
        json::{AsTypeOrDeserializationError, TryGetIndex},
    },
};

pub struct IndyVdrLedgerPool {
    // visibility strictly for internal unit testing
    pub(self) runner: Option<PoolRunner>,
}

impl IndyVdrLedgerPool {
    pub fn new_from_runner(runner: PoolRunner) -> Self {
        IndyVdrLedgerPool { runner: Some(runner) }
    }

    pub fn new(config: LedgerPoolConfig) -> VcxResult<Self> {
        let vdr_config = IndyVdrPoolConfig::default();
        let txns = PoolTransactions::from_json_file(config.genesis_file_path)?;

        let runner = PoolBuilder::from(vdr_config).transactions(txns)?.into_runner()?;

        Ok(IndyVdrLedgerPool { runner: Some(runner) })
    }
}

impl std::fmt::Debug for IndyVdrLedgerPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IndyVdrLedgerPool")
            .field("runner", &"PoolRunner")
            .finish()
    }
}

pub struct IndyVdrLedger {
    profile: Arc<dyn Profile>,
    pool: Arc<IndyVdrLedgerPool>,
}

impl IndyVdrLedger {
    pub fn new(profile: Arc<dyn Profile>, pool: Arc<IndyVdrLedgerPool>) -> Self {
        IndyVdrLedger { profile, pool }
    }

    pub fn request_builder(&self) -> VcxResult<RequestBuilder> {
        // TODO - confirm correct protocol version?
        let v = settings::get_protocol_version();
        let version = ProtocolVersion::from_id(v as u64)?;
        Ok(RequestBuilder::new(version))
    }

    async fn _submit_request(&self, request: PreparedRequest) -> VcxResult<String> {
        // indyvdr send_request is Async via a callback.
        // Use oneshot channel to send result from callback, converting the fn to future.
        type VdrSendRequestResult =
            Result<(RequestResult<String>, Option<HashMap<String, f32, RandomState>>), VdrError>;
        let (sender, recv) = oneshot::channel::<VdrSendRequestResult>();
        self.pool
            .runner
            .as_ref()
            .ok_or(
                // should not happen - strictly for unit testing
                AriesVcxError::from_msg(
                    AriesVcxErrorKind::NoPoolOpen,
                    "IndyVdrLedgerPool runner was not provided",
                ),
            )?
            .send_request(
                request,
                Box::new(move |result| {
                    // unable to handle a failure from `send` here
                    sender.send(result).ok();
                }),
            )?;

        let send_req_result: VdrSendRequestResult = recv
            .await
            .map_err(|e| AriesVcxError::from_msg(AriesVcxErrorKind::InvalidState, e))?;
        let (result, _) = send_req_result?;

        let reply = match result {
            RequestResult::Reply(reply) => Ok(reply),
            RequestResult::Failed(failed) => Err(failed),
        };

        Ok(reply?)
    }

    async fn _sign_and_submit_request(&self, submitter_did: &str, request: PreparedRequest) -> VcxResult<String> {
        let mut request = request;
        let to_sign = request.get_signature_input()?;

        let wallet = self.profile.inject_wallet();

        let signer_verkey = wallet.key_for_local_did(submitter_did).await?;

        let signature = self
            .profile
            .inject_wallet()
            .sign(&signer_verkey, to_sign.as_bytes())
            .await?;

        request.set_signature(&signature)?;

        self._submit_request(request).await
    }

    async fn _build_get_cred_def_request(
        &self,
        submitter_did: Option<&str>,
        cred_def_id: &str,
    ) -> VcxResult<PreparedRequest> {
        let identifier = if let Some(did) = submitter_did {
            Some(DidValue::from_str(did)?)
        } else {
            None
        };
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
    ) -> VcxResult<PreparedRequest> {
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
    ) -> VcxResult<PreparedRequest> {
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
}

#[async_trait]
impl BaseLedger for IndyVdrLedger {
    async fn sign_and_submit_request(&self, submitter_did: &str, request_json: &str) -> VcxResult<String> {
        let request = PreparedRequest::from_request_json(request_json)?;

        self._sign_and_submit_request(submitter_did, request).await
    }

    async fn submit_request(&self, request_json: &str) -> VcxResult<String> {
        let request = PreparedRequest::from_request_json(request_json)?;
        self._submit_request(request).await
    }

    async fn endorse_transaction(&self, endorser_did: &str, request_json: &str) -> VcxResult<()> {
        let _ = (endorser_did, request_json);
        Err(unimplemented_method_err("indy_vdr endorse_transaction"))
    }

    async fn set_endorser(&self, submitter_did: &str, request_json: &str, endorser: &str) -> VcxResult<String> {
        let _ = (submitter_did, request_json, endorser);
        Err(unimplemented_method_err("indy_vdr set_endorser"))
    }

    async fn get_txn_author_agreement(&self) -> VcxResult<String> {
        Err(unimplemented_method_err("indy_vdr get_txn_author_agreement"))
    }

    async fn get_nym(&self, did: &str) -> VcxResult<String> {
        let dest = DidValue::from_str(did)?;
        let request = self.request_builder()?.build_get_nym_request(None, &dest)?;

        self._submit_request(request).await
    }

    async fn publish_nym(
        &self,
        submitter_did: &str,
        target_did: &str,
        verkey: Option<&str>,
        data: Option<&str>,
        role: Option<&str>,
    ) -> VcxResult<String> {
        // TODO - FUTURE: convert data into "alias" for indy vdr. for now throw unimplemented
        if data.is_some() {
            return Err(unimplemented_method_err("indy_vdr publish_nym with data"));
        }
        let alias = None;

        let identifier = DidValue::from_str(submitter_did)?;
        let dest = DidValue::from_str(target_did)?;
        let request = self.request_builder()?.build_nym_request(
            &identifier,
            &dest,
            verkey.map(String::from),
            alias,
            role.map(String::from),
        )?;

        self._sign_and_submit_request(submitter_did, request).await
    }

    async fn get_schema(&self, schema_id: &str, submitter_did: Option<&str>) -> VcxResult<String> {
        let _ = submitter_did;
        // TODO - future - try from cache first
        // TODO - future - do we need to handle someone submitting a schema request by seq number?

        let id = SchemaId::from_str(schema_id)?;

        let request = self.request_builder()?.build_get_schema_request(None, &id)?;

        let response = self._submit_request(request).await?;

        // process the response
        let response_json: Value = serde_json::from_str(&response)?;
        let result_json = (&response_json).try_get("result")?;
        let data_json = result_json.try_get("data")?;

        let seq_no = result_json.get("seqNo").and_then(|x| x.as_u64().map(|x| x as u32));

        let name = data_json.try_get("name")?;
        let name = name.try_as_str()?;
        let version = data_json.try_get("version")?;
        let version = version.try_as_str()?;
        let dest = result_json.try_get("dest")?;
        let dest = dest.try_as_str()?;
        let schema_id = SchemaId::new(&DidValue::from_str(dest)?, name, version);

        let attr_names = data_json.try_get("attr_names")?;
        let attr_names: AttributeNames = serde_json::from_value(attr_names.to_owned())?;

        let schema = SchemaV1 {
            id: schema_id,
            name: name.to_string(),
            version: version.to_string(),
            attr_names,
            seq_no,
        };

        // TODO - future - store in cache if submitter_did provided

        Ok(serde_json::to_string(&Schema::SchemaV1(schema))?)
    }

    async fn get_cred_def(&self, cred_def_id: &str, submitter_did: Option<&str>) -> VcxResult<String> {
        // todo - try from cache if submitter_did provided

        let request = self._build_get_cred_def_request(submitter_did, cred_def_id).await?;

        let response = self._submit_request(request).await?;

        // process the response

        let response_json: Value = serde_json::from_str(&response)?;
        let result_json = (&response_json).try_get("result")?;

        let schema_id = result_json.try_get("ref")?;
        let signature_type = result_json.try_get("signature_type")?;
        let tag = result_json.get("tag").map_or(json!("default"), |x| x.to_owned());
        let origin_did = result_json.try_get("origin")?;
        // (from ACApy) FIXME: issuer has a method to create a cred def ID
        // may need to qualify the DID
        let cred_def_id = format!(
            "{}:3:{}:{}:{}",
            origin_did.try_as_str()?,
            signature_type.try_as_str()?,
            schema_id,
            (&tag).try_as_str()?
        );
        let data = _get_response_json_data_field(&response)?;

        let cred_def_value = json!({
            "ver": "1.0",
            "id": cred_def_id,
            "schemaId": schema_id.to_string(), // expected as json string, not as json int
            "type": signature_type,
            "tag": tag,
            "value": data
        });

        let cred_def_json = serde_json::to_string(&cred_def_value)?;

        // todo - store in cache if submitter_did provided

        Ok(cred_def_json)
    }

    async fn get_attr(&self, target_did: &str, attr_name: &str) -> VcxResult<String> {
        let request = self._build_get_attr_request(None, target_did, attr_name).await?;

        self._submit_request(request).await
    }

    async fn add_attr(&self, target_did: &str, attrib_json: &str) -> VcxResult<String> {
        let request = self._build_attrib_request(target_did, target_did, Some(attrib_json))?;
        let request = _append_txn_author_agreement_to_request(request).await?;

        self._sign_and_submit_request(target_did, request).await
    }

    async fn get_rev_reg_def_json(&self, rev_reg_id: &str) -> VcxResult<String> {
        let id = RevocationRegistryId::from_str(rev_reg_id)?;
        let request = self.request_builder()?.build_get_revoc_reg_def_request(None, &id)?;
        let res = self._submit_request(request).await?;

        let mut data = _get_response_json_data_field(&res)?;

        data["ver"] = Value::String("1.0".to_string());

        Ok(serde_json::to_string(&data)?)
    }

    async fn get_rev_reg_delta_json(
        &self,
        rev_reg_id: &str,
        from: Option<u64>,
        to: Option<u64>,
    ) -> VcxResult<(String, String, u64)> {
        let revoc_reg_def_id = RevocationRegistryId::from_str(rev_reg_id)?;

        let from = from.map(|x| x as i64);
        let current_time = current_epoch_time();
        let to = to.map_or(current_time, |x| x as i64);

        let request = self
            .request_builder()?
            .build_get_revoc_reg_delta_request(None, &revoc_reg_def_id, from, to)?;
        let res = self._submit_request(request).await?;

        let res_data = _get_response_json_data_field(&res)?;
        let response_value = (&res_data).try_get("value")?;

        let empty_json_list = json!([]);

        let mut delta_value = json!({
            "accum": response_value.try_get("accum_to")?.try_get("value")?.try_get("accum")?,
            "issued": if let Some(v) = response_value.get("issued") { v } else { &empty_json_list },
            "revoked": if let Some(v) = response_value.get("revoked") { v } else { &empty_json_list }
        });

        if let Some(accum_from) = response_value
            .get("accum_from")
            .and_then(|val| (!val.is_null()).then_some(val))
        {
            let prev_accum = accum_from.try_get("value")?.try_get("accum")?;
            // to check - should this be 'prevAccum'?
            delta_value["prev_accum"] = prev_accum.to_owned();
        }

        let reg_delta = json!({"ver": "1.0", "value": delta_value});

        let delta_timestamp =
            response_value
                .try_get("accum_to")?
                .try_get("txnTime")?
                .as_u64()
                .ok_or(AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidJson,
                    "Error parsing accum_to.txnTime value as u64",
                ))?;

        let response_reg_def_id = (&res_data)
            .try_get("revocRegDefId")?
            .as_str()
            .ok_or(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidJson,
                "Erroring parsing revocRegDefId value as string",
            ))?;
        if response_reg_def_id != rev_reg_id {
            return Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidRevocationDetails,
                "ID of revocation registry response does not match requested ID",
            ));
        }

        Ok((
            rev_reg_id.to_string(),
            serde_json::to_string(&reg_delta)?,
            delta_timestamp,
        ))
    }

    async fn get_rev_reg(&self, rev_reg_id: &str, timestamp: u64) -> VcxResult<(String, String, u64)> {
        let _ = (rev_reg_id, timestamp);
        Err(unimplemented_method_err("indy_vdr get_rev_reg"))
    }

    async fn get_ledger_txn(&self, seq_no: i32, submitter_did: Option<&str>) -> VcxResult<String> {
        let _ = (seq_no, submitter_did);
        Err(unimplemented_method_err("indy_vdr get_ledger_txn"))
    }

    async fn build_schema_request(&self, submitter_did: &str, schema_json: &str) -> VcxResult<String> {
        let _ = (submitter_did, schema_json);
        Err(unimplemented_method_err("indy_vdr build_schema_request"))
    }

    async fn publish_schema(
        &self,
        schema_json: &str,
        submitter_did: &str,
        endorser_did: Option<String>,
    ) -> VcxResult<()> {
        let _ = (schema_json, submitter_did, endorser_did);
        Err(unimplemented_method_err("indy_vdr publish_schema"))
    }

    async fn publish_cred_def(&self, cred_def_json: &str, submitter_did: &str) -> VcxResult<()> {
        let _ = (cred_def_json, submitter_did);
        Err(unimplemented_method_err("indy_vdr publish_cred_def"))
    }

    async fn publish_rev_reg_def(
        &self,
        rev_reg_def: &RevocationRegistryDefinition,
        submitter_did: &str,
    ) -> VcxResult<()> {
        let _ = (rev_reg_def, submitter_did);
        Err(unimplemented_method_err("indy_vdr publish_rev_reg_def"))
    }

    async fn publish_rev_reg_delta(
        &self,
        rev_reg_id: &str,
        rev_reg_entry_json: &str,
        submitter_did: &str,
    ) -> VcxResult<()> {
        let _ = (rev_reg_entry_json, rev_reg_id, submitter_did);
        Err(unimplemented_method_err("indy_vdr publish_rev_reg_delta"))
    }
}

fn unimplemented_method_err(method_name: &str) -> AriesVcxError {
    AriesVcxError::from_msg(
        AriesVcxErrorKind::UnimplementedFeature,
        format!("method called '{}' is not yet implemented in AriesVCX", method_name),
    )
}

fn current_epoch_time() -> i64 {
    time::get_time().sec
}

async fn _append_txn_author_agreement_to_request(request: PreparedRequest) -> VcxResult<PreparedRequest> {
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

fn _get_response_json_data_field(response_json: &str) -> VcxResult<Value> {
    let res: Value = serde_json::from_str(response_json)?;
    let result = (&res).try_get("result")?;
    Ok(result.try_get("data")?.to_owned())
}

#[cfg(test)]
#[cfg(feature = "general_test")]
mod unit_tests {
    use std::sync::Arc;

    use super::IndyVdrLedger;
    use crate::{
        common::{primitives::revocation_registry::RevocationRegistryDefinition, test_utils::mock_profile},
        errors::error::{AriesVcxErrorKind, VcxResult},
        plugins::ledger::{base_ledger::BaseLedger, indy_vdr_ledger::IndyVdrLedgerPool},
    };

    #[tokio::test]
    async fn test_unimplemented_methods() {
        // test used to assert which methods are unimplemented currently, can be removed after all methods
        // implemented

        fn assert_unimplemented<T: std::fmt::Debug>(result: VcxResult<T>) {
            assert_eq!(result.unwrap_err().kind(), AriesVcxErrorKind::UnimplementedFeature)
        }

        let profile = mock_profile();
        let pool = Arc::new(IndyVdrLedgerPool { runner: None });
        let ledger: Box<dyn BaseLedger> = Box::new(IndyVdrLedger::new(profile, pool));

        assert_unimplemented(ledger.endorse_transaction("", "").await);
        assert_unimplemented(ledger.set_endorser("", "", "").await);
        assert_unimplemented(ledger.get_txn_author_agreement().await);
        assert_unimplemented(ledger.get_rev_reg("", 0).await);
        assert_unimplemented(ledger.get_ledger_txn(0, None).await);
        assert_unimplemented(ledger.build_schema_request("", "").await);
        assert_unimplemented(ledger.publish_schema("", "", None).await);
        assert_unimplemented(ledger.publish_cred_def("", "").await);
        assert_unimplemented(
            ledger
                .publish_rev_reg_def(&RevocationRegistryDefinition::default(), "")
                .await,
        );
    }
}
