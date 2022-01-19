use std::str::FromStr;

use cosmrs::rpc::endpoint::abci_query;
use cosmrs::rpc::endpoint::abci_query::Response as QueryResponse;
use indy_api_types::errors::{IndyResult, IndyErrorKind, IndyResultExt};
use log_derive::logfn;

use crate::domain::cheqd_ledger::cheqd::v1::messages::{
    VerificationMethod,
    MsgCreateDidPayload, MsgCreateDidResponse,
    MsgUpdateDidPayload, MsgUpdateDidResponse,
};
use crate::domain::cheqd_ledger::cheqd::v1::queries::{QueryGetDidResponse, StateValue};
use crate::domain::cheqd_ledger::cheqd::v1::messages::{
    MsgCreateSchema,
    MsgCreateCredDef,
};

use crate::domain::cheqd_ledger::CheqdProto;
use crate::services::CheqdLedgerService;
use crate::utils::cheqd_crypto::check_proofs;
use crate::utils::cheqd_ledger::{make_verification_id, make_base58_btc, VERKEY_TYPE};
use std::collections::HashMap;
use crate::domain::cheqd_ledger::cheqd::v1::models::Did;
use indy_api_types::IndyError;


impl CheqdLedgerService {
    #[logfn(Info)]
    pub(crate) fn cheqd_build_msg_create_did(
        &self,
        did: &str,
        verkey: &str,
    ) -> IndyResult<Vec<u8>> {
        let verif_method_alias = make_verification_id(did);

        let verification_method = VerificationMethod::new(
            verif_method_alias.clone(),
            VERKEY_TYPE.to_string(),
            did.to_string(),
            HashMap::new(),
            make_base58_btc(verkey));

        MsgCreateDidPayload::new(
            Vec::new(),
            did.to_string(),
            vec!(did.to_string()),
            vec!(verification_method),
            vec!(verif_method_alias),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        )
            .to_proto_bytes()
    }

    #[logfn(Info)]
    pub(crate) fn cheqd_parse_msg_create_did_resp(
        &self,
        resp: &str,
    ) -> IndyResult<String> {
        self.parse_msg_resp::<MsgCreateDidResponse>(resp)
    }

    #[logfn(Info)]
    pub(crate) fn cheqd_build_msg_update_did(
        &self,
        did: &str,
        verkey: &str,
        version_id: &str,
    ) -> IndyResult<Vec<u8>> {
        let verif_method_alias = make_verification_id(did);

        let verification_method = VerificationMethod::new(
            verif_method_alias.clone(),
            VERKEY_TYPE.to_string(),
            did.to_string(),
            HashMap::new(),
            make_base58_btc(verkey));

        MsgUpdateDidPayload::new(
            Vec::new(),
            did.to_string(),
            vec!(did.to_string()),
            vec!(verification_method),
            vec!(verif_method_alias),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            version_id.to_string(),
        )
            .to_proto_bytes()
    }

    #[logfn(Info)]
    pub(crate) fn cheqd_parse_msg_update_did_resp(
        &self,
        resp: &str,
    ) -> IndyResult<String> {
        self.parse_msg_resp::<MsgUpdateDidResponse>(resp)
    }

    #[logfn(Info)]
    pub(crate) fn cheqd_build_query_get_did(
        &self,
        did: &str,
    ) -> IndyResult<String> {
        let query_data = format!("did:{}", did).as_bytes().to_vec();
        let path = format!("/store/cheqd/key");
        let path = cosmrs::tendermint::abci::Path::from_str(&path)?;
        let req = abci_query::Request::new(Some(path), query_data, None, true);
        json_string_result!(req)
    }

    #[logfn(Info)]
    pub(crate) fn cheqd_parse_query_get_did_resp(
        &self,
        resp: &str,
    ) -> IndyResult<String> {
        // Step 1 - make a general Response object
        let resp: QueryResponse = serde_json::from_str(resp).to_indy(
            IndyErrorKind::InvalidStructure,
            "Cannot deserialize response after requesting DID",
        )?;
        // Step 2 - Get state value and MAke StateValue object from it
        let result_state_value = if !resp.response.value.is_empty() {
            StateValue::from_proto_bytes(&resp.response.value)?
        } else {
            return Err(IndyError::from_msg(
                IndyErrorKind::InvalidStructure,
                "Response cannot have empty value. ",
            ));
        };
        // Step 3 - check proofs
        check_proofs(&resp)?;
        // Step 4 - get state Data object
        let state_data = result_state_value.data.ok_or(
            IndyError::from_msg(
                IndyErrorKind::InvalidStructure,
                "Data field of StateValue should be placed"))?;
        match state_data.type_url.as_str() {
            "/cheqdid.cheqdnode.cheqd.v1.Did" => {
                // Create QueryGetDidResponse object from json
                let did_response = QueryGetDidResponse::new(
                    Some(Did::from_proto_bytes(state_data.value.as_slice())?),
                    result_state_value.metadata);
                // Make json String
                let json_result = serde_json::to_string(&did_response).to_indy(
                    IndyErrorKind::InvalidState,
                    "Cannot serialize QueryGetDidResponse object",
                )?;
                Ok(json_result)
            }
            _ => Err(IndyError::from_msg(IndyErrorKind::InvalidStructure,
                                         "Did structure is expected as data for StateValue here."))
        }
    }

    #[logfn(Info)]
    pub(crate) fn cheqd_build_msg_create_schema(
        &self,
        _did: &str,
        _data: MsgCreateSchema,
    ) -> IndyResult<Vec<u8>> {
        Ok(Vec::new())
    }

    #[logfn(Info)]
    pub(crate) fn cheqd_build_msg_create_cred_def(
        &self,
        _did: &str,
        _data: MsgCreateCredDef,
    ) -> IndyResult<Vec<u8>> {
        Ok(Vec::new())
    }

    #[logfn(Info)]
    pub(crate) fn cheqd_build_query_get_schema(
        &self,
        _id: &str,
    ) -> IndyResult<String> {
        Ok(String::new())
    }

    #[logfn(Info)]
    pub(crate) fn cheqd_parse_query_get_schema_resp(
        &self,
        _resp: &str,
    ) -> IndyResult<String> {
        Ok(String::new())
    }

    #[logfn(Info)]
    pub(crate) fn cheqd_build_query_get_cred_def(
        &self,
        _id: &str,
    ) -> IndyResult<String> {
        Ok(String::new())
    }

    #[logfn(Info)]
    pub(crate) fn cheqd_parse_query_get_cred_def_resp(
        &self,
        _resp: &str,
    ) -> IndyResult<String> {
        Ok(String::new())
    }
}
