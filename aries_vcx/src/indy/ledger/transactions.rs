use vdrtools::{DidValue, Locator, PoolHandle, WalletHandle};

use crate::{
    common::ledger::transactions::{Request, Response},
    errors::error::prelude::*,
    global::settings,
    indy::utils::mocks::pool_mocks::PoolMocks,
    utils,
    utils::{
        constants::{
            rev_def_json, CRED_DEF_ID, CRED_DEF_JSON, CRED_DEF_REQ, REVOC_REG_TYPE, REV_REG_DELTA_JSON, REV_REG_ID,
            REV_REG_JSON, SCHEMA_ID, SCHEMA_JSON, SCHEMA_TXN, SUBMIT_SCHEMA_RESPONSE,
        },
        parse_and_validate,
        random::generate_random_did,
    },
};

pub async fn multisign_request(wallet_handle: WalletHandle, did: &str, request: &str) -> VcxResult<String> {
    let res = Locator::instance()
        .ledger_controller
        .multi_sign_request(wallet_handle, DidValue(did.into()), request.into())
        .await?;

    Ok(res)
}

pub async fn libindy_sign_and_submit_request(
    wallet_handle: WalletHandle,
    pool_handle: PoolHandle,
    issuer_did: &str,
    request_json: &str,
) -> VcxResult<String> {
    trace!(
        "libindy_sign_and_submit_request >>> issuer_did: {}, request_json: {}",
        issuer_did,
        request_json
    );
    if settings::indy_mocks_enabled() {
        return Ok(r#"{"rc":"success"}"#.to_string());
    }
    if PoolMocks::has_pool_mock_responses() {
        warn!("libindy_sign_and_submit_request >> retrieving pool mock response");
        return Ok(PoolMocks::get_next_pool_response());
    };

    let res = Locator::instance()
        .ledger_controller
        .sign_and_submit_request(
            pool_handle,
            wallet_handle,
            DidValue(issuer_did.into()),
            request_json.into(),
        )
        .await?;

    Ok(res)
}

pub async fn libindy_submit_request(pool_handle: PoolHandle, request_json: &str) -> VcxResult<String> {
    trace!("libindy_submit_request >>> request_json: {}", request_json);

    let res = Locator::instance()
        .ledger_controller
        .submit_request(pool_handle, request_json.into())
        .await?;

    Ok(res)
}

pub async fn libindy_build_schema_request(submitter_did: &str, data: &str) -> VcxResult<String> {
    trace!(
        "libindy_build_schema_request >>> submitter_did: {}, data: {}",
        submitter_did,
        data
    );

    let res = Locator::instance()
        .ledger_controller
        .build_schema_request(DidValue(submitter_did.into()), parse_and_validate(data)?)?;

    Ok(res)
}

pub async fn libindy_build_create_credential_def_txn(
    submitter_did: &str,
    credential_def_json: &str,
) -> VcxResult<String> {
    trace!(
        "libindy_build_create_credential_def_txn >>> submitter_did: {}, credential_def_json: {}",
        submitter_did,
        credential_def_json
    );

    let res = Locator::instance()
        .ledger_controller
        .build_cred_def_request(DidValue(submitter_did.into()), parse_and_validate(credential_def_json)?)?;

    Ok(res)
}

pub async fn libindy_get_txn_author_agreement(pool_handle: PoolHandle) -> VcxResult<String> {
    if settings::indy_mocks_enabled() {
        return Ok(utils::constants::DEFAULT_AUTHOR_AGREEMENT.to_string());
    }

    let did = &generate_random_did();

    let get_author_agreement_request = Locator::instance()
        .ledger_controller
        .build_get_txn_author_agreement_request(Some(did.into()), None)?;

    let get_author_agreement_response = libindy_submit_request(pool_handle, &get_author_agreement_request).await?;

    let get_author_agreement_response = serde_json::from_str::<serde_json::Value>(&get_author_agreement_response)
        .map_err(|err| AriesVcxError::from_msg(AriesVcxErrorKind::InvalidLedgerResponse, format!("{:?}", err)))?;

    let mut author_agreement_data = get_author_agreement_response["result"]["data"]
        .as_object()
        .map_or(json!({}), |data| json!(data));

    let get_acceptance_mechanism_request = Locator::instance()
        .ledger_controller
        .build_get_acceptance_mechanisms_request(Some(did.into()), None, None)?;

    let get_acceptance_mechanism_response =
        libindy_submit_request(pool_handle, &get_acceptance_mechanism_request).await?;

    let get_acceptance_mechanism_response =
        serde_json::from_str::<serde_json::Value>(&get_acceptance_mechanism_response)
            .map_err(|err| AriesVcxError::from_msg(AriesVcxErrorKind::InvalidLedgerResponse, format!("{:?}", err)))?;

    if let Some(aml) = get_acceptance_mechanism_response["result"]["data"]["aml"].as_object() {
        author_agreement_data["aml"] = json!(aml);
    }

    Ok(author_agreement_data.to_string())
}

// TODO: remove async
pub async fn append_txn_author_agreement_to_request(request_json: &str) -> VcxResult<String> {
    trace!("append_txn_author_agreement_to_request >>> request_json: ...");

    if let Some(author_agreement) = utils::author_agreement::get_txn_author_agreement()? {
        Locator::instance()
            .ledger_controller
            .append_txn_author_agreement_acceptance_to_request(
                request_json.into(),
                author_agreement.text,
                author_agreement.version,
                author_agreement.taa_digest,
                author_agreement.acceptance_mechanism_type,
                author_agreement.time_of_acceptance,
            )
            .map_err(AriesVcxError::from)
    } else {
        Ok(request_json.to_string())
    }
}

// TODO: remove async
pub async fn libindy_build_attrib_request(
    submitter_did: &str,
    target_did: &str,
    hash: Option<&str>,
    raw: Option<&str>,
    enc: Option<&str>,
) -> VcxResult<String> {
    let res = Locator::instance().ledger_controller.build_attrib_request(
        submitter_did.into(),
        target_did.into(),
        hash.map(|s| s.to_owned()),
        raw.map(serde_json::from_str).transpose()?,
        enc.map(|s| s.to_owned()),
    )?;

    Ok(res)
}

// TODO: remove async
pub async fn libindy_build_get_nym_request(submitter_did: Option<&str>, did: &str) -> VcxResult<String> {
    let res = Locator::instance()
        .ledger_controller
        .build_get_nym_request(submitter_did.map(|s| s.into()), did.into())?;

    Ok(res)
}

pub async fn libindy_build_nym_request(
    submitter_did: &str,
    target_did: &str,
    verkey: Option<&str>,
    data: Option<&str>,
    role: Option<&str>,
) -> VcxResult<String> {
    if PoolMocks::has_pool_mock_responses() {
        warn!("libindy_build_nym_request >> retrieving pool mock response");
        return Ok(PoolMocks::get_next_pool_response());
    }

    let res = Locator::instance()
        .ledger_controller
        .build_nym_request(
            submitter_did.into(),
            target_did.into(),
            verkey.map(|s| s.into()),
            data.map(|s| s.into()),
            role.map(|s| s.into()),
        )
        .await?;

    Ok(res)
}

pub async fn get_nym(pool_handle: PoolHandle, did: &str) -> VcxResult<String> {
    let submitter_did = generate_random_did();

    let get_nym_req = libindy_build_get_nym_request(Some(&submitter_did), did).await?;

    libindy_submit_request(pool_handle, &get_nym_req).await
}

fn parse_response(response: &str) -> VcxResult<Response> {
    serde_json::from_str::<Response>(response).map_err(|err| {
        AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidJson,
            format!("Cannot deserialize object: {}", err),
        )
    })
}

pub async fn libindy_get_schema(
    wallet_handle: WalletHandle,
    pool_handle: PoolHandle,
    submitter_did: &str,
    schema_id: &str,
) -> VcxResult<String> {
    let res = Locator::instance()
        .cache_controller
        .get_schema(
            pool_handle,
            wallet_handle,
            submitter_did.into(),
            vdrtools::SchemaId(schema_id.into()),
            serde_json::from_str("{}")?,
        )
        .await?;

    Ok(res)
}

async fn libindy_get_cred_def(
    wallet_handle: WalletHandle,
    pool_handle: PoolHandle,
    cred_def_id: &str,
) -> VcxResult<String> {
    let submitter_did = &generate_random_did();
    trace!(
        "libindy_get_cred_def >>> pool_handle: {}, wallet_handle: {:?}, submitter_did: {}",
        pool_handle,
        wallet_handle,
        submitter_did
    );

    let res = Locator::instance()
        .cache_controller
        .get_cred_def(
            pool_handle,
            wallet_handle,
            submitter_did.into(),
            cred_def_id.into(),
            serde_json::from_str("{}")?,
        )
        .await?;

    Ok(res)
}

pub async fn set_endorser(
    wallet_handle: WalletHandle,
    submitter_did: &str,
    request: &str,
    endorser: &str,
) -> VcxResult<String> {
    if settings::indy_mocks_enabled() {
        return Ok(utils::constants::REQUEST_WITH_ENDORSER.to_string());
    }

    let request = Locator::instance()
        .ledger_controller
        .append_request_endorser(request.into(), endorser.into())?;

    multisign_request(wallet_handle, submitter_did, &request).await
}

pub async fn endorse_transaction(
    wallet_handle: WalletHandle,
    pool_handle: PoolHandle,
    endorser_did: &str,
    transaction_json: &str,
) -> VcxResult<()> {
    //TODO Potentially VCX should handle case when endorser would like to pay fee
    if settings::indy_mocks_enabled() {
        return Ok(());
    }

    _verify_transaction_can_be_endorsed(transaction_json, endorser_did)?;

    let transaction = multisign_request(wallet_handle, endorser_did, transaction_json).await?;
    let response = libindy_submit_request(pool_handle, &transaction).await?;

    match parse_response(&response)? {
        Response::Reply(_) => Ok(()),
        Response::Reject(res) | Response::ReqNACK(res) => Err(AriesVcxError::from_msg(
            AriesVcxErrorKind::PostMessageFailed,
            format!("{:?}", res.reason),
        )),
    }
}

fn _verify_transaction_can_be_endorsed(transaction_json: &str, _did: &str) -> VcxResult<()> {
    let transaction: Request = serde_json::from_str(transaction_json)
        .map_err(|err| AriesVcxError::from_msg(AriesVcxErrorKind::InvalidJson, format!("{:?}", err)))?;

    let transaction_endorser = transaction.endorser.ok_or(AriesVcxError::from_msg(
        AriesVcxErrorKind::InvalidJson,
        "Transaction cannot be endorsed: endorser DID is not set.",
    ))?;

    if transaction_endorser != _did {
        return Err(AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidJson,
            format!(
                "Transaction cannot be endorsed: transaction endorser DID `{}` and sender DID `{}` are different",
                transaction_endorser, _did
            ),
        ));
    }

    let identifier = transaction.identifier.as_str();
    if transaction.signature.is_none()
        && !transaction
            .signatures
            .as_ref()
            .map(|signatures| signatures.contains_key(identifier))
            .unwrap_or(false)
    {
        return Err(AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidJson,
            "Transaction cannot be endorsed: the author must sign the transaction.".to_string(),
        ));
    }

    Ok(())
}

pub async fn build_attrib_request(
    submitter_did: &str,
    target_did: &str,
    hash: Option<&str>,
    raw: Option<&str>,
    enc: Option<&str>,
) -> VcxResult<String> {
    trace!(
        "build_attrib_request >>> submitter_did: {}, target_did: {}, hash: {:?}, raw: {:?}, enc: {:?}",
        submitter_did,
        target_did,
        hash,
        raw,
        enc
    );
    if settings::indy_mocks_enabled() {
        return Ok("{}".into());
    }
    let request = libindy_build_attrib_request(submitter_did, target_did, hash, raw, enc).await?;
    let request = append_txn_author_agreement_to_request(&request).await?;

    Ok(request)
}

pub async fn add_attr(
    wallet_handle: WalletHandle,
    pool_handle: PoolHandle,
    did: &str,
    attrib_json: &str,
) -> VcxResult<String> {
    trace!("add_attr >>> did: {}, attrib_json: {}", did, attrib_json);
    let attrib_req = build_attrib_request(did, did, None, Some(attrib_json), None).await?;
    libindy_sign_and_submit_request(wallet_handle, pool_handle, did, &attrib_req).await
}

pub async fn get_attr(pool_handle: PoolHandle, did: &str, attr_name: &str) -> VcxResult<String> {
    let get_attrib_req = Locator::instance().ledger_controller.build_get_attrib_request(
        None,
        did.into(),
        Some(attr_name.into()),
        None,
        None,
    )?;

    libindy_submit_request(pool_handle, &get_attrib_req).await
}

pub async fn sign_and_submit_to_ledger(
    wallet_handle: WalletHandle,
    pool_handle: PoolHandle,
    submitter_did: &str,
    req: &str,
) -> VcxResult<String> {
    debug!(
        "sign_and_submit_to_ledger(submitter_did: {}, req: {}",
        submitter_did, req
    );
    if settings::indy_mocks_enabled() {
        return Ok(SUBMIT_SCHEMA_RESPONSE.to_string());
    }
    let response = libindy_sign_and_submit_request(wallet_handle, pool_handle, submitter_did, req).await?;
    debug!("sign_and_submit_to_ledger >>> response: {}", &response);
    Ok(response)
}

pub async fn libindy_build_revoc_reg_def_request(submitter_did: &str, rev_reg_def_json: &str) -> VcxResult<String> {
    if settings::indy_mocks_enabled() {
        return Ok("".to_string());
    }

    let res = Locator::instance()
        .ledger_controller
        .build_revoc_reg_def_request(submitter_did.into(), parse_and_validate(rev_reg_def_json)?)?;

    Ok(res)
}

// TODO: remove async
pub async fn libindy_build_revoc_reg_entry_request(
    submitter_did: &str,
    rev_reg_id: &str,
    rev_def_type: &str,
    value: &str,
) -> VcxResult<String> {
    if settings::indy_mocks_enabled() {
        return Ok("".to_string());
    }

    let res = Locator::instance().ledger_controller.build_revoc_reg_entry_request(
        submitter_did.into(),
        rev_reg_id.into(),
        rev_def_type.into(),
        serde_json::from_str(value)?,
    )?;

    Ok(res)
}

pub async fn libindy_build_get_revoc_reg_def_request(submitter_did: &str, rev_reg_id: &str) -> VcxResult<String> {
    let res = Locator::instance()
        .ledger_controller
        .build_get_revoc_reg_def_request(Some(submitter_did.into()), rev_reg_id.into())?;

    Ok(res)
}

// TODO: remove async
pub async fn libindy_parse_get_revoc_reg_def_response(rev_reg_def_json: &str) -> VcxResult<(String, String)> {
    let res = Locator::instance()
        .ledger_controller
        .parse_revoc_reg_def_response(rev_reg_def_json.into())?;

    Ok(res)
}

pub async fn libindy_build_get_revoc_reg_delta_request(
    submitter_did: &str,
    rev_reg_id: &str,
    from: i64,
    to: i64,
) -> VcxResult<String> {
    let res = Locator::instance()
        .ledger_controller
        .build_get_revoc_reg_delta_request(Some(submitter_did.into()), rev_reg_id.into(), Some(from), to)?;

    Ok(res)
}

async fn libindy_build_get_revoc_reg_request(
    submitter_did: &str,
    rev_reg_id: &str,
    timestamp: u64,
) -> VcxResult<String> {
    let res = Locator::instance().ledger_controller.build_get_revoc_reg_request(
        Some(submitter_did.into()),
        rev_reg_id.into(),
        timestamp as i64,
    )?;

    Ok(res)
}

async fn libindy_parse_get_revoc_reg_response(get_cred_def_resp: &str) -> VcxResult<(String, String, u64)> {
    let res = Locator::instance()
        .ledger_controller
        .parse_revoc_reg_response(get_cred_def_resp.into())?;

    Ok(res)
}

pub async fn libindy_parse_get_revoc_reg_delta_response(
    get_rev_reg_delta_response: &str,
) -> VcxResult<(String, String, u64)> {
    let res = Locator::instance()
        .ledger_controller
        .parse_revoc_reg_delta_response(get_rev_reg_delta_response.into())?;

    Ok(res)
}

pub async fn build_schema_request(submitter_did: &str, schema: &str) -> VcxResult<String> {
    trace!(
        "build_schema_request >>> submitter_did: {}, schema: {}",
        submitter_did,
        schema
    );

    if settings::indy_mocks_enabled() {
        return Ok(SCHEMA_TXN.to_string());
    }

    let request = libindy_build_schema_request(submitter_did, schema).await?;

    let request = append_txn_author_agreement_to_request(&request).await?;

    Ok(request)
}

pub async fn build_rev_reg_request(issuer_did: &str, rev_reg_def_json: &str) -> VcxResult<String> {
    if settings::indy_mocks_enabled() {
        debug!("build_rev_reg_request >>> returning mocked value");
        return Ok("".to_string());
    }

    let rev_reg_def_req = libindy_build_revoc_reg_def_request(issuer_did, rev_reg_def_json).await?;
    let rev_reg_def_req = append_txn_author_agreement_to_request(&rev_reg_def_req).await?;
    Ok(rev_reg_def_req)
}

pub async fn get_rev_reg_def_json(pool_handle: PoolHandle, rev_reg_id: &str) -> VcxResult<(String, String)> {
    if settings::indy_mocks_enabled() {
        debug!("get_rev_reg_def_json >>> returning mocked value");
        return Ok((REV_REG_ID.to_string(), rev_def_json()));
    }

    let submitter_did = generate_random_did();

    let req = libindy_build_get_revoc_reg_def_request(&submitter_did, rev_reg_id).await?;
    let res = libindy_submit_request(pool_handle, &req).await?;

    libindy_parse_get_revoc_reg_def_response(&res).await
}

pub async fn build_rev_reg_delta_request(
    issuer_did: &str,
    rev_reg_id: &str,
    rev_reg_entry_json: &str,
) -> VcxResult<String> {
    trace!(
        "build_rev_reg_delta_request >>> issuer_did: {}, rev_reg_id: {}, rev_reg_entry_json: {}",
        issuer_did,
        rev_reg_id,
        rev_reg_entry_json
    );

    let request =
        libindy_build_revoc_reg_entry_request(issuer_did, rev_reg_id, REVOC_REG_TYPE, rev_reg_entry_json).await?;

    let request = append_txn_author_agreement_to_request(&request).await?;

    Ok(request)
}

pub async fn get_rev_reg_delta_json(
    pool_handle: PoolHandle,
    rev_reg_id: &str,
    from: Option<u64>,
    to: Option<u64>,
) -> VcxResult<(String, String, u64)> {
    trace!(
        "get_rev_reg_delta_json >>> pool_handle: {:?}, rev_reg_id: {}, from: {:?}, to: {:?}",
        pool_handle,
        rev_reg_id,
        from,
        to
    );
    if settings::indy_mocks_enabled() {
        debug!("get_rev_reg_delta_json >>> returning mocked value");
        return Ok((REV_REG_ID.to_string(), REV_REG_DELTA_JSON.to_string(), 1));
    }

    let submitter_did = generate_random_did();

    let from: i64 = if let Some(_from) = from { _from as i64 } else { -1 };
    let to = if let Some(_to) = to {
        _to as i64
    } else {
        time::get_time().sec
    };

    let req = libindy_build_get_revoc_reg_delta_request(&submitter_did, rev_reg_id, from, to).await?;

    let res = libindy_submit_request(pool_handle, &req).await?;

    libindy_parse_get_revoc_reg_delta_response(&res).await
}

pub async fn get_rev_reg(
    pool_handle: PoolHandle,
    rev_reg_id: &str,
    timestamp: u64,
) -> VcxResult<(String, String, u64)> {
    if settings::indy_mocks_enabled() {
        return Ok((REV_REG_ID.to_string(), REV_REG_JSON.to_string(), 1));
    }

    let submitter_did = generate_random_did();

    let req = libindy_build_get_revoc_reg_request(&submitter_did, rev_reg_id, timestamp).await?;

    let res = libindy_submit_request(pool_handle, &req).await?;

    libindy_parse_get_revoc_reg_response(&res).await
}

async fn libindy_build_get_txn_request(submitter_did: Option<&str>, seq_no: i32) -> VcxResult<String> {
    let res =
        Locator::instance()
            .ledger_controller
            .build_get_txn_request(submitter_did.map(|s| s.into()), None, seq_no)?;

    Ok(res)
}

pub async fn build_get_txn_request(submitter_did: Option<&str>, seq_no: i32) -> VcxResult<String> {
    trace!(
        "build_get_txn_request >>> submitter_did: {:?}, seq_no: {}",
        submitter_did,
        seq_no
    );
    let request = libindy_build_get_txn_request(submitter_did, seq_no).await?;
    let request = append_txn_author_agreement_to_request(&request).await?;
    Ok(request)
}

pub async fn get_ledger_txn(
    wallet_handle: WalletHandle,
    pool_handle: PoolHandle,
    seq_no: i32,
    submitter_did: Option<&str>,
) -> VcxResult<String> {
    trace!(
        "get_ledger_txn >>> submitter_did: {:?}, seq_no: {}",
        submitter_did,
        seq_no
    );
    let req = build_get_txn_request(submitter_did, seq_no).await?;
    let res = if let Some(submitter_did) = submitter_did {
        libindy_sign_and_submit_request(wallet_handle, pool_handle, submitter_did, &req).await?
    } else {
        libindy_submit_request(pool_handle, &req).await?
    };
    check_response(&res)?;
    Ok(res)
}

pub fn _check_schema_response(response: &str) -> VcxResult<()> {
    // TODO: saved backwardcampatibilyty but actually we can better handle response
    match parse_response(response)? {
        Response::Reply(_) => Ok(()),
        Response::Reject(reject) => Err(AriesVcxError::from_msg(
            AriesVcxErrorKind::DuplicationSchema,
            format!("{:?}", reject),
        )),
        Response::ReqNACK(reqnack) => Err(AriesVcxError::from_msg(
            AriesVcxErrorKind::UnknownSchemaRejection,
            format!("{:?}", reqnack),
        )),
    }
}

pub(in crate::indy) fn check_response(response: &str) -> VcxResult<()> {
    if settings::indy_mocks_enabled() {
        return Ok(());
    }
    match parse_response(response)? {
        Response::Reply(_) => Ok(()),
        Response::Reject(res) | Response::ReqNACK(res) => Err(AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidLedgerResponse,
            format!("{:?}", res),
        )),
    }
}

pub async fn get_schema_json(
    wallet_handle: WalletHandle,
    pool_handle: PoolHandle,
    schema_id: &str,
) -> VcxResult<(String, String)> {
    trace!("get_schema_json >>> schema_id: {}", schema_id);
    if settings::indy_mocks_enabled() {
        return Ok((SCHEMA_ID.to_string(), SCHEMA_JSON.to_string()));
    }

    let submitter_did = generate_random_did();

    let schema_json = libindy_get_schema(wallet_handle, pool_handle, &submitter_did, schema_id).await?;

    Ok((schema_id.to_string(), schema_json))
}

pub async fn build_cred_def_request(issuer_did: &str, cred_def_json: &str) -> VcxResult<String> {
    if settings::indy_mocks_enabled() {
        return Ok(CRED_DEF_REQ.to_string());
    }

    let cred_def_req = libindy_build_create_credential_def_txn(issuer_did, cred_def_json).await?;

    let cred_def_req = append_txn_author_agreement_to_request(&cred_def_req).await?;

    Ok(cred_def_req)
}

pub async fn get_cred_def_json(
    wallet_handle: WalletHandle,
    pool_handle: PoolHandle,
    cred_def_id: &str,
) -> VcxResult<(String, String)> {
    if settings::indy_mocks_enabled() {
        debug!("get_cred_def_json >>> returning mocked value");
        return Ok((CRED_DEF_ID.to_string(), CRED_DEF_JSON.to_string()));
    }

    let cred_def_json = libindy_get_cred_def(wallet_handle, pool_handle, cred_def_id).await?;

    Ok((cred_def_id.to_string(), cred_def_json))
}

#[cfg(test)]
#[cfg(feature = "general_test")]
mod test {
    use super::*;
    use crate::utils::devsetup::*;

    #[test]
    fn test_verify_transaction_can_be_endorsed() {
        let _setup = SetupDefaults::init();

        // success
        let transaction = r#"{"reqId":1, "identifier": "EbP4aYNeTHL6q385GuVpRV", "signature": "gkVDhwe2", "endorser": "NcYxiDXkpYi6ov5FcYDi1e"}"#;
        assert!(_verify_transaction_can_be_endorsed(transaction, "NcYxiDXkpYi6ov5FcYDi1e").is_ok());

        // no author signature
        let transaction =
            r#"{"reqId":1, "identifier": "EbP4aYNeTHL6q385GuVpRV", "endorser": "NcYxiDXkpYi6ov5FcYDi1e"}"#;
        assert!(_verify_transaction_can_be_endorsed(transaction, "NcYxiDXkpYi6ov5FcYDi1e").is_err());

        // different endorser did
        let transaction =
            r#"{"reqId":1, "identifier": "EbP4aYNeTHL6q385GuVpRV", "endorser": "NcYxiDXkpYi6ov5FcYDi1e"}"#;
        assert!(_verify_transaction_can_be_endorsed(transaction, "EbP4aYNeTHL6q385GuVpRV").is_err());
    }
}

#[cfg(test)]
#[cfg(feature = "pool_tests")]
pub mod integration_tests {
    use crate::{indy::ledger::transactions::get_ledger_txn, utils::devsetup::SetupWalletPool};

    #[tokio::test]
    async fn test_get_txn() {
        SetupWalletPool::run(|setup| async move {
            get_ledger_txn(setup.wallet_handle, setup.pool_handle, 0, None)
                .await
                .unwrap_err();
            let txn = get_ledger_txn(setup.wallet_handle, setup.pool_handle, 1, None).await;
            assert!(txn.is_ok());

            get_ledger_txn(setup.wallet_handle, setup.pool_handle, 0, Some(&setup.institution_did))
                .await
                .unwrap_err();
            let txn = get_ledger_txn(setup.wallet_handle, setup.pool_handle, 1, Some(&setup.institution_did)).await;
            assert!(txn.is_ok());
        })
        .await;
    }
}
