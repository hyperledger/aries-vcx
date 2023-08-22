use aries_vcx::handlers::util::OfferInfo;
use aries_vcx::messages::AriesMessage;
use aries_vcx::protocols::SendClosure;
use serde_json;

use aries_vcx::handlers::issuance::issuer::Issuer;
use aries_vcx::handlers::issuance::mediated_issuer::issuer_find_messages_to_handle;

use crate::api_vcx::api_global::profile::{get_main_anoncreds, get_main_wallet};
use crate::api_vcx::api_handle::connection;
use crate::api_vcx::api_handle::connection::HttpClient;
use crate::api_vcx::api_handle::credential_def;
use crate::api_vcx::api_handle::mediated_connection;
use crate::api_vcx::api_handle::object_cache::ObjectCache;
use crate::api_vcx::api_handle::revocation_registry::REV_REG_MAP;

use crate::errors::error::{LibvcxError, LibvcxErrorKind, LibvcxResult};

lazy_static! {
    static ref ISSUER_CREDENTIAL_MAP: ObjectCache<Issuer> = ObjectCache::<Issuer>::new("issuer-credentials-cache");
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

pub async fn update_state(handle: u32, message: Option<&str>, connection_handle: u32) -> LibvcxResult<u32> {
    trace!("issuer_credential::update_state >>> ");
    let mut credential = ISSUER_CREDENTIAL_MAP.get_cloned(handle)?;
    if credential.is_terminal_state() {
        return Ok(credential.get_state().into());
    }
    if let Some(message) = message {
        let msg: AriesMessage = serde_json::from_str(message).map_err(|err| {
            LibvcxError::from_msg(
                LibvcxErrorKind::InvalidOption,
                format!("Cannot update state: Message deserialization failed: {:?}", err),
            )
        })?;
        credential.process_aries_msg(msg.into()).await?;
    } else {
        let messages = mediated_connection::get_messages(connection_handle).await?;
        if let Some((uid, msg)) = issuer_find_messages_to_handle(&credential, messages) {
            credential.process_aries_msg(msg.into()).await?;
            mediated_connection::update_message_status(connection_handle, &uid).await?;
        }
    }
    let res: u32 = credential.get_state().into();
    ISSUER_CREDENTIAL_MAP.insert(handle, credential)?;
    Ok(res)
}

pub async fn update_state_with_message_nonmediated(
    handle: u32,
    connection_handle: u32,
    message: &str,
) -> LibvcxResult<u32> {
    trace!("issuer_credential::update_state_nonmediated >>> ");
    let mut credential = ISSUER_CREDENTIAL_MAP.get_cloned(handle)?;
    if credential.is_terminal_state() {
        return Ok(credential.get_state().into());
    }

    let con = connection::get_cloned_generic_connection(&connection_handle)?;
    let wallet = get_main_wallet()?;

    let send_message: SendClosure =
        Box::new(|msg: AriesMessage| Box::pin(async move { con.send_message(&wallet, &msg, &HttpClient).await }));

    let message: AriesMessage = serde_json::from_str(message).map_err(|err| {
        LibvcxError::from_msg(
            LibvcxErrorKind::InvalidOption,
            format!("Cannot update state: Message deserialization failed: {:?}", err),
        )
    })?;
    credential.process_aries_msg(message.into()).await?;

    let res: u32 = credential.get_state().into();
    ISSUER_CREDENTIAL_MAP.insert(handle, credential)?;
    Ok(res)
}

pub fn get_state(handle: u32) -> LibvcxResult<u32> {
    ISSUER_CREDENTIAL_MAP.get(handle, |credential| Ok(credential.get_state().into()))
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
    ISSUER_CREDENTIAL_MAP
        .release(handle)
        .map_err(|e| LibvcxError::from_msg(LibvcxErrorKind::InvalidIssuerCredentialHandle, e.to_string()))
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
    let issuer_credential: IssuerCredentials = serde_json::from_str(credential_data).map_err(|err| {
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
            "Cannot issue credential of specified credential definition has not been published on the ledger",
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
            &get_main_anoncreds()?,
            offer_info.clone(),
            comment.map(|s| s.to_string()),
        )
        .await?;
    ISSUER_CREDENTIAL_MAP.insert(credential_handle, credential)
}

pub fn mark_credential_offer_msg_sent(handle: u32) -> LibvcxResult<()> {
    let mut credential = ISSUER_CREDENTIAL_MAP.get_cloned(handle)?;
    credential.mark_credential_offer_msg_sent()?;
    ISSUER_CREDENTIAL_MAP.insert(handle, credential)
}

pub fn get_credential_offer_msg(handle: u32) -> LibvcxResult<AriesMessage> {
    ISSUER_CREDENTIAL_MAP.get(handle, |credential| Ok(credential.get_credential_offer_msg()?))
}

pub async fn send_credential_offer_v2(credential_handle: u32, connection_handle: u32) -> LibvcxResult<()> {
    let mut credential = ISSUER_CREDENTIAL_MAP.get_cloned(credential_handle)?;
    let send_message = mediated_connection::send_message_closure(connection_handle).await?;
    credential.send_credential_offer(send_message).await?;
    ISSUER_CREDENTIAL_MAP.insert(credential_handle, credential)?;
    Ok(())
}

pub async fn send_credential_offer_nonmediated(credential_handle: u32, connection_handle: u32) -> LibvcxResult<()> {
    let mut credential = ISSUER_CREDENTIAL_MAP.get_cloned(credential_handle)?;

    let con = connection::get_cloned_generic_connection(&connection_handle)?;
    let wallet = get_main_wallet()?;

    let send_message: SendClosure =
        Box::new(|msg: AriesMessage| Box::pin(async move { con.send_message(&wallet, &msg, &HttpClient).await }));

    credential.send_credential_offer(send_message).await?;
    ISSUER_CREDENTIAL_MAP.insert(credential_handle, credential)?;
    Ok(())
}

pub async fn send_credential(handle: u32, connection_handle: u32) -> LibvcxResult<u32> {
    let mut credential = ISSUER_CREDENTIAL_MAP.get_cloned(handle)?;
    credential
        .send_credential(
            &get_main_anoncreds()?,
            mediated_connection::send_message_closure(connection_handle).await?,
        )
        .await?;
    let state: u32 = credential.get_state().into();
    ISSUER_CREDENTIAL_MAP.insert(handle, credential)?;
    Ok(state)
}

pub async fn send_credential_nonmediated(handle: u32, connection_handle: u32) -> LibvcxResult<u32> {
    let mut credential = ISSUER_CREDENTIAL_MAP.get_cloned(handle)?;
    let con = connection::get_cloned_generic_connection(&connection_handle)?;
    let wallet = get_main_wallet()?;

    let send_message: SendClosure =
        Box::new(|msg: AriesMessage| Box::pin(async move { con.send_message(&wallet, &msg, &HttpClient).await }));

    credential.send_credential(&get_main_anoncreds()?, send_message).await?;
    let state: u32 = credential.get_state().into();
    ISSUER_CREDENTIAL_MAP.insert(handle, credential)?;
    Ok(state)
}

pub async fn revoke_credential_local(handle: u32) -> LibvcxResult<()> {
    let credential = ISSUER_CREDENTIAL_MAP.get_cloned(handle)?;
    credential
        .revoke_credential_local(&get_main_anoncreds()?)
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
#[allow(clippy::unwrap_used)]
pub mod tests {
    #[cfg(test)]
    use crate::api_vcx::api_handle::credential_def::tests::create_and_publish_nonrevocable_creddef;
    #[cfg(test)]
    use crate::api_vcx::api_handle::mediated_connection::test_utils::build_test_connection_inviter_requested;
    use crate::aries_vcx::protocols::issuance::issuer::state_machine::IssuerState;
    use crate::errors::error;
    use aries_vcx::utils::constants::V3_OBJECT_SERIALIZE_VERSION;
    use aries_vcx::utils::devsetup::SetupMocks;
    use aries_vcx::utils::mockdata::mockdata_credex::ARIES_CREDENTIAL_REQUEST;
    use aries_vcx::utils::mockdata::mockdata_mediated_connection::ARIES_CONNECTION_ACK;

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
        assert_eq!(to_string(handle).unwrap_err().kind, LibvcxErrorKind::InvalidHandle)
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
    async fn test_send_credential_offer() {
        let _setup = SetupMocks::init();

        let connection_handle = build_test_connection_inviter_requested().await;

        let credential_handle = _issuer_credential_create();

        let (_, cred_def_handle) = create_and_publish_nonrevocable_creddef().await;
        build_credential_offer_msg_v2(credential_handle, cred_def_handle, 123, _cred_json(), None)
            .await
            .unwrap();
        send_credential_offer_v2(credential_handle, connection_handle)
            .await
            .unwrap();
        assert_eq!(get_state(credential_handle).unwrap(), u32::from(IssuerState::OfferSent));
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
    async fn test_update_state_with_message() {
        let _setup = SetupMocks::init();

        let connection_handle = build_test_connection_inviter_requested().await;
        let credential_handle = _issuer_credential_create();
        let (_, cred_def_handle) = create_and_publish_nonrevocable_creddef().await;
        build_credential_offer_msg_v2(credential_handle, cred_def_handle, 1234, _cred_json(), None)
            .await
            .unwrap();
        send_credential_offer_v2(credential_handle, connection_handle)
            .await
            .unwrap();
        assert_eq!(get_state(credential_handle).unwrap(), u32::from(IssuerState::OfferSent));

        update_state(credential_handle, Some(ARIES_CREDENTIAL_REQUEST), connection_handle)
            .await
            .unwrap();
        assert_eq!(
            get_state(credential_handle).unwrap(),
            u32::from(IssuerState::RequestReceived)
        );
    }

    #[tokio::test]
    async fn test_update_state_with_bad_message() {
        let _setup = SetupMocks::init();

        let handle_conn = build_test_connection_inviter_requested().await;
        let handle_cred = _issuer_credential_create();
        let (_, cred_def_handle) = create_and_publish_nonrevocable_creddef().await;
        build_credential_offer_msg_v2(handle_cred, cred_def_handle, 1234, _cred_json(), None)
            .await
            .unwrap();
        send_credential_offer_v2(handle_cred, handle_conn).await.unwrap();
        assert_eq!(get_state(handle_cred).unwrap(), u32::from(IssuerState::OfferSent));

        // try to update state with nonsense message
        let result = update_state(handle_cred, Some(ARIES_CONNECTION_ACK), handle_conn).await;
        assert!(result.is_ok()); // todo: maybe we should rather return error if update_state doesn't progress state
        assert_eq!(get_state(handle_cred).unwrap(), u32::from(IssuerState::OfferSent));
    }

    #[tokio::test]
    async fn test_release_all() {
        let _setup = SetupMocks::init();

        let h1 = _issuer_credential_create();
        let h2 = _issuer_credential_create();
        let h3 = _issuer_credential_create();
        release_all();
        assert_eq!(is_valid_handle(h1), false);
        assert_eq!(is_valid_handle(h2), false);
        assert_eq!(is_valid_handle(h3), false);
    }
}
