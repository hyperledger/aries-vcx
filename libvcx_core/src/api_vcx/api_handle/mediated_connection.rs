use std::collections::HashMap;

use aries_vcx::{
    agency_client::{api::downloaded_message::DownloadedMessage, MessageStatusCode},
    common::ledger::transactions::into_did_doc,
    handlers::{connection::mediated_connection::MediatedConnection, util::AnyInvitation},
    messages::{
        msg_fields::protocols::connection::{
            invitation::{Invitation, InvitationContent},
            request::Request,
        },
        AriesMessage,
    },
    protocols::mediated_connection::pairwise_info::PairwiseInfo,
};
use serde_json;
use uuid::Uuid;

use crate::{
    api_vcx::{
        api_global::{
            agency_client::get_main_agency_client,
            profile::{get_main_ledger_read, get_main_wallet},
            wallet::{wallet_sign, wallet_verify},
        },
        api_handle::object_cache::ObjectCache,
    },
    errors::error::{LibvcxError, LibvcxErrorKind, LibvcxResult},
};

lazy_static! {
    pub static ref CONNECTION_MAP: ObjectCache<MediatedConnection> =
        ObjectCache::<MediatedConnection>::new("connections-cache");
}

pub fn generate_public_invitation(public_did: &str, label: &str) -> LibvcxResult<String> {
    trace!(
        "generate_public_invite >>> label: {}, public_did: {}",
        public_did,
        label
    );
    let content = InvitationContent::builder_public()
        .label(label.to_owned())
        .did(public_did.to_owned())
        .build();
    let invite: Invitation = Invitation::builder()
        .id(Uuid::new_v4().to_string())
        .content(content)
        .build();

    let invitation = AriesMessage::from(invite);
    Ok(json!(invitation).to_string())
}

pub fn is_valid_handle(handle: u32) -> bool {
    CONNECTION_MAP.has_handle(handle)
}

pub fn get_agent_did(handle: u32) -> LibvcxResult<String> {
    CONNECTION_MAP.get(handle, |connection| {
        Ok(connection
            .cloud_agent_info()
            .ok_or(LibvcxError::from_msg(
                LibvcxErrorKind::NoAgentInformation,
                "Missing cloud agent info",
            ))?
            .agent_did)
    })
}

pub fn get_agent_verkey(handle: u32) -> LibvcxResult<String> {
    CONNECTION_MAP.get(handle, |connection| {
        Ok(connection
            .cloud_agent_info()
            .ok_or(LibvcxError::from_msg(
                LibvcxErrorKind::NoAgentInformation,
                "Missing cloud agent info",
            ))?
            .agent_vk)
    })
}

pub fn get_pw_did(handle: u32) -> LibvcxResult<String> {
    CONNECTION_MAP.get(handle, |connection| {
        Ok(connection.pairwise_info().pw_did.to_string())
    })
}

pub fn get_pw_verkey(handle: u32) -> LibvcxResult<String> {
    CONNECTION_MAP.get(handle, |connection| {
        Ok(connection.pairwise_info().pw_vk.clone())
    })
}

pub fn get_their_pw_did(handle: u32) -> LibvcxResult<String> {
    let connection = CONNECTION_MAP.get_cloned(handle)?;
    connection.remote_did().map_err(|err| err.into())
}

pub fn get_their_pw_verkey(handle: u32) -> LibvcxResult<String> {
    let connection = CONNECTION_MAP.get_cloned(handle)?;
    connection.remote_vk().map_err(|err| err.into())
}

pub async fn verify_signature(
    connection_handle: u32,
    data: &[u8],
    signature: &[u8],
) -> LibvcxResult<bool> {
    let vk = get_their_pw_verkey(connection_handle)?;
    wallet_verify(&vk, data, signature).await
}

pub async fn sign_data(connection_handle: u32, data: &[u8]) -> LibvcxResult<Vec<u8>> {
    let vk = get_pw_verkey(connection_handle)?;
    wallet_sign(&vk, data).await
}

pub fn get_thread_id(handle: u32) -> LibvcxResult<String> {
    CONNECTION_MAP.get(handle, |connection| Ok(connection.get_thread_id()))
}

pub fn get_state(handle: u32) -> u32 {
    trace!("get_state >>> handle = {:?}", handle);
    CONNECTION_MAP
        .get(handle, |connection| Ok(connection.get_state().into()))
        .unwrap_or(0)
}

pub fn get_source_id(handle: u32) -> LibvcxResult<String> {
    CONNECTION_MAP.get(handle, |connection| Ok(connection.get_source_id()))
}

pub fn store_connection(connection: MediatedConnection) -> LibvcxResult<u32> {
    CONNECTION_MAP
        .add(connection)
        .map_err(|e| LibvcxError::from_msg(LibvcxErrorKind::CreateConnection, e.to_string()))
}

pub async fn create_connection(source_id: &str) -> LibvcxResult<u32> {
    trace!("create_connection >>> source_id: {}", source_id);
    let connection = MediatedConnection::create(
        source_id,
        get_main_wallet()?.as_ref(),
        &get_main_agency_client()?,
        true,
    )
    .await?;
    store_connection(connection)
}

pub async fn create_connection_with_invite(source_id: &str, details: &str) -> LibvcxResult<u32> {
    debug!("create connection {} with invite {}", source_id, details);
    if let Ok(invitation) = serde_json::from_str::<AnyInvitation>(details) {
        let ddo = into_did_doc(get_main_ledger_read()?.as_ref(), &invitation).await?;
        let connection = MediatedConnection::create_with_invite(
            source_id,
            get_main_wallet()?.as_ref(),
            &get_main_agency_client()?,
            invitation,
            ddo,
            true,
        )
        .await?;
        store_connection(connection)
    } else {
        Err(LibvcxError::from_msg(
            LibvcxErrorKind::InvalidJson,
            "Used invite has invalid structure",
        ))
    }
}

pub async fn create_with_request_v2(request: &str, pw_info: PairwiseInfo) -> LibvcxResult<u32> {
    let request: Request = serde_json::from_str(request).map_err(|err| {
        LibvcxError::from_msg(
            LibvcxErrorKind::InvalidJson,
            format!("Cannot deserialize connection request: {:?}", err),
        )
    })?;

    let connection = MediatedConnection::create_with_request(
        get_main_wallet()?.as_ref(),
        request,
        pw_info,
        &get_main_agency_client()?,
    )
    .await?;
    store_connection(connection)
}

pub async fn send_generic_message(handle: u32, msg: &str) -> LibvcxResult<String> {
    let connection = CONNECTION_MAP.get_cloned(handle)?;

    connection
        .send_generic_message(get_main_wallet()?.as_ref(), msg)
        .await
        .map_err(|err| err.into())
}

pub async fn send_handshake_reuse(handle: u32, oob_msg: &str) -> LibvcxResult<()> {
    let connection = CONNECTION_MAP.get_cloned(handle)?;

    connection
        .send_handshake_reuse(get_main_wallet()?.as_ref(), oob_msg)
        .await
        .map_err(|err| err.into())
}

pub async fn update_state_with_message(handle: u32, message: &str) -> LibvcxResult<u32> {
    let mut connection = CONNECTION_MAP.get_cloned(handle)?;
    let message: AriesMessage = serde_json::from_str(message).map_err(|err| {
        LibvcxError::from_msg(
            LibvcxErrorKind::InvalidJson,
            format!(
                "Failed to deserialize message {} into A2AMessage, err: {:?}",
                message, err
            ),
        )
    })?;

    connection
        .update_state_with_message(
            get_main_wallet()?.as_ref(),
            get_main_agency_client()?,
            Some(message),
        )
        .await?;
    let state: u32 = connection.get_state().into();
    CONNECTION_MAP.insert(handle, connection)?;
    Ok(state)
}

pub async fn handle_message(handle: u32, message: &str) -> LibvcxResult<()> {
    let mut connection = CONNECTION_MAP.get_cloned(handle)?;
    let message: AriesMessage = serde_json::from_str(message).map_err(|err| {
        LibvcxError::from_msg(
            LibvcxErrorKind::InvalidJson,
            format!(
                "Failed to deserialize message {} into A2AMessage, err: {:?}",
                message, err
            ),
        )
    })?;

    connection
        .handle_message(message, get_main_wallet()?.as_ref())
        .await?;
    CONNECTION_MAP.insert(handle, connection)
}

pub async fn update_state(handle: u32) -> LibvcxResult<u32> {
    let mut connection = CONNECTION_MAP.get_cloned(handle)?;
    if connection.is_in_final_state() {
        info!(
            "connection::update_state >> connection {} is in final state, trying to respond to \
             messages",
            handle
        );

        connection
            .find_and_handle_message(get_main_wallet()?.as_ref(), &get_main_agency_client()?)
            .await?
    } else {
        info!(
            "connection::update_state >> connection {} is not in final state, trying to update \
             state",
            handle
        );

        connection
            .find_message_and_update_state(get_main_wallet()?.as_ref(), &get_main_agency_client()?)
            .await?
    };
    let state: u32 = connection.get_state().into();
    CONNECTION_MAP.insert(handle, connection)?;
    Ok(state)
}

pub async fn delete_connection(handle: u32) -> LibvcxResult<()> {
    let connection = CONNECTION_MAP.get_cloned(handle)?;
    connection.delete(&get_main_agency_client()?).await?;
    release(handle)
}

pub async fn connect(handle: u32) -> LibvcxResult<Option<String>> {
    let mut connection = CONNECTION_MAP.get_cloned(handle)?;

    connection
        .connect(
            get_main_wallet()?.as_ref(),
            &get_main_agency_client()?,
            None,
        )
        .await?;
    let invitation = connection
        .get_invite_details()
        .map(|invitation| match invitation {
            AnyInvitation::Con(invitation) => {
                json!(AriesMessage::from(invitation.clone())).to_string()
            }
            AnyInvitation::Oob(invitation) => {
                json!(AriesMessage::from(invitation.clone())).to_string()
            }
        });
    CONNECTION_MAP.insert(handle, connection)?;
    Ok(invitation)
}

pub fn to_string(handle: u32) -> LibvcxResult<String> {
    CONNECTION_MAP.get(handle, |connection| {
        connection.to_string().map_err(|err| err.into())
    })
}

pub fn from_string(connection_data: &str) -> LibvcxResult<u32> {
    let connection = MediatedConnection::from_string(connection_data)?;
    CONNECTION_MAP.add(connection)
}

pub fn release(handle: u32) -> LibvcxResult<()> {
    CONNECTION_MAP
        .release(handle)
        .map_err(|e| LibvcxError::from_msg(LibvcxErrorKind::InvalidConnectionHandle, e.to_string()))
}

pub fn release_all() {
    CONNECTION_MAP.drain().ok();
}

pub fn get_invite_details(handle: u32) -> LibvcxResult<String> {
    CONNECTION_MAP
        .get(handle, |connection| {
            connection
                .get_invite_details()
                .map(|invitation| match invitation {
                    AnyInvitation::Con(invitation) => {
                        json!(AriesMessage::from(invitation.clone())).to_string()
                    }
                    AnyInvitation::Oob(invitation) => {
                        json!(AriesMessage::from(invitation.clone())).to_string()
                    }
                })
                .ok_or(LibvcxError::from_msg(
                    LibvcxErrorKind::ActionNotSupported,
                    "Invitation is not available for the connection.",
                ))
        })
        .map_err(|e| LibvcxError::from_msg(LibvcxErrorKind::InvalidConnectionHandle, e.to_string()))
}

pub async fn get_messages(handle: u32) -> LibvcxResult<HashMap<String, AriesMessage>> {
    let connection = CONNECTION_MAP.get_cloned(handle)?;
    connection
        .get_messages(&get_main_agency_client()?)
        .await
        .map_err(|err| err.into())
}

pub async fn update_message_status(handle: u32, uid: &str) -> LibvcxResult<()> {
    let connection = CONNECTION_MAP.get_cloned(handle)?;
    connection
        .update_message_status(uid, &get_main_agency_client()?)
        .await
        .map_err(|err| err.into())
}

pub async fn get_message_by_id(handle: u32, msg_id: &str) -> LibvcxResult<AriesMessage> {
    let connection = CONNECTION_MAP.get_cloned(handle)?;
    connection
        .get_message_by_id(msg_id, &get_main_agency_client()?)
        .await
        .map_err(|err| err.into())
}

pub async fn send_message(handle: u32, message: AriesMessage) -> LibvcxResult<()> {
    trace!("connection::send_message >>>");
    let connection = CONNECTION_MAP.get_cloned(handle)?;
    let wallet = get_main_wallet()?;
    let send_message = connection.send_message_closure(wallet.as_ref()).await?;
    send_message(message).await.map_err(|err| err.into())
}

pub async fn send_ping(handle: u32, comment: Option<&str>) -> LibvcxResult<()> {
    let mut connection = CONNECTION_MAP.get_cloned(handle)?;

    connection
        .send_ping(get_main_wallet()?.as_ref(), comment.map(String::from))
        .await?;
    CONNECTION_MAP.insert(handle, connection)
}

pub async fn send_discovery_features(
    handle: u32,
    query: Option<&str>,
    comment: Option<&str>,
) -> LibvcxResult<()> {
    let connection = CONNECTION_MAP.get_cloned(handle)?;

    connection
        .send_discovery_query(
            get_main_wallet()?.as_ref(),
            query.map(String::from),
            comment.map(String::from),
        )
        .await?;
    CONNECTION_MAP.insert(handle, connection)
}

pub async fn get_connection_info(handle: u32) -> LibvcxResult<String> {
    let connection = CONNECTION_MAP.get_cloned(handle)?;
    connection
        .get_connection_info(&get_main_agency_client()?)
        .await
        .map_err(|err| err.into())
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct MessageByConnection {
    #[serde(rename = "pairwiseDID")]
    pub pairwise_did: String,
    pub msgs: Vec<DownloadedMessage>,
}

pub fn parse_status_codes(
    status_codes: Option<Vec<String>>,
) -> LibvcxResult<Option<Vec<MessageStatusCode>>> {
    match status_codes {
        Some(codes) => {
            let codes = codes
                .iter()
                .map(|code| {
                    ::serde_json::from_str::<MessageStatusCode>(&format!("\"{}\"", code)).map_err(
                        |err| {
                            LibvcxError::from_msg(
                                LibvcxErrorKind::InvalidJson,
                                format!("Cannot parse message status code: {}", err),
                            )
                        },
                    )
                })
                .collect::<LibvcxResult<Vec<MessageStatusCode>>>()?;
            Ok(Some(codes))
        }
        None => Ok(None),
    }
}

pub fn parse_connection_handles(conn_handles: Vec<String>) -> LibvcxResult<Vec<u32>> {
    trace!(
        "parse_connection_handles >>> conn_handles: {:?}",
        conn_handles
    );
    let codes = conn_handles
        .iter()
        .map(|handle| {
            ::serde_json::from_str::<u32>(handle).map_err(|err| {
                LibvcxError::from_msg(
                    LibvcxErrorKind::InvalidJson,
                    format!("Cannot parse connection handles: {}", err),
                )
            })
        })
        .collect::<LibvcxResult<Vec<u32>>>()?;
    Ok(codes)
}

pub async fn download_messages(
    conn_handles: Vec<u32>,
    status_codes: Option<Vec<MessageStatusCode>>,
    uids: Option<Vec<String>>,
) -> LibvcxResult<Vec<MessageByConnection>> {
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
            .download_messages(
                &get_main_agency_client()?,
                status_codes.clone(),
                uids.clone(),
            )
            .await?;
        res.push(MessageByConnection {
            pairwise_did: connection.pairwise_info().pw_did.clone(),
            msgs,
        });
    }
    trace!("download_messages <<< res: {:?}", res);
    Ok(res)
}

#[allow(clippy::unwrap_used)]
pub mod test_utils {
    use aries_vcx::utils::mockdata::mockdata_mediated_connection::{
        ARIES_CONNECTION_REQUEST, CONNECTION_SM_INVITEE_COMPLETED,
    };

    use super::*;

    pub async fn mock_connection() -> u32 {
        build_test_connection_inviter_requested().await
    }

    pub async fn build_test_connection_inviter_null() -> u32 {
        create_connection("faber_to_alice").await.unwrap()
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
}

#[cfg(test)]
pub mod tests {
    use aries_vcx::{
        self,
        utils::{constants, devsetup::SetupMocks},
    };
    use serde_json::Value;

    use super::*;
    use crate::api_vcx::VcxStateType;

    fn _setup() {
        let _setup = SetupMocks::init();
    }

    fn _source_id() -> &'static str {
        "test connection"
    }

    #[tokio::test]
    async fn test_get_state_fails() {
        let _setup = SetupMocks::init();

        let state = get_state(1);
        assert_eq!(state, VcxStateType::VcxStateNone as u32);
    }

    #[tokio::test]
    async fn test_get_string_fails() {
        let _setup = SetupMocks::init();

        let rc = to_string(0);
        assert_eq!(rc.unwrap_err().kind(), LibvcxErrorKind::InvalidHandle);
    }

    #[test]
    fn test_generate_public_invitation() {
        let _setup = SetupMocks::init();

        let invitation =
            generate_public_invitation(constants::INSTITUTION_DID, "faber-enterprise").unwrap();
        let parsed: Value = serde_json::from_str(&invitation).unwrap();
        assert!(parsed["@id"].is_string());
        assert_eq!(
            parsed["@type"],
            "https://didcomm.org/connections/1.0/invitation"
        );
        assert_eq!(parsed["did"], constants::INSTITUTION_DID);
        assert_eq!(parsed["label"], "faber-enterprise");
    }
}
