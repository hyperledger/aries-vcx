use indy_vdr as vdr;
use indy_vdr_proxy_client::VdrProxyClient;
use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use time::OffsetDateTime;
use vdr::ledger::requests::schema::{AttributeNames, Schema, SchemaV1};

use async_trait::async_trait;
use serde_json::Value;
use tokio::sync::oneshot;
use vdr::common::error::VdrError;
use vdr::config::PoolConfig as IndyVdrPoolConfig;
use vdr::ledger::identifiers::{CredentialDefinitionId, RevocationRegistryId, SchemaId};
use vdr::ledger::requests::author_agreement::TxnAuthrAgrmtAcceptanceData;
use vdr::ledger::RequestBuilder;
use vdr::pool::{PoolBuilder, PoolTransactions};
use vdr::pool::{PoolRunner, PreparedRequest, ProtocolVersion, RequestResult};
use vdr::utils::did::DidValue;
use vdr::utils::Qualifiable;

use crate::errors::error::VcxCoreResult;
use crate::errors::error::{AriesVcxCoreError, AriesVcxCoreErrorKind};
use crate::global::author_agreement::get_txn_author_agreement;
use crate::global::settings;
use crate::utils::json::{AsTypeOrDeserializationError, TryGetIndex};
use crate::wallet::base_wallet::BaseWallet;

use super::base_ledger::BaseLedger;

pub struct VdrProxyLedger {
    wallet: Arc<dyn BaseWallet>,
    client: VdrProxyClient,
}

impl Debug for VdrProxyLedger {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("VdrProxyLedger")
    }
}

fn unimplemented_method_err(method_name: &str) -> AriesVcxCoreError {
    AriesVcxCoreError::from_msg(
        AriesVcxCoreErrorKind::UnimplementedFeature,
        format!("method called '{}' is not yet implemented in AriesVCX", method_name),
    )
}

fn _get_response_json_data_field(response_json: &str) -> VcxCoreResult<Value> {
    let res: Value = serde_json::from_str(response_json)?;
    let result = (&res).try_get("result")?;
    Ok(result.try_get("data")?.to_owned())
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

impl VdrProxyLedger {
    pub fn new(wallet: Arc<dyn BaseWallet>, client: VdrProxyClient) -> Self {
        Self { wallet, client }
    }

    pub fn request_builder(&self) -> VcxCoreResult<RequestBuilder> {
        let v = settings::get_protocol_version();
        let version = ProtocolVersion::from_id(v as u64)?;
        Ok(RequestBuilder::new(version))
    }

    async fn _sign_request(&self, submitter_did: &str, request: PreparedRequest) -> VcxCoreResult<PreparedRequest> {
        let to_sign = request.get_signature_input()?;
        let signer_verkey = self.wallet.key_for_local_did(submitter_did).await?;
        let signature = self.wallet.sign(&signer_verkey, to_sign.as_bytes()).await?;

        let request = {
            let mut request = request;
            request.set_signature(&signature)?;
            request
        };

        Ok(request)
    }

    async fn _build_get_cred_def_request(
        &self,
        submitter_did: Option<&str>,
        cred_def_id: &str,
    ) -> VcxCoreResult<PreparedRequest> {
        todo!()
    }

    async fn _build_get_attr_request(
        &self,
        submitter_did: Option<&str>,
        target_did: &str,
        attribute_name: &str,
    ) -> VcxCoreResult<PreparedRequest> {
        todo!()
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

    fn _process_schema_response(response: &str) -> VcxCoreResult<Schema> {
        let response_json: Value = serde_json::from_str(response)?;
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

        Ok(Schema::SchemaV1(schema))
    }

    fn _process_rev_reg_delta_response(response: &str, rev_reg_id: &str) -> VcxCoreResult<(String, String, u64)> {
        let res_data = _get_response_json_data_field(&response)?;
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
                .ok_or(AriesVcxCoreError::from_msg(
                    AriesVcxCoreErrorKind::InvalidJson,
                    "Error parsing accum_to.txnTime value as u64",
                ))?;

        let response_reg_def_id = (&res_data)
            .try_get("revocRegDefId")?
            .as_str()
            .ok_or(AriesVcxCoreError::from_msg(
                AriesVcxCoreErrorKind::InvalidJson,
                "Erroring parsing revocRegDefId value as string",
            ))?;
        if response_reg_def_id != rev_reg_id {
            return Err(AriesVcxCoreError::from_msg(
                AriesVcxCoreErrorKind::InvalidRevocationDetails,
                "ID of revocation registry response does not match requested ID",
            ));
        }

        Ok((
            rev_reg_id.to_string(),
            serde_json::to_string(&reg_delta)?,
            delta_timestamp,
        ))
    }
}

#[async_trait]
impl BaseLedger for VdrProxyLedger {
    async fn sign_and_submit_request(&self, submitter_did: &str, request_json: &str) -> VcxCoreResult<String> {
        let request = self
            ._sign_request(submitter_did, PreparedRequest::from_request_json(request_json)?)
            .await?;
        self.client.post(request).await.map_err(|err| err.into())
    }

    async fn submit_request(&self, request_json: &str) -> VcxCoreResult<String> {
        let request = PreparedRequest::from_request_json(request_json)?;
        self.client.post(request).await.map_err(|err| err.into())
    }

    async fn endorse_transaction(&self, endorser_did: &str, request_json: &str) -> VcxCoreResult<()> {
        Err(unimplemented_method_err("indy_vdr endorse_transaction"))
    }

    async fn set_endorser(&self, submitter_did: &str, request_json: &str, endorser: &str) -> VcxCoreResult<String> {
        Err(unimplemented_method_err("indy_vdr set_endorser"))
    }

    async fn get_txn_author_agreement(&self) -> VcxCoreResult<String> {
        self.client.get_txn_author_agreement().await.map_err(|err| err.into())
    }

    async fn get_nym(&self, did: &str) -> VcxCoreResult<String> {
        self.client.get_nym(did).await.map_err(|err| err.into())
    }

    async fn publish_nym(
        &self,
        submitter_did: &str,
        target_did: &str,
        verkey: Option<&str>,
        data: Option<&str>,
        role: Option<&str>,
    ) -> VcxCoreResult<String> {
        let request = self
            ._sign_request(
                submitter_did,
                self.request_builder()?.build_nym_request(
                    &DidValue(submitter_did.to_string()),
                    &DidValue(target_did.to_string()),
                    verkey.map(String::from),
                    data.map(String::from),
                    role.map(String::from),
                )?,
            )
            .await?;
        self.client.post(request).await.map_err(|err| err.into())
    }

    async fn get_schema(&self, schema_id: &str, submitter_did: Option<&str>) -> VcxCoreResult<String> {
        let response = self.client.get_schema(schema_id).await?;

        let schema = Self::_process_schema_response(&response)?;

        Ok(serde_json::to_string(&schema)?)
    }

    async fn get_cred_def(&self, cred_def_id: &str, submitter_did: Option<&str>) -> VcxCoreResult<String> {
        self.client.get_cred_def(cred_def_id).await.map_err(|err| err.into())
    }

    async fn get_attr(&self, target_did: &str, attr_name: &str) -> VcxCoreResult<String> {
        self.client
            .get_attrib(target_did, attr_name)
            .await
            .map_err(|err| err.into())
    }

    async fn add_attr(&self, target_did: &str, attrib_json: &str) -> VcxCoreResult<String> {
        let request = self._build_attrib_request(target_did, target_did, Some(attrib_json))?;
        let request = _append_txn_author_agreement_to_request(request).await?;
        let request = self._sign_request(target_did, request).await?;
        self.client.post(request).await.map_err(|err| err.into())
    }

    async fn get_rev_reg_def_json(&self, rev_reg_id: &str) -> VcxCoreResult<String> {
        self.client.get_rev_reg_def(rev_reg_id).await.map_err(|err| err.into())
    }

    async fn get_rev_reg_delta_json(
        &self,
        rev_reg_id: &str,
        from: Option<u64>,
        to: Option<u64>,
    ) -> VcxCoreResult<(String, String, u64)> {
        let request = self.request_builder()?.build_get_revoc_reg_delta_request(
            None,
            &RevocationRegistryId::from_str(rev_reg_id)?,
            from.map(|x| x as i64),
            to.map_or(OffsetDateTime::now_utc().unix_timestamp() as i64, |x| x as i64),
        )?;

        let response = self.client.post(request).await?;

        Self::_process_rev_reg_delta_response(&response, rev_reg_id)
    }

    async fn get_rev_reg(&self, rev_reg_id: &str, timestamp: u64) -> VcxCoreResult<(String, String, u64)> {
        Err(unimplemented_method_err("indy_vdr get_rev_reg"))
    }

    async fn get_ledger_txn(&self, seq_no: i32, submitter_did: Option<&str>) -> VcxCoreResult<String> {
        // self.client.get_txn(subledger, seq_no).await?;
        Err(unimplemented_method_err("indy_vdr get_ledger_txn"))
    }

    async fn build_schema_request(&self, submitter_did: &str, schema_json: &str) -> VcxCoreResult<String> {
        Err(unimplemented_method_err("indy_vdr build_schema_request"))
    }

    async fn publish_schema(
        &self,
        schema_json: &str,
        submitter_did: &str,
        endorser_did: Option<String>,
    ) -> VcxCoreResult<()> {
        Err(unimplemented_method_err("indy_vdr publish_cred_def"))
    }

    async fn publish_cred_def(&self, cred_def_json: &str, submitter_did: &str) -> VcxCoreResult<()> {
        Err(unimplemented_method_err("indy_vdr publish_rev_reg_def"))
    }

    async fn publish_rev_reg_def(&self, rev_reg_def: &str, submitter_did: &str) -> VcxCoreResult<()> {
        Err(unimplemented_method_err("indy_vdr publish_rev_reg_def"))
    }

    async fn publish_rev_reg_delta(
        &self,
        rev_reg_id: &str,
        rev_reg_entry_json: &str,
        submitter_did: &str,
    ) -> VcxCoreResult<()> {
        Err(unimplemented_method_err("indy_vdr publish_rev_reg_delta"))
    }
}

#[cfg(test)]
mod unit_tests {}
