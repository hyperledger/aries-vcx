use std::collections::HashMap;

use aries_vcx::protocols::connection::pairwise_info::PairwiseInfo;
use serde_json;

use crate::api_lib::global::profile::{get_main_profile, get_main_profile_optional_pool};
use aries_vcx::agency_client::api::downloaded_message::DownloadedMessage;
use aries_vcx::agency_client::MessageStatusCode;
use aries_vcx::error::{VcxError, VcxErrorKind, VcxResult};
use aries_vcx::handlers::connection::mediated_connection::MediatedConnection;
use aries_vcx::xyz::ledger::transactions::into_did_doc;
use aries_vcx::messages::a2a::A2AMessage;
use aries_vcx::messages::connection::invite::Invitation as InvitationV3;
use aries_vcx::messages::connection::invite::PublicInvitation;
use aries_vcx::messages::connection::request::Request;
use aries_vcx::protocols::SendClosure;
use aries_vcx::utils::error;

use crate::api_lib::api_handle::agent::PUBLIC_AGENT_MAP;
use crate::api_lib::api_handle::object_cache::ObjectCache;
use crate::api_lib::global::agency_client::get_main_agency_client;

lazy_static! {
    pub static ref CONNECTION_MAP: ObjectCache<MediatedConnection> = ObjectCache::<MediatedConnection>::new("connections-cache");
}

pub fn generate_public_invitation(public_did: &str, label: &str) -> VcxResult<String> {
    trace!(
        "generate_public_invite >>> label: {}, public_did: {}",
        public_did,
        label
    );
    let invitation =
        A2AMessage::ConnectionInvitationPublic(PublicInvitation::create().set_public_did(public_did)?.set_label(label));
    Ok(json!(invitation).to_string())
}

pub fn is_valid_handle(handle: u32) -> bool {
    CONNECTION_MAP.has_handle(handle)
}

pub fn get_agent_did(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |connection| {
        Ok(connection
            .cloud_agent_info()
            .ok_or(VcxError::from_msg(
                VcxErrorKind::NoAgentInformation,
                "Missing cloud agent info",
            ))?
            .agent_did
            .to_string())
    })
}

pub fn get_agent_verkey(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |connection| {
        Ok(connection
            .cloud_agent_info()
            .ok_or(VcxError::from_msg(
                VcxErrorKind::NoAgentInformation,
                "Missing cloud agent info",
            ))?
            .agent_vk
            .clone())
    })
}

pub fn get_pw_did(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |connection| Ok(connection.pairwise_info().pw_did.to_string()))
}

pub fn get_pw_verkey(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |connection| Ok(connection.pairwise_info().pw_vk.clone()))
}

pub fn get_their_pw_did(handle: u32) -> VcxResult<String> {
    let connection = CONNECTION_MAP.get_cloned(handle)?;
    connection.remote_did().map_err(|err| err.into())
}

pub fn get_their_pw_verkey(handle: u32) -> VcxResult<String> {
    let connection = CONNECTION_MAP.get_cloned(handle)?;
    connection.remote_vk().map_err(|err| err.into())
}

pub fn get_thread_id(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |connection| Ok(connection.get_thread_id()))
}

pub fn get_state(handle: u32) -> u32 {
    trace!("get_state >>> handle = {:?}", handle);
    CONNECTION_MAP
        .get(handle, |connection| Ok(connection.get_state().into()))
        .unwrap_or(0)
}

pub fn get_source_id(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |connection| Ok(connection.get_source_id()))
}

pub fn store_connection(connection: MediatedConnection) -> VcxResult<u32> {
    CONNECTION_MAP
        .add(connection)
        .or(Err(VcxError::from(VcxErrorKind::CreateConnection)))
}

pub async fn create_connection(source_id: &str) -> VcxResult<u32> {
    trace!("create_connection >>> source_id: {}", source_id);
    let connection = MediatedConnection::create(
        source_id,
        &get_main_profile_optional_pool(), // do not throw if pool is not open
        &get_main_agency_client().unwrap(),
        true,
    )
    .await?;
    store_connection(connection)
}

pub async fn create_connection_with_invite(source_id: &str, details: &str) -> VcxResult<u32> {
    debug!("create connection {} with invite {}", source_id, details);
    if let Some(invitation) = serde_json::from_str::<InvitationV3>(details).ok() {
        let profile = get_main_profile()?;
        let ddo = into_did_doc(&profile, &invitation).await?;
        let connection = MediatedConnection::create_with_invite(
            source_id,
            &profile,
            &get_main_agency_client().unwrap(),
            invitation,
            ddo,
            true,
        )
        .await?;
        store_connection(connection)
    } else {
        Err(VcxError::from_msg(
            VcxErrorKind::InvalidJson,
            "Used invite has invalid structure",
        ))
        // TODO: Specific error type
    }
}

pub async fn create_with_request(request: &str, agent_handle: u32) -> VcxResult<u32> {
    let agent = PUBLIC_AGENT_MAP.get_cloned(agent_handle)?;
    let request: Request = serde_json::from_str(request).map_err(|err| {
        VcxError::from_msg(
            VcxErrorKind::InvalidJson,
            format!("Cannot deserialize connection request: {:?}", err),
        )
    })?;
    let profile = get_main_profile_optional_pool(); // do not throw if pool is not open
    let connection = MediatedConnection::create_with_request(
        &profile,
        request,
        agent.pairwise_info(),
        &get_main_agency_client().unwrap(),
    )
    .await?;
    store_connection(connection)
}

pub async fn create_with_request_v2(request: &str, pw_info: PairwiseInfo) -> VcxResult<u32> {
    let request: Request = serde_json::from_str(request).map_err(|err| {
        VcxError::from_msg(
            VcxErrorKind::InvalidJson,
            format!("Cannot deserialize connection request: {:?}", err),
        )
    })?;
    let profile = get_main_profile_optional_pool(); // do not throw if pool is not open
    let connection = MediatedConnection::create_with_request(
        &profile,
        request,
        pw_info,
        &get_main_agency_client().unwrap(),
    )
    .await?;
    store_connection(connection)
}

pub async fn send_generic_message(handle: u32, msg: &str) -> VcxResult<String> {
    let connection = CONNECTION_MAP.get_cloned(handle)?;
    let profile = get_main_profile_optional_pool(); // do not throw if pool is not open
    connection
        .send_generic_message(&profile, msg)
        .await
        .map_err(|err| err.into())
}

pub async fn send_handshake_reuse(handle: u32, oob_msg: &str) -> VcxResult<()> {
    let connection = CONNECTION_MAP.get_cloned(handle)?;
    let profile = get_main_profile_optional_pool(); // do not throw if pool is not open
    connection
        .send_handshake_reuse(&profile, oob_msg)
        .await
        .map_err(|err| err.into())
}

pub async fn update_state_with_message(handle: u32, message: &str) -> VcxResult<u32> {
    let mut connection = CONNECTION_MAP.get_cloned(handle)?;
    let message: A2AMessage = serde_json::from_str(message).map_err(|err| {
        VcxError::from_msg(
            VcxErrorKind::InvalidJson,
            format!(
                "Failed to deserialize message {} into A2AMessage, err: {:?}",
                message, err
            ),
        )
    })?;
    let profile = get_main_profile_optional_pool(); // do not throw if pool is not open
    connection
        .update_state_with_message(
            &profile,
            get_main_agency_client().unwrap(),
            Some(message),
        )
        .await?;
    CONNECTION_MAP.insert(handle, connection)?;
    Ok(error::SUCCESS.code_num)
}

pub async fn handle_message(handle: u32, message: &str) -> VcxResult<u32> {
    let mut connection = CONNECTION_MAP.get_cloned(handle)?;
    let message: A2AMessage = serde_json::from_str(message).map_err(|err| {
        VcxError::from_msg(
            VcxErrorKind::InvalidJson,
            format!(
                "Failed to deserialize message {} into A2AMessage, err: {:?}",
                message, err
            ),
        )
    })?;
    let profile = get_main_profile_optional_pool(); // do not throw if pool is not open
    connection.handle_message(message, &profile).await?;
    CONNECTION_MAP.insert(handle, connection)?;
    Ok(error::SUCCESS.code_num)
}

pub async fn update_state(handle: u32) -> VcxResult<u32> {
    let mut connection = CONNECTION_MAP.get_cloned(handle)?;
    let res = if connection.is_in_final_state() {
        info!(
            "connection::update_state >> connection {} is in final state, trying to respond to messages",
            handle
        );
        let profile = get_main_profile_optional_pool(); // do not throw if pool is not open
        match connection
            .find_and_handle_message(&profile, &get_main_agency_client().unwrap())
            .await
        {
            Ok(_) => Ok(error::SUCCESS.code_num),
            Err(err) => Err(err.into()),
        }
    } else {
        info!(
            "connection::update_state >> connection {} is not in final state, trying to update state",
            handle
        );
        let profile = get_main_profile_optional_pool(); // do not throw if pool is not open
        match connection
            .find_message_and_update_state(&profile, &get_main_agency_client().unwrap())
            .await
        {
            Ok(_) => Ok(error::SUCCESS.code_num),
            Err(err) => Err(err.into()),
        }
    };
    CONNECTION_MAP.insert(handle, connection)?;
    res
}

pub async fn delete_connection(handle: u32) -> VcxResult<u32> {
    let connection = CONNECTION_MAP.get_cloned(handle)?;
    connection.delete(&get_main_agency_client().unwrap()).await?;
    release(handle)?;
    Ok(error::SUCCESS.code_num)
}

pub async fn connect(handle: u32) -> VcxResult<Option<String>> {
    let mut connection = CONNECTION_MAP.get_cloned(handle)?;
    let profile = get_main_profile_optional_pool(); // do not throw if pool is not open
    connection
        .connect(&profile, &get_main_agency_client().unwrap())
        .await?;
    let invitation = connection.get_invite_details().map(|invitation| match invitation {
        InvitationV3::Pairwise(invitation) => json!(invitation.to_a2a_message()).to_string(),
        InvitationV3::Public(invitation) => json!(invitation.to_a2a_message()).to_string(),
        InvitationV3::OutOfBand(invitation) => json!(invitation.to_a2a_message()).to_string(),
    });
    CONNECTION_MAP.insert(handle, connection)?;
    Ok(invitation)
}

pub fn to_string(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |connection| connection.to_string().map_err(|err| err.into()))
}

pub fn from_string(connection_data: &str) -> VcxResult<u32> {
    let connection = MediatedConnection::from_string(connection_data)?;
    CONNECTION_MAP.add(connection)
}

pub fn release(handle: u32) -> VcxResult<()> {
    CONNECTION_MAP
        .release(handle)
        .or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn release_all() {
    CONNECTION_MAP.drain().ok();
}

pub fn get_invite_details(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP
        .get(handle, |connection| {
            connection
                .get_invite_details()
                .map(|invitation| match invitation {
                    InvitationV3::Pairwise(invitation) => json!(invitation.to_a2a_message()).to_string(),
                    InvitationV3::Public(invitation) => json!(invitation.to_a2a_message()).to_string(),
                    InvitationV3::OutOfBand(invitation) => json!(invitation.to_a2a_message()).to_string(),
                })
                .ok_or(VcxError::from(VcxErrorKind::ActionNotSupported))
        })
        .or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub async fn get_messages(handle: u32) -> VcxResult<HashMap<String, A2AMessage>> {
    let connection = CONNECTION_MAP.get_cloned(handle)?;
    connection
        .get_messages(&get_main_agency_client().unwrap())
        .await
        .map_err(|err| err.into())
}

pub async fn update_message_status(handle: u32, uid: &str) -> VcxResult<()> {
    let connection = CONNECTION_MAP.get_cloned(handle)?;
    connection
        .update_message_status(uid, &get_main_agency_client().unwrap())
        .await
        .map_err(|err| err.into())
}

pub async fn get_message_by_id(handle: u32, msg_id: &str) -> VcxResult<A2AMessage> {
    let connection = CONNECTION_MAP.get_cloned(handle)?;
    connection
        .get_message_by_id(msg_id, &get_main_agency_client().unwrap())
        .await
        .map_err(|err| err.into())
}

pub async fn send_message(handle: u32, message: A2AMessage) -> VcxResult<()> {
    trace!("connection::send_message >>>");
    let send_message = send_message_closure(handle).await?;
    send_message(message).await.map_err(|err| err.into())
}

pub async fn send_message_closure(handle: u32) -> VcxResult<SendClosure> {
    let connection = CONNECTION_MAP.get_cloned(handle)?;
    let profile = get_main_profile_optional_pool(); // do not throw if pool is not open
    connection
        .send_message_closure(&profile)
        .await
        .map_err(|err| err.into())
}

pub async fn send_ping(handle: u32, comment: Option<&str>) -> VcxResult<()> {
    let mut connection = CONNECTION_MAP.get_cloned(handle)?;
    let profile = get_main_profile_optional_pool(); // do not throw if pool is not open
    connection
        .send_ping(&profile, comment.map(String::from))
        .await?;
    CONNECTION_MAP.insert(handle, connection)
}

pub async fn send_discovery_features(handle: u32, query: Option<&str>, comment: Option<&str>) -> VcxResult<()> {
    let connection = CONNECTION_MAP.get_cloned(handle)?;
    let profile = get_main_profile_optional_pool(); // do not throw if pool is not open
    connection
        .send_discovery_query(
            &profile,
            query.map(String::from),
            comment.map(String::from),
        )
        .await?;
    CONNECTION_MAP.insert(handle, connection)
}

pub async fn get_connection_info(handle: u32) -> VcxResult<String> {
    let connection = CONNECTION_MAP.get_cloned(handle)?;
    connection
        .get_connection_info(&get_main_agency_client().unwrap())
        .await
        .map_err(|err| err.into())
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct MessageByConnection {
    #[serde(rename = "pairwiseDID")]
    pub pairwise_did: String,
    pub msgs: Vec<DownloadedMessage>,
}

pub fn parse_status_codes(status_codes: Option<Vec<String>>) -> VcxResult<Option<Vec<MessageStatusCode>>> {
    match status_codes {
        Some(codes) => {
            let codes = codes
                .iter()
                .map(|code| {
                    ::serde_json::from_str::<MessageStatusCode>(&format!("\"{}\"", code)).map_err(|err| {
                        VcxError::from_msg(
                            VcxErrorKind::InvalidJson,
                            format!("Cannot parse message status code: {}", err),
                        )
                    })
                })
                .collect::<VcxResult<Vec<MessageStatusCode>>>()?;
            Ok(Some(codes))
        }
        None => Ok(None),
    }
}

pub fn parse_connection_handles(conn_handles: Vec<String>) -> VcxResult<Vec<u32>> {
    trace!("parse_connection_handles >>> conn_handles: {:?}", conn_handles);
    let codes = conn_handles
        .iter()
        .map(|handle| {
            ::serde_json::from_str::<u32>(handle).map_err(|err| {
                VcxError::from_msg(
                    VcxErrorKind::InvalidJson,
                    format!("Cannot parse connection handles: {}", err),
                )
            })
        })
        .collect::<VcxResult<Vec<u32>>>()?;
    Ok(codes)
}

pub async fn download_messages(
    conn_handles: Vec<u32>,
    status_codes: Option<Vec<MessageStatusCode>>,
    uids: Option<Vec<String>>,
) -> VcxResult<Vec<MessageByConnection>> {
    trace!(
        "download_messages >>> cann_handles: {:?}, status_codes: {:?}, uids: {:?}",
        conn_handles,
        status_codes,
        uids
    );
    let mut res = Vec::new();
    let mut connections = Vec::new();
    for conn_handle in conn_handles {
        let connection = CONNECTION_MAP.get(conn_handle, |connection| Ok(connection.clone()))?;
        connections.push(connection)
    }
    for connection in connections {
        let msgs = connection
            .download_messages(&get_main_agency_client().unwrap(), status_codes.clone(), uids.clone())
            .await?;
        res.push(MessageByConnection {
            pairwise_did: connection.pairwise_info().pw_did.clone(),
            msgs,
        });
    }
    trace!("download_messages <<< res: {:?}", res);
    Ok(res)
}

#[cfg(test)]
pub mod tests {
    use serde_json::Value;

    use aries_vcx;
    use aries_vcx::agency_client::testing::mocking::AgencyMockDecrypted;
    use aries_vcx::global::settings;
    use aries_vcx::messages::connection::invite::test_utils::{_pairwise_invitation_json, _public_invitation_json};
    use aries_vcx::utils::constants;
    use aries_vcx::utils::devsetup::{SetupEmpty, SetupMocks};
    use aries_vcx::utils::mockdata::mockdata_connection::{
        ARIES_CONNECTION_ACK, ARIES_CONNECTION_INVITATION, ARIES_CONNECTION_REQUEST, CONNECTION_SM_INVITEE_COMPLETED,
    };

    use crate::api_lib::api_handle::agent::create_public_agent;
    use crate::api_lib::api_handle::mediated_connection;
    use crate::api_lib::VcxStateType;

    use super::*;

    pub async fn mock_connection() -> u32 {
        build_test_connection_inviter_requested().await
    }

    fn _setup() {
        let _setup = SetupEmpty::init();
    }

    fn _source_id() -> &'static str {
        "test connection"
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_create_connection_works() {
        let _setup = SetupMocks::init();
        let connection_handle = mediated_connection::create_connection(_source_id()).await.unwrap();
        assert!(mediated_connection::is_valid_handle(connection_handle));
        assert_eq!(0, mediated_connection::get_state(connection_handle));
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_create_connection_with_pairwise_invite() {
        let _setup = SetupMocks::init();
        let connection_handle = mediated_connection::create_connection_with_invite(_source_id(), &_pairwise_invitation_json())
            .await
            .unwrap();
        assert!(mediated_connection::is_valid_handle(connection_handle));
        assert_eq!(1, mediated_connection::get_state(connection_handle));
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_create_connection_with_public_invite() {
        let _setup = SetupMocks::init();
        let connection_handle = mediated_connection::create_connection_with_invite(_source_id(), &_public_invitation_json())
            .await
            .unwrap();
        assert!(mediated_connection::is_valid_handle(connection_handle));
        assert_eq!(1, mediated_connection::get_state(connection_handle));
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_create_connection_with_request() {
        let _setup = SetupMocks::init();
        let institution_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();
        let agent_handle = create_public_agent("test", &institution_did).await.unwrap();
        let connection_handle = mediated_connection::create_with_request(ARIES_CONNECTION_REQUEST, agent_handle)
            .await
            .unwrap();
        assert!(mediated_connection::is_valid_handle(connection_handle));
        assert_eq!(2, mediated_connection::get_state(connection_handle));
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_get_connection_state_works() {
        let _setup = SetupMocks::init();
        let connection_handle = mediated_connection::create_connection(_source_id()).await.unwrap();
        assert_eq!(0, mediated_connection::get_state(connection_handle));
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_connection_delete() {
        let _setup = SetupMocks::init();
        warn!(">> test_connection_delete going to create connection");
        let connection_handle = mediated_connection::create_connection(_source_id()).await.unwrap();
        warn!(">> test_connection_delete checking is valid handle");
        assert!(mediated_connection::is_valid_handle(connection_handle));

        mediated_connection::release(connection_handle).unwrap();
        assert!(!mediated_connection::is_valid_handle(connection_handle));
    }

    pub async fn build_test_connection_inviter_null() -> u32 {
        let handle = create_connection("faber_to_alice").await.unwrap();
        handle
    }

    pub async fn build_test_connection_inviter_invited() -> u32 {
        let handle = create_connection("faber_to_alice").await.unwrap();
        connect(handle).await.unwrap();
        handle
    }

    pub fn build_test_connection_invitee_completed() -> u32 {
        from_string(CONNECTION_SM_INVITEE_COMPLETED).unwrap()
    }

    pub async fn build_test_connection_inviter_requested() -> u32 {
        let handle = build_test_connection_inviter_invited().await;
        update_state_with_message(handle, ARIES_CONNECTION_REQUEST)
            .await
            .unwrap();
        handle
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_create_connection() {
        let _setup = SetupMocks::init();

        let handle = create_connection("test_create_connection").await.unwrap();
        assert_eq!(get_state(handle), VcxStateType::VcxStateNone as u32);

        connect(handle).await.unwrap();
        assert_eq!(get_pw_did(handle).unwrap(), constants::DID);
        assert_eq!(get_pw_verkey(handle).unwrap(), constants::VERKEY);

        AgencyMockDecrypted::set_next_decrypted_response(constants::GET_MESSAGES_DECRYPTED_RESPONSE);
        AgencyMockDecrypted::set_next_decrypted_message(ARIES_CONNECTION_REQUEST);
        update_state(handle).await.unwrap();
        assert_eq!(get_state(handle), VcxStateType::VcxStateRequestReceived as u32);

        AgencyMockDecrypted::set_next_decrypted_response(constants::GET_MESSAGES_DECRYPTED_RESPONSE);
        AgencyMockDecrypted::set_next_decrypted_message(ARIES_CONNECTION_ACK);
        update_state(handle).await.unwrap();
        assert_eq!(get_state(handle), VcxStateType::VcxStateAccepted as u32);

        // This errors b/c we release handle in delete connection
        assert!(release(handle).is_ok());
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_create_drop_create() {
        let _setup = SetupMocks::init();

        let handle = create_connection("test_create_drop_create").await.unwrap();

        assert_eq!(get_state(handle), VcxStateType::VcxStateNone as u32);
        let did1 = get_pw_did(handle).unwrap();

        release(handle).unwrap();

        let handle2 = create_connection("test_create_drop_create").await.unwrap();

        assert_eq!(get_state(handle2), VcxStateType::VcxStateNone as u32);
        let did2 = get_pw_did(handle2).unwrap();

        assert_ne!(handle, handle2);
        assert_eq!(did1, did2);

        release(handle2).unwrap();
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_connection_release_fails() {
        let _setup = SetupEmpty::init();

        let rc = release(1);
        assert_eq!(rc.unwrap_err().kind(), VcxErrorKind::InvalidConnectionHandle);
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_get_state_fails() {
        let _setup = SetupEmpty::init();

        let state = get_state(1);
        assert_eq!(state, VcxStateType::VcxStateNone as u32);
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_get_string_fails() {
        let _setup = SetupEmpty::init();

        let rc = to_string(0);
        assert_eq!(rc.unwrap_err().kind(), VcxErrorKind::InvalidHandle);
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_get_service_endpoint() {
        let _setup = SetupMocks::init();

        let handle = create_connection("test_get_qr_code_data").await.unwrap();

        connect(handle).await.unwrap();

        let details = get_invite_details(handle).unwrap();
        assert!(details.contains("\"serviceEndpoint\":"));

        assert_eq!(
            get_invite_details(0).unwrap_err().kind(),
            VcxErrorKind::InvalidConnectionHandle
        );
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_retry_connection() {
        let _setup = SetupMocks::init();

        let handle = create_connection("test_serialize_deserialize").await.unwrap();

        assert_eq!(get_state(handle), VcxStateType::VcxStateNone as u32);

        connect(handle).await.unwrap();
        connect(handle).await.unwrap();
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_release_all() {
        let _setup = SetupMocks::init();

        let h1 = create_connection("rel1").await.unwrap();
        let h2 = create_connection("rel2").await.unwrap();
        let h3 = create_connection("rel3").await.unwrap();
        let h4 = create_connection("rel4").await.unwrap();
        let h5 = create_connection("rel5").await.unwrap();
        release_all();
        assert_eq!(release(h1).unwrap_err().kind(), VcxErrorKind::InvalidConnectionHandle);
        assert_eq!(release(h2).unwrap_err().kind(), VcxErrorKind::InvalidConnectionHandle);
        assert_eq!(release(h3).unwrap_err().kind(), VcxErrorKind::InvalidConnectionHandle);
        assert_eq!(release(h4).unwrap_err().kind(), VcxErrorKind::InvalidConnectionHandle);
        assert_eq!(release(h5).unwrap_err().kind(), VcxErrorKind::InvalidConnectionHandle);
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_create_with_valid_invite_details() {
        let _setup = SetupMocks::init();

        let handle = create_connection_with_invite("alice", ARIES_CONNECTION_INVITATION)
            .await
            .unwrap();
        connect(handle).await.unwrap();

        let handle_2 = create_connection_with_invite("alice", ARIES_CONNECTION_INVITATION)
            .await
            .unwrap();
        connect(handle_2).await.unwrap();
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_process_acceptance_message() {
        let _setup = SetupMocks::init();

        let handle = create_connection("test_process_acceptance_message").await.unwrap();
        assert_eq!(
            error::SUCCESS.code_num,
            update_state_with_message(handle, ARIES_CONNECTION_REQUEST)
                .await
                .unwrap()
        );
    }

    //     #[tokio::test]
    //     #[cfg(feature = "general_test")]
    //     async fn test_connection_handle_is_found() {
    //         let _setup = SetupMocks::init();
    //         let handle = create_connection_with_invite("alice", ARIES_CONNECTION_INVITATION).await.unwrap();
    //
    //         CONNECTION_MAP.get_mut(handle, |_connection| {
    //             { Ok(()) }.boxed()
    //         }).await.unwrap();
    //     }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_send_generic_message_fails_with_invalid_connection() {
        let _setup = SetupMocks::init();

        let handle = mediated_connection::tests::build_test_connection_inviter_invited().await;

        let err = send_generic_message(handle, "this is the message").await.unwrap_err();
        assert_eq!(err.kind(), VcxErrorKind::NotReady);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_generate_public_invitation() {
        let _setup = SetupMocks::init();

        let invitation = generate_public_invitation(constants::INSTITUTION_DID, "faber-enterprise").unwrap();
        let parsed: Value = serde_json::from_str(&invitation).unwrap();
        assert!(parsed["@id"].is_string());
        assert_eq!(parsed["@type"], "https://didcomm.org/connections/1.0/invitation");
        assert_eq!(parsed["did"], constants::INSTITUTION_DID);
        assert_eq!(parsed["label"], "faber-enterprise");
    }
}
