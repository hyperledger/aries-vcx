use std::collections::HashMap;

use aries_vcx::{
    handlers::{
        issuance::issuer::Issuer,
        util::{matches_opt_thread_id, matches_thread_id, OfferInfo},
    },
    messages::{
        msg_fields::protocols::{
            cred_issuance::{v1::CredentialIssuanceV1, CredentialIssuance},
            notification::Notification,
        },
        AriesMessage,
    },
    protocols::{issuance::issuer::state_machine::IssuerState, SendClosure},
};
use serde_json;

use super::mediated_connection::send_message;
use crate::{
    api_vcx::{
        api_global::profile::{get_main_anoncreds, get_main_wallet},
        api_handle::{
            connection, connection::HttpClient, credential_def, mediated_connection,
            object_cache::ObjectCache, revocation_registry::REV_REG_MAP, ToU32,
        },
    },
    errors::error::{LibvcxError, LibvcxErrorKind, LibvcxResult},
};

lazy_static! {
    static ref ISSUER_CREDENTIAL_MAP: ObjectCache<Issuer> =
        ObjectCache::<Issuer>::new("issuer-credentials-cache");
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "version", content = "data")]
enum IssuerCredentials {
    #[serde(rename = "2.0")]
    V3(Issuer),
}

pub fn issuer_credential_create(source_id: String) -> LibvcxResult<u32> {
    ISSUER_CREDENTIAL_MAP.add(Issuer::create(&source_id)?)
}

pub fn issuer_find_message_to_handle(
    sm: &Issuer,
    messages: HashMap<String, AriesMessage>,
) -> Option<(String, AriesMessage)> {
    trace!(
        "issuer_find_messages_to_handle >>> messages: {:?}, state: {:?}",
        messages,
        sm
    );

    for (uid, message) in messages {
        match sm.get_state() {
            IssuerState::Initial => {
                if let AriesMessage::CredentialIssuance(CredentialIssuance::V1(
                    CredentialIssuanceV1::ProposeCredential(_),
                )) = &message
                {
                    info!(
                        "In state IssuerState::OfferSet, found matching message ProposeCredential"
                    );
                    return Some((uid, message));
                }
            }
            IssuerState::OfferSet => match &message {
                AriesMessage::CredentialIssuance(CredentialIssuance::V1(
                    CredentialIssuanceV1::RequestCredential(msg),
                )) => {
                    info!(
                        "In state IssuerState::OfferSet, found potentially matching message \
                         RequestCredential"
                    );
                    warn!("Matching for {}", sm.get_thread_id().unwrap().as_str()); // todo: the state machine has "test" thid, and doesnt match msg
                    warn!("Msg: {msg:?}");
                    if matches_opt_thread_id!(msg, sm.get_thread_id().unwrap().as_str()) {
                        return Some((uid, message));
                    }
                }
                AriesMessage::CredentialIssuance(CredentialIssuance::V1(
                    CredentialIssuanceV1::ProposeCredential(msg),
                )) => {
                    info!(
                        "In state IssuerState::OfferSet, found potentially matching message \
                         ProposeCredential"
                    );
                    if matches_opt_thread_id!(msg, sm.get_thread_id().unwrap().as_str()) {
                        return Some((uid, message));
                    }
                }
                AriesMessage::ReportProblem(msg) => {
                    info!("In state IssuerState::OfferSet, found matching message ReportProblem");
                    if matches_opt_thread_id!(msg, sm.get_thread_id().unwrap().as_str()) {
                        return Some((uid, message));
                    }
                }
                _ => {}
            },
            IssuerState::CredentialSet => match &message {
                AriesMessage::CredentialIssuance(CredentialIssuance::V1(
                    CredentialIssuanceV1::Ack(msg),
                )) => {
                    info!(
                        "In state IssuerState::CredentialSet, found matching message \
                         CredentialIssuance::Ack"
                    );
                    if matches_thread_id!(msg, sm.get_thread_id().unwrap().as_str()) {
                        return Some((uid, message));
                    }
                }
                AriesMessage::Notification(Notification::Ack(msg)) => {
                    info!(
                        "In state IssuerState::CredentialSet, found matching message \
                         Notification::Ack"
                    );
                    if matches_thread_id!(msg, sm.get_thread_id().unwrap().as_str()) {
                        return Some((uid, message));
                    }
                }
                AriesMessage::ReportProblem(msg) => {
                    info!(
                        "In state IssuerState::CredentialSet, found matching message ReportProblem"
                    );
                    if matches_opt_thread_id!(msg, sm.get_thread_id().unwrap().as_str()) {
                        return Some((uid, message));
                    }
                }
                _ => {}
            },
            _ => {}
        };
    }
    None
}

pub async fn update_state(
    handle: u32,
    message: Option<&str>,
    connection_handle: u32,
) -> LibvcxResult<u32> {
    trace!("issuer_credential::update_state >>> ");
    let mut credential = ISSUER_CREDENTIAL_MAP.get_cloned(handle)?;
    if credential.is_terminal_state() {
        return Ok(credential.get_state().to_u32());
    }
    if let Some(message) = message {
        let msg: AriesMessage = serde_json::from_str(message).map_err(|err| {
            LibvcxError::from_msg(
                LibvcxErrorKind::InvalidOption,
                format!(
                    "Cannot update state: Message deserialization failed: {:?}",
                    err
                ),
            )
        })?;
        credential.process_aries_msg(msg).await?;
    } else {
        let messages = mediated_connection::get_messages(connection_handle).await?;
        if let Some((uid, msg)) = issuer_find_message_to_handle(&credential, messages) {
            credential.process_aries_msg(msg).await?;
            mediated_connection::update_message_status(connection_handle, &uid).await?;
        }
    }
    let res: u32 = credential.get_state().to_u32();
    ISSUER_CREDENTIAL_MAP.insert(handle, credential)?;
    Ok(res)
}

pub async fn update_state_with_message_nonmediated(
    handle: u32,
    _connection_handle: u32,
    message: &str,
) -> LibvcxResult<u32> {
    trace!("issuer_credential::update_state_nonmediated >>> ");
    let mut credential = ISSUER_CREDENTIAL_MAP.get_cloned(handle)?;
    if credential.is_terminal_state() {
        return Ok(credential.get_state().to_u32());
    }

    let message: AriesMessage = serde_json::from_str(message).map_err(|err| {
        LibvcxError::from_msg(
            LibvcxErrorKind::InvalidOption,
            format!(
                "Cannot update state: Message deserialization failed: {:?}",
                err
            ),
        )
    })?;
    credential.process_aries_msg(message).await?;

    let res: u32 = credential.get_state().to_u32();
    ISSUER_CREDENTIAL_MAP.insert(handle, credential)?;
    Ok(res)
}

pub fn get_state(handle: u32) -> LibvcxResult<u32> {
    ISSUER_CREDENTIAL_MAP.get(handle, |credential| Ok(credential.get_state().to_u32()))
}

pub fn get_credential_status(handle: u32) -> LibvcxResult<u32> {
    ISSUER_CREDENTIAL_MAP.get(handle, |credential| {
        credential.get_credential_status().map_err(|err| err.into())
    })
}

pub fn get_revocation_id(handle: u32) -> LibvcxResult<String> {
    ISSUER_CREDENTIAL_MAP.get(handle, |credential| {
        credential.get_revocation_id().map_err(|err| err.into())
    })
}

pub fn release(handle: u32) -> LibvcxResult<()> {
    ISSUER_CREDENTIAL_MAP.release(handle).map_err(|e| {
        LibvcxError::from_msg(
            LibvcxErrorKind::InvalidIssuerCredentialHandle,
            e.to_string(),
        )
    })
}

pub fn release_all() {
    ISSUER_CREDENTIAL_MAP.drain().ok();
}

pub fn is_valid_handle(handle: u32) -> bool {
    ISSUER_CREDENTIAL_MAP.has_handle(handle)
}

pub fn to_string(handle: u32) -> LibvcxResult<String> {
    ISSUER_CREDENTIAL_MAP.get(handle, |credential| {
        serde_json::to_string(&IssuerCredentials::V3(credential.clone())).map_err(|err| {
            LibvcxError::from_msg(
                LibvcxErrorKind::InvalidState,
                format!("cannot serialize IssuerCredential credentialect: {:?}", err),
            )
        })
    })
}

pub fn from_string(credential_data: &str) -> LibvcxResult<u32> {
    let issuer_credential: IssuerCredentials =
        serde_json::from_str(credential_data).map_err(|err| {
            LibvcxError::from_msg(
                LibvcxErrorKind::InvalidJson,
                format!("Cannot deserialize IssuerCredential: {:?}", err),
            )
        })?;

    match issuer_credential {
        IssuerCredentials::V3(credential) => ISSUER_CREDENTIAL_MAP.add(credential),
    }
}

pub async fn build_credential_offer_msg_v2(
    credential_handle: u32,
    cred_def_handle: u32,
    rev_reg_handle: u32,
    credential_json: &str,
    comment: Option<&str>,
) -> LibvcxResult<()> {
    if !credential_def::check_is_published(cred_def_handle)? {
        return Err(LibvcxError::from_msg(
            LibvcxErrorKind::InvalidJson,
            "Cannot issue credential of specified credential definition has not been published on \
             the ledger",
        ));
    };
    // todo: add check if rev reg was published
    let mut credential = ISSUER_CREDENTIAL_MAP.get_cloned(credential_handle)?;
    let cred_def = credential_def::CREDENTIALDEF_MAP.get_cloned(cred_def_handle)?;
    let offer_info = if cred_def.get_support_revocation() {
        let rev_reg = REV_REG_MAP.get_cloned(rev_reg_handle)?;
        OfferInfo {
            credential_json: credential_json.to_string(),
            cred_def_id: credential_def::get_cred_def_id(cred_def_handle)?,
            rev_reg_id: Some(rev_reg.get_rev_reg_id()),
            tails_file: Some(rev_reg.get_tails_dir()),
        }
    } else {
        OfferInfo {
            credential_json: credential_json.to_string(),
            cred_def_id: credential_def::get_cred_def_id(cred_def_handle)?,
            rev_reg_id: None,
            tails_file: None,
        }
    };
    credential
        .build_credential_offer_msg(
            get_main_wallet()?.as_ref(),
            get_main_anoncreds()?.as_ref(),
            offer_info.clone(),
            comment.map(|s| s.to_string()),
        )
        .await?;
    ISSUER_CREDENTIAL_MAP.insert(credential_handle, credential)
}

pub fn get_credential_offer_msg(handle: u32) -> LibvcxResult<AriesMessage> {
    ISSUER_CREDENTIAL_MAP.get(handle, |credential| {
        Ok(credential.get_credential_offer_msg()?)
    })
}

pub async fn send_credential_offer_v2(
    credential_handle: u32,
    connection_handle: u32,
) -> LibvcxResult<()> {
    let credential = ISSUER_CREDENTIAL_MAP.get_cloned(credential_handle)?;
    let credential_offer = credential.get_credential_offer_msg()?;
    send_message(connection_handle, credential_offer).await?;
    ISSUER_CREDENTIAL_MAP.insert(credential_handle, credential)?;
    Ok(())
}

pub async fn send_credential_offer_nonmediated(
    credential_handle: u32,
    connection_handle: u32,
) -> LibvcxResult<()> {
    let credential = ISSUER_CREDENTIAL_MAP.get_cloned(credential_handle)?;

    let con = connection::get_cloned_generic_connection(&connection_handle)?;
    let wallet = get_main_wallet()?;

    let send_message: SendClosure = Box::new(|msg: AriesMessage| {
        Box::pin(async move { con.send_message(wallet.as_ref(), &msg, &HttpClient).await })
    });
    let credential_offer = credential.get_credential_offer_msg()?;
    send_message(credential_offer).await?;

    ISSUER_CREDENTIAL_MAP.insert(credential_handle, credential)?;
    Ok(())
}

pub async fn send_credential(handle: u32, connection_handle: u32) -> LibvcxResult<u32> {
    let mut credential = ISSUER_CREDENTIAL_MAP.get_cloned(handle)?;
    credential
        .build_credential(get_main_wallet()?.as_ref(), get_main_anoncreds()?.as_ref())
        .await?;
    match credential.get_state() {
        IssuerState::Failed => {
            let problem_report = credential.get_problem_report()?;
            send_message(connection_handle, problem_report.into()).await?;
        }
        _ => {
            let msg_issue_credential = credential.get_msg_issue_credential()?;
            send_message(connection_handle, msg_issue_credential.into()).await?;
        }
    }
    let state: u32 = credential.get_state().to_u32();
    ISSUER_CREDENTIAL_MAP.insert(handle, credential)?;
    Ok(state)
}

pub async fn send_credential_nonmediated(handle: u32, connection_handle: u32) -> LibvcxResult<u32> {
    let mut credential = ISSUER_CREDENTIAL_MAP.get_cloned(handle)?;
    let con = connection::get_cloned_generic_connection(&connection_handle)?;
    let wallet = get_main_wallet()?;
    let send_closure: SendClosure = Box::new(|msg: AriesMessage| {
        Box::pin(async move { con.send_message(wallet.as_ref(), &msg, &HttpClient).await })
    });
    credential
        .build_credential(get_main_wallet()?.as_ref(), get_main_anoncreds()?.as_ref())
        .await?;
    match credential.get_state() {
        IssuerState::Failed => {
            let problem_report = credential.get_problem_report()?;
            send_closure(problem_report.into()).await?;
        }
        _ => {
            let msg_issue_credential = credential.get_msg_issue_credential()?;
            send_closure(msg_issue_credential.into()).await?;
        }
    }
    let state: u32 = credential.get_state().to_u32();
    ISSUER_CREDENTIAL_MAP.insert(handle, credential)?;
    Ok(state)
}

pub async fn revoke_credential_local(handle: u32) -> LibvcxResult<()> {
    let credential = ISSUER_CREDENTIAL_MAP.get_cloned(handle)?;
    credential
        .revoke_credential_local(get_main_wallet()?.as_ref(), get_main_anoncreds()?.as_ref())
        .await
        .map_err(|err| err.into())
}

pub fn get_rev_reg_id(handle: u32) -> LibvcxResult<String> {
    ISSUER_CREDENTIAL_MAP.get(handle, |credential| {
        credential.get_rev_reg_id().map_err(|err| err.into())
    })
}

pub fn is_revokable(handle: u32) -> LibvcxResult<bool> {
    ISSUER_CREDENTIAL_MAP.get(handle, |credential| Ok(credential.is_revokable()))
}

pub fn get_source_id(handle: u32) -> LibvcxResult<String> {
    ISSUER_CREDENTIAL_MAP.get(handle, |credential| {
        credential.get_source_id().map_err(|err| err.into())
    })
}

pub fn get_thread_id(handle: u32) -> LibvcxResult<String> {
    ISSUER_CREDENTIAL_MAP.get(handle, |credential| {
        credential.get_thread_id().map_err(|err| err.into())
    })
}

#[cfg(test)]
mod tests {
    use aries_vcx_core::test_utils::{
        constants::V3_OBJECT_SERIALIZE_VERSION, devsetup::SetupMocks,
    };

    use super::*;

    fn _issuer_credential_create() -> u32 {
        issuer_credential_create("1".to_string()).unwrap()
    }

    fn _cred_json() -> &'static str {
        "{\"attr\":\"value\"}"
    }

    #[test]
    fn test_vcx_issuer_credential_release() {
        let _setup = SetupMocks::init();
        let handle = _issuer_credential_create();
        release(handle).unwrap();
        assert_eq!(
            to_string(handle).unwrap_err().kind,
            LibvcxErrorKind::InvalidHandle
        )
    }

    #[tokio::test]
    async fn test_issuer_credential_create_succeeds() {
        let _setup = SetupMocks::init();

        let handle = _issuer_credential_create();
        assert!(handle > 0);
    }

    #[tokio::test]
    async fn test_to_string_succeeds() {
        let _setup = SetupMocks::init();

        let handle = _issuer_credential_create();
        let string = to_string(handle).unwrap();
        assert!(!string.is_empty());
    }

    #[tokio::test]
    async fn test_from_string_succeeds() {
        let _setup = SetupMocks::init();

        let handle = _issuer_credential_create();

        let string = to_string(handle).unwrap();

        let value: serde_json::Value = serde_json::from_str(&string).unwrap();
        assert_eq!(value["version"], V3_OBJECT_SERIALIZE_VERSION);

        release(handle).unwrap();

        let new_handle = from_string(&string).unwrap();

        let new_string = to_string(new_handle).unwrap();
        assert_eq!(new_string, string);
    }

    #[tokio::test]
    async fn test_release_all() {
        let _setup = SetupMocks::init();

        let h1 = _issuer_credential_create();
        let h2 = _issuer_credential_create();
        let h3 = _issuer_credential_create();
        release_all();
        assert!(!is_valid_handle(h1));
        assert!(!is_valid_handle(h2));
        assert!(!is_valid_handle(h3));
    }
}
