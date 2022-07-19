use std::collections::HashMap;

use indy::cache;
use futures::executor::block_on;
use futures::future::TryFutureExt;
use indy::ledger;
use serde_json;

use crate::{settings, utils};
use crate::error::prelude::*;
use crate::libindy::utils::pool::get_pool_handle;
use crate::libindy::utils::signus::create_and_store_my_did;
use crate::libindy::utils::wallet::get_wallet_handle;
use crate::libindy::utils::mocks::pool_mocks::PoolMocks;
use crate::messages::connection::did::Did;
use crate::messages::connection::service::FullService;
use crate::utils::constants::SUBMIT_SCHEMA_RESPONSE;
use crate::utils::random::generate_random_did;

pub async fn multisign_request(did: &str, request: &str) -> VcxResult<String> {
    ledger::multi_sign_request(get_wallet_handle(), did, request)
        .map_err(VcxError::from)
        .await
}

pub async fn libindy_sign_request(did: &str, request: &str) -> VcxResult<String> {
    ledger::sign_request(get_wallet_handle(), did, request)
        .map_err(VcxError::from)
        .await
}

pub async fn libindy_sign_and_submit_request(issuer_did: &str, request_json: &str) -> VcxResult<String> {
    trace!("libindy_sign_and_submit_request >>> issuer_did: {}, request_json: {}", issuer_did, request_json);
    if settings::indy_mocks_enabled() { return Ok(r#"{"rc":"success"}"#.to_string()); }
    if PoolMocks::has_pool_mock_responses() {
        warn!("libindy_sign_and_submit_request >> retrieving pool mock response");
        return Ok(PoolMocks::get_next_pool_response());
    };

    let pool_handle = get_pool_handle()?;
    let wallet_handle = get_wallet_handle();

    ledger::sign_and_submit_request(pool_handle, wallet_handle, issuer_did, request_json)
        .map_err(VcxError::from)
        .await
}

pub async fn libindy_submit_request(request_json: &str) -> VcxResult<String> {
    trace!("libindy_submit_request >>> request_json: {}", request_json);
    let pool_handle = get_pool_handle()?;

    ledger::submit_request(pool_handle, request_json)
        .map_err(VcxError::from)
        .await
}

pub async fn libindy_build_schema_request(submitter_did: &str, data: &str) -> VcxResult<String> {
    trace!("libindy_build_schema_request >>> submitter_did: {}, data: {}", submitter_did, data);
    ledger::build_schema_request(submitter_did, data)
        .map_err(VcxError::from)
        .await
}

pub async fn libindy_build_create_credential_def_txn(submitter_did: &str,
                                               credential_def_json: &str) -> VcxResult<String> {
    trace!("libindy_build_create_credential_def_txn >>> submitter_did: {}, credential_def_json: {}", submitter_did, credential_def_json);
    ledger::build_cred_def_request(submitter_did, credential_def_json)
        .map_err(VcxError::from)
        .await
}

pub async fn libindy_get_txn_author_agreement() -> VcxResult<String> {
    if settings::indy_mocks_enabled() { return Ok(utils::constants::DEFAULT_AUTHOR_AGREEMENT.to_string()); }

    let did = generate_random_did();

    let get_author_agreement_request = ledger::build_get_txn_author_agreement_request(Some(&did), None)
        .await?;

    let get_author_agreement_response = libindy_submit_request(&get_author_agreement_request).await?;

    let get_author_agreement_response = serde_json::from_str::<serde_json::Value>(&get_author_agreement_response)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidLedgerResponse, format!("{:?}", err)))?;

    let mut author_agreement_data = get_author_agreement_response["result"]["data"].as_object()
        .map_or(json!({}), |data| json!(data));

    let get_acceptance_mechanism_request = ledger::build_get_acceptance_mechanisms_request(Some(&did), None, None)
        .await?;

    let get_acceptance_mechanism_response = libindy_submit_request(&get_acceptance_mechanism_request).await?;

    let get_acceptance_mechanism_response = serde_json::from_str::<serde_json::Value>(&get_acceptance_mechanism_response)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidLedgerResponse, format!("{:?}", err)))?;

    if let Some(aml) = get_acceptance_mechanism_response["result"]["data"]["aml"].as_object() {
        author_agreement_data["aml"] = json!(aml);
    }

    Ok(author_agreement_data.to_string())
}

pub async fn append_txn_author_agreement_to_request(request_json: &str) -> VcxResult<String> {
    trace!("append_txn_author_agreement_to_request >>> request_json: ...");
    if let Some(author_agreement) = utils::author_agreement::get_txn_author_agreement()? {
        ledger::append_txn_author_agreement_acceptance_to_request(request_json,
                                                                  author_agreement.text.as_ref().map(String::as_str),
                                                                  author_agreement.version.as_ref().map(String::as_str),
                                                                  author_agreement.taa_digest.as_ref().map(String::as_str),
                                                                  &author_agreement.acceptance_mechanism_type,
                                                                  author_agreement.time_of_acceptance)
            .map_err(VcxError::from)
            .await
    } else {
        Ok(request_json.to_string())
    }
}

pub async fn libindy_build_auth_rules_request(submitter_did: &str, data: &str) -> VcxResult<String> {
    ledger::build_auth_rules_request(submitter_did, data)
        .map_err(VcxError::from)
        .await
}

pub async fn libindy_build_attrib_request(submitter_did: &str, target_did: &str, hash: Option<&str>, raw: Option<&str>, enc: Option<&str>) -> VcxResult<String> {
    ledger::build_attrib_request(submitter_did, target_did, hash, raw, enc)
        .map_err(VcxError::from)
        .await
}

pub async fn libindy_build_get_auth_rule_request(submitter_did: Option<&str>, txn_type: Option<&str>, action: Option<&str>, field: Option<&str>,
                                           old_value: Option<&str>, new_value: Option<&str>) -> VcxResult<String> {
    ledger::build_get_auth_rule_request(submitter_did, txn_type, action, field, old_value, new_value)
        .map_err(VcxError::from)
        .await
}

pub async fn libindy_build_get_nym_request(submitter_did: Option<&str>, did: &str) -> VcxResult<String> {
    ledger::build_get_nym_request(submitter_did, did)
        .map_err(VcxError::from)
        .await
}

pub async fn libindy_build_nym_request(submitter_did: &str, target_did: &str, verkey: Option<&str>, data: Option<&str>, role: Option<&str>) -> VcxResult<String> {
    if PoolMocks::has_pool_mock_responses() {
        warn!("libindy_build_nym_request >> retrieving pool mock response");
        Ok(PoolMocks::get_next_pool_response())
    } else {
        ledger::build_nym_request(submitter_did, target_did, verkey, data, role)
            .map_err(VcxError::from)
            .await
    }
}

pub mod auth_rule {
    use std::sync::Mutex;
    use std::sync::Once;

    use crate::libindy;

    use super::*;

    /**
    Structure for parsing GET_AUTH_RULE response
    # parameters
    result - the payload containing data relevant to the GET_AUTH_RULE transaction
     */
    #[derive(Serialize, Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct GetAuthRuleResponse {
        pub result: GetAuthRuleResult,
    }

    /**
    Structure of the result value within the GAT_AUTH_RULE response
     # parameters
    identifier - The DID this request was submitted from
    req_id - Unique ID number of the request with transaction
    txn_type - the type of transaction that was submitted
    data - A key:value map with the action id as the key and the auth rule as the value
     */
    #[derive(Serialize, Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct GetAuthRuleResult {
        pub identifier: String,
        pub req_id: u64,
        // This is to change the json key to adhear to the functionality on ledger
        #[serde(rename = "type")]
        pub txn_type: String,
        pub data: Vec<AuthRule>,
    }

    /**
    Enum of the constraint type within the GAT_AUTH_RULE result data
     # parameters
    Role - The final constraint
    And - Combine multiple constraints all of them must be met
    Or - Combine multiple constraints any of them must be met
    Forbidden - action is forbidden
     */
    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(tag = "constraint_id")]
    pub enum Constraint {
        #[serde(rename = "OR")]
        OrConstraint(CombinationConstraint),
        #[serde(rename = "AND")]
        AndConstraint(CombinationConstraint),
        #[serde(rename = "ROLE")]
        RoleConstraint(RoleConstraint),
        #[serde(rename = "FORBIDDEN")]
        ForbiddenConstraint(ForbiddenConstraint),
    }

    /**
    The final constraint
     # parameters
    sig_count - The number of signatures required to execution action
    role - The role which the user must have to execute the action.
    metadata -  An additional parameters of the constraint (contains transaction FEE cost).
    need_to_be_owner - The flag specifying if a user must be an owner of the transaction.
     */
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct RoleConstraint {
        pub sig_count: Option<u32>,
        pub role: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub metadata: Option<Metadata>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub need_to_be_owner: Option<bool>,
    }

    /**
    The empty constraint means that action is forbidden
     */
    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(deny_unknown_fields)]
    pub struct ForbiddenConstraint {}

    /**
    The constraint metadata
     # parameters
    fees - The action cost
     */
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct Metadata {
        pub fees: Option<String>,
    }

    /**
    Combine multiple constraints
     # parameters
    auth_constraints - The type of the combination
     */
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct CombinationConstraint {
        pub auth_constraints: Vec<Constraint>,
    }

    /* Map contains default Auth Rules set on the Ledger*/
    lazy_static! {
        static ref AUTH_RULES: Mutex<Vec<AuthRule>> = Default::default();
    }

    /* Helper structure to store auth rule set on the Ledger */
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct AuthRule {
        auth_action: String,
        auth_type: String,
        field: String,
        old_value: Option<String>,
        new_value: Option<String>,
        constraint: Constraint,
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
    pub struct Action {
        pub auth_type: String,
        pub auth_action: String,
        pub field: String,
        pub old_value: Option<String>,
        pub new_value: Option<String>,
    }

    async fn _send_auth_rules(submitter_did: &str, data: &Vec<AuthRule>) -> VcxResult<()> {
        let data = serde_json::to_string(&data)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot serialize auth rules: {:?}", err)))?;

        let auth_rules_request = libindy_build_auth_rules_request(submitter_did, &data).await?;

        let response = ledger::sign_and_submit_request(get_pool_handle()?, get_wallet_handle(), submitter_did, &auth_rules_request)
            .await?;

        let response: serde_json::Value = serde_json::from_str(&response)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidLedgerResponse, format!("{:?}", err)))?;

        match response["op"].as_str().unwrap_or_default() {
            "REPLY" => Ok(()),
            _ => Err(VcxError::from(VcxErrorKind::InvalidLedgerResponse))
        }
    }

    async fn _get_default_ledger_auth_rules() {
        lazy_static! {
            static ref GET_DEFAULT_AUTH_CONSTRAINTS: Once = Once::new();

        }

        GET_DEFAULT_AUTH_CONSTRAINTS.call_once(|| {
            let get_auth_rule_request = block_on(indy::ledger::build_get_auth_rule_request(None, None, None, None, None, None)).unwrap();
            let get_auth_rule_response = block_on(libindy::utils::ledger::libindy_submit_request(&get_auth_rule_request)).unwrap();

            let response: GetAuthRuleResponse = serde_json::from_str(&get_auth_rule_response)
                .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidLedgerResponse, err)).unwrap();

            let mut auth_rules = AUTH_RULES.lock().unwrap();
            *auth_rules = response.result.data;
        })
    }

    pub async fn get_action_auth_rule(action: (&str, &str, &str, Option<&str>, Option<&str>)) -> VcxResult<String> {
        let (txn_type, action, field, old_value, new_value) = action;

        if settings::indy_mocks_enabled() { return Ok(json!({"result":{"data":[{"new_value":"0","constraint":{"need_to_be_owner":false,"sig_count":1,"metadata":{"fees":txn_type},"role":"0","constraint_id":"ROLE"},"field":"role","auth_type":"1","auth_action":"ADD"}],"identifier":"LibindyDid111111111111","auth_action":"ADD","new_value":"0","reqId":15616,"auth_type":"1","type":"121","field":"role"},"op":"REPLY"}).to_string()); }

        let did = generate_random_did();


        let request = libindy_build_get_auth_rule_request(Some(&did), Some(txn_type), Some(action), Some(field), old_value, new_value).await?;

        let response_json = libindy_submit_request(&request).await?;

        let response: serde_json::Value = serde_json::from_str(&response_json)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidLedgerResponse, format!("{:?}", err)))?;

        match response["op"].as_str().unwrap_or_default() {
            "REPLY" => Ok(response_json),
            _ => Err(VcxError::from(VcxErrorKind::InvalidLedgerResponse))
        }
    }
}

pub async fn get_nym(did: &str) -> VcxResult<String> {
    let submitter_did = generate_random_did();

    let get_nym_req = libindy_build_get_nym_request(Some(&submitter_did), &did).await?;
    libindy_submit_request(&get_nym_req).await
}

pub async fn get_role(did: &str) -> VcxResult<String> {
    if settings::indy_mocks_enabled() { return Ok(settings::DEFAULT_ROLE.to_string()); }

    let get_nym_resp = get_nym(&did).await?;
    let get_nym_resp: serde_json::Value = serde_json::from_str(&get_nym_resp)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidLedgerResponse, format!("{:?}", err)))?;
    let data: serde_json::Value = serde_json::from_str(&get_nym_resp["result"]["data"].as_str().unwrap_or("{}"))
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidLedgerResponse, format!("{:?}", err)))?;
    let role = data["role"].as_str().unwrap_or("null").to_string();
    Ok(role)
}

pub fn parse_response(response: &str) -> VcxResult<Response> {
    serde_json::from_str::<Response>(response)
        .to_vcx(VcxErrorKind::InvalidJson, "Cannot deserialize transaction response")
}

pub async fn libindy_get_schema(submitter_did: &str, schema_id: &str) -> VcxResult<String> {
    let pool_handle = get_pool_handle()?;
    let wallet_handle = get_wallet_handle();

    cache::get_schema(pool_handle, wallet_handle, submitter_did, schema_id, "{}")
        .await
        .map_err(VcxError::from)
}

pub async fn libindy_build_get_cred_def_request(submitter_did: Option<&str>, cred_def_id: &str) -> VcxResult<String> {
    ledger::build_get_cred_def_request(submitter_did, cred_def_id)
        .await
        .map_err(VcxError::from)
}

pub async fn libindy_get_cred_def(cred_def_id: &str) -> VcxResult<String> {
    let pool_handle = get_pool_handle()?;
    let wallet_handle = get_wallet_handle();
    let submitter_did = generate_random_did();
    trace!("libindy_get_cred_def >>> pool_handle: {}, wallet_handle: {:?}, submitter_did: {}", pool_handle, wallet_handle, submitter_did);

    cache::get_cred_def(pool_handle, wallet_handle, &submitter_did, cred_def_id, "{}")
        .await
        .map_err(VcxError::from)
}

pub async fn set_endorser(request: &str, endorser: &str) -> VcxResult<String> {
    if settings::indy_mocks_enabled() { return Ok(utils::constants::REQUEST_WITH_ENDORSER.to_string()); }

    let did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID)?;

    let request = ledger::append_request_endorser(request, endorser)
        .await?;

    multisign_request(&did, &request).await
}

pub async fn endorse_transaction(transaction_json: &str) -> VcxResult<()> {
    //TODO Potentially VCX should handle case when endorser would like to pay fee
    if settings::indy_mocks_enabled() { return Ok(()); }

    let submitter_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID)?;

    _verify_transaction_can_be_endorsed(transaction_json, &submitter_did)?;

    let transaction = multisign_request(&submitter_did, transaction_json).await?;
    let response = libindy_submit_request(&transaction).await?;

    match parse_response(&response)? {
        Response::Reply(_) => Ok(()),
        Response::Reject(res) | Response::ReqNACK(res) => Err(VcxError::from_msg(VcxErrorKind::PostMessageFailed, format!("{:?}", res.reason))),
    }
}

fn _verify_transaction_can_be_endorsed(transaction_json: &str, _did: &str) -> VcxResult<()> {
    let transaction: Request = serde_json::from_str(transaction_json)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("{:?}", err)))?;

    let transaction_endorser = transaction.endorser
        .ok_or(VcxError::from_msg(VcxErrorKind::InvalidJson, "Transaction cannot be endorsed: endorser DID is not set."))?;

    if transaction_endorser != _did {
        return Err(VcxError::from_msg(VcxErrorKind::InvalidJson,
                                      format!("Transaction cannot be endorsed: transaction endorser DID `{}` and sender DID `{}` are different", transaction_endorser, _did)));
    }

    let identifier = transaction.identifier.as_str();
    if transaction.signature.is_none() && !transaction.signatures.as_ref().map(|signatures| signatures.contains_key(identifier)).unwrap_or(false) {
        return Err(VcxError::from_msg(VcxErrorKind::InvalidJson,
                                      format!("Transaction cannot be endorsed: the author must sign the transaction.")));
    }

    Ok(())
}

pub async fn build_attrib_request(submitter_did: &str, target_did: &str, hash: Option<&str>, raw: Option<&str>, enc: Option<&str>) -> VcxResult<String> {
    trace!("build_attrib_request >>> submitter_did: {}, target_did: {}, hash: {:?}, raw: {:?}, enc: {:?}", submitter_did, target_did, hash, raw, enc);
    if settings::indy_mocks_enabled() {
        return Ok("{}".into());
    }
    let request = libindy_build_attrib_request(submitter_did, target_did, hash, raw, enc).await?;
    let request = append_txn_author_agreement_to_request(&request).await?;

    Ok(request)
}

pub async fn add_attr(did: &str, attrib_json: &str) -> VcxResult<String> {
    trace!("add_attr >>> did: {}, attrib_json: {}", did, attrib_json);
    let attrib_req = build_attrib_request(&did, &did, None, Some(attrib_json), None).await?;
    libindy_sign_and_submit_request(&did, &attrib_req).await
}

pub async fn get_attr(did: &str, attr_name: &str) -> VcxResult<String> {
    let get_attrib_req = ledger::build_get_attrib_request(None, &did, Some(attr_name), None, None).await?;
    libindy_submit_request(&get_attrib_req).await
}

pub async fn get_service(did: &Did) -> VcxResult<FullService> {
    let attr_resp = get_attr(&did.to_string(), "service").await?;
    let data = get_data_from_response(&attr_resp)?;
    let ser_service = match data["service"].as_str() {
        Some(ser_service) => ser_service.to_string(),
        None => {
            warn!("Failed converting service read from ledger {:?} to string, falling back to new single-serialized format", data["service"]);
            data["service"].to_string()
        }
    };
    serde_json::from_str(&ser_service)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::SerializationError, format!("Failed to deserialize service read from the ledger: {:?}", err)))
}

pub async fn add_service(did: &str, service: &FullService) -> VcxResult<String> {
    let attrib_json = json!({ "service": service }).to_string();
    add_attr(did, &attrib_json).await
}

fn get_data_from_response(resp: &str) -> VcxResult<serde_json::Value> {
    let resp: serde_json::Value = serde_json::from_str(&resp)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidLedgerResponse, format!("{:?}", err)))?;
    serde_json::from_str(&resp["result"]["data"].as_str().unwrap_or("{}"))
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidLedgerResponse, format!("{:?}", err)))
}

#[cfg(test)]
mod test {
    use std::thread;
    use std::time::Duration;

    use crate::utils::devsetup::*;

    use super::*;

    pub async fn add_service_old(did: &str, service: &FullService) -> VcxResult<String> {
        let ser_service = serde_json::to_string(service)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::SerializationError, format!("Failed to serialize service before writing to ledger: {:?}", err)))?;
        let attrib_json = json!({ "service": ser_service }).to_string();
        add_attr(did, &attrib_json).await
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_verify_transaction_can_be_endorsed() {
        let _setup = SetupDefaults::init();

        // success
        let transaction = r#"{"reqId":1, "identifier": "EbP4aYNeTHL6q385GuVpRV", "signature": "gkVDhwe2", "endorser": "NcYxiDXkpYi6ov5FcYDi1e"}"#;
        assert!(_verify_transaction_can_be_endorsed(transaction, "NcYxiDXkpYi6ov5FcYDi1e").is_ok());

        // no author signature
        let transaction = r#"{"reqId":1, "identifier": "EbP4aYNeTHL6q385GuVpRV", "endorser": "NcYxiDXkpYi6ov5FcYDi1e"}"#;
        assert!(_verify_transaction_can_be_endorsed(transaction, "NcYxiDXkpYi6ov5FcYDi1e").is_err());

        // different endorser did
        let transaction = r#"{"reqId":1, "identifier": "EbP4aYNeTHL6q385GuVpRV", "endorser": "NcYxiDXkpYi6ov5FcYDi1e"}"#;
        assert!(_verify_transaction_can_be_endorsed(transaction, "EbP4aYNeTHL6q385GuVpRV").is_err());
    }

    #[cfg(feature = "pool_tests")]
    #[tokio::test]
    async fn test_endorse_transaction() {
        let _setup = SetupWithWalletAndAgency::init().await;

        let (author_did, _) = add_new_did(None).await;
        let (endorser_did, _) = add_new_did(Some("ENDORSER")).await;

        settings::set_config_value(settings::CONFIG_INSTITUTION_DID, &endorser_did);

        let schema_request = libindy_build_schema_request(&author_did, utils::constants::SCHEMA_DATA).await.unwrap();
        let schema_request = ledger::append_request_endorser(&schema_request, &endorser_did).await.unwrap();
        let schema_request = multisign_request(&author_did, &schema_request).await.unwrap();

        endorse_transaction(&schema_request).await.unwrap();
    }

    #[cfg(feature = "pool_tests")]
    #[tokio::test]
    async fn test_add_get_service() {
        let _setup = SetupWithWalletAndAgency::init().await;

        let did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();
        let expect_service = FullService::default();
        add_service(&did, &expect_service).await.unwrap();
        thread::sleep(Duration::from_millis(50));
        let service = get_service(&Did::new(&did).unwrap()).await.unwrap();

        assert_eq!(expect_service, service)
    }

    #[cfg(feature = "pool_tests")]
    #[tokio::test]
    async fn test_add_get_service_old() {
        let _setup = SetupWithWalletAndAgency::init().await;

        let did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();
        let expect_service = FullService::default();
        add_service_old(&did, &expect_service).await.unwrap();
        thread::sleep(Duration::from_millis(50));
        let service = get_service(&Did::new(&did).unwrap()).await.unwrap();

        assert_eq!(expect_service, service)
    }
}


#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    pub req_id: u64,
    pub identifier: String,
    pub signature: Option<String>,
    pub signatures: Option<HashMap<String, String>>,
    pub endorser: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "op")]
pub enum Response {
    #[serde(rename = "REQNACK")]
    ReqNACK(Reject),
    #[serde(rename = "REJECT")]
    Reject(Reject),
    #[serde(rename = "REPLY")]
    Reply(Reply),
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Reject {
    pub reason: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Reply {
    ReplyV0(ReplyV0),
    ReplyV1(ReplyV1),
}

#[derive(Debug, Deserialize)]
pub struct ReplyV0 {
    pub result: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct ReplyV1 {
    pub data: ReplyDataV1,
}

#[derive(Debug, Deserialize)]
pub struct ReplyDataV1 {
    pub result: serde_json::Value,
}

pub async fn publish_txn_on_ledger(req: &str) -> VcxResult<String> {
    debug!("publish_txn_on_ledger(req: {}", req);
    if settings::indy_mocks_enabled() {
        return Ok(SUBMIT_SCHEMA_RESPONSE.to_string());
    }
    let did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID)?;
    libindy_sign_and_submit_request(&did, req).await
}

pub async fn add_new_did(role: Option<&str>) -> (String, String) {
    let institution_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();

    let (did, verkey) = create_and_store_my_did(None, None).await.unwrap();
    let mut req_nym = ledger::build_nym_request(&institution_did, &did, Some(&verkey), None, role).await.unwrap();

    req_nym = append_txn_author_agreement_to_request(&req_nym).await.unwrap();

    libindy_sign_and_submit_request(&institution_did, &req_nym).await.unwrap();
    (did, verkey)
}
