use aries_vcx::{
    common::proofs::proof_request::PresentationRequestData,
    handlers::proof_presentation::{
        mediated_verifier::verifier_find_message_to_handle, verifier::Verifier,
    },
    messages::AriesMessage,
    protocols::{
        proof_presentation::verifier::verification_status::PresentationVerificationStatus,
        SendClosure,
    },
};
use serde_json;

use crate::{
    api_vcx::{
        api_global::profile::{
            get_main_anoncreds, get_main_anoncreds_ledger_read, get_main_wallet,
        },
        api_handle::{
            connection,
            connection::HttpClient,
            mediated_connection::{self, send_message},
            object_cache::ObjectCache,
        },
    },
    errors::error::{LibvcxError, LibvcxErrorKind, LibvcxResult},
};

lazy_static! {
    static ref PROOF_MAP: ObjectCache<Verifier> = ObjectCache::<Verifier>::new("proofs-cache");
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "version", content = "data")]
enum Proofs {
    #[serde(rename = "2.0")]
    V3(Verifier),
}

pub async fn create_proof(
    source_id: String,
    requested_attrs: String,
    requested_predicates: String,
    revocation_details: String,
    name: String,
) -> LibvcxResult<u32> {
    let presentation_request = PresentationRequestData::create(&get_main_anoncreds()?, &name)
        .await?
        .set_requested_attributes_as_string(requested_attrs)?
        .set_requested_predicates_as_string(requested_predicates)?
        .set_not_revoked_interval(revocation_details)?;
    let verifier = Verifier::create_from_request(source_id, &presentation_request)?;
    PROOF_MAP.add(verifier)
}

pub fn is_valid_handle(handle: u32) -> bool {
    PROOF_MAP.has_handle(handle)
}

pub async fn update_state(
    handle: u32,
    message: Option<&str>,
    connection_handle: u32,
) -> LibvcxResult<u32> {
    let mut proof = PROOF_MAP.get_cloned(handle)?;
    trace!(
        "proof::update_state >>> handle: {}, message: {:?}, connection_handle: {}",
        handle,
        message,
        connection_handle
    );
    if !proof.progressable_by_message() {
        return Ok(proof.get_state().into());
    }

    if let Some(message) = message {
        let message: AriesMessage = serde_json::from_str(message).map_err(|err| {
            LibvcxError::from_msg(
                LibvcxErrorKind::InvalidOption,
                format!(
                    "Cannot updated state with message: Message deserialization failed: {:?}",
                    err
                ),
            )
        })?;
        trace!(
            "proof::update_state >>> updating using message {:?}",
            message
        );
        if let Some(message) = proof
            .process_aries_msg(
                &get_main_anoncreds_ledger_read()?,
                &get_main_anoncreds()?,
                message,
            )
            .await?
        {
            send_message(connection_handle, message).await?;
        }
    } else {
        let messages = mediated_connection::get_messages(connection_handle).await?;
        trace!("proof::update_state >>> found messages: {:?}", messages);
        if let Some((uid, message)) = verifier_find_message_to_handle(&proof, messages) {
            if let Some(message) = proof
                .process_aries_msg(
                    &get_main_anoncreds_ledger_read()?,
                    &get_main_anoncreds()?,
                    message,
                )
                .await?
            {
                send_message(connection_handle, message).await?;
            }
            mediated_connection::update_message_status(connection_handle, &uid).await?;
        };
    }
    let state: u32 = proof.get_state().into();
    PROOF_MAP.insert(handle, proof)?;
    Ok(state)
}

pub async fn update_state_nonmediated(
    handle: u32,
    connection_handle: u32,
    message: &str,
) -> LibvcxResult<u32> {
    let mut proof = PROOF_MAP.get_cloned(handle)?;
    trace!(
        "proof::update_state_nonmediated >>> handle: {}, message: {:?}, connection_handle: {}",
        handle,
        message,
        connection_handle
    );
    if !proof.progressable_by_message() {
        return Ok(proof.get_state().into());
    }

    let con = connection::get_cloned_generic_connection(&connection_handle)?;
    let wallet = get_main_wallet()?;

    let send_message: SendClosure = Box::new(|msg: AriesMessage| {
        Box::pin(async move { con.send_message(&wallet, &msg, &HttpClient).await })
    });

    let message: AriesMessage = serde_json::from_str(message).map_err(|err| {
        LibvcxError::from_msg(
            LibvcxErrorKind::InvalidOption,
            format!(
                "Cannot updated state with message: Message deserialization failed: {:?}",
                err
            ),
        )
    })?;
    if let Some(message) = proof
        .process_aries_msg(
            &get_main_anoncreds_ledger_read()?,
            &get_main_anoncreds()?,
            message,
        )
        .await?
    {
        send_message(message).await?;
    }

    let state: u32 = proof.get_state().into();
    PROOF_MAP.insert(handle, proof)?;
    Ok(state)
}

pub fn get_state(handle: u32) -> LibvcxResult<u32> {
    PROOF_MAP.get(handle, |proof| Ok(proof.get_state().into()))
}

pub fn release(handle: u32) -> LibvcxResult<()> {
    PROOF_MAP
        .release(handle)
        .map_err(|e| LibvcxError::from_msg(LibvcxErrorKind::InvalidProofHandle, e.to_string()))
}

pub fn release_all() {
    PROOF_MAP.drain().ok();
}

pub fn to_string(handle: u32) -> LibvcxResult<String> {
    PROOF_MAP.get(handle, |proof| {
        serde_json::to_string(&Proofs::V3(proof.clone())).map_err(|err| {
            LibvcxError::from_msg(
                LibvcxErrorKind::InvalidState,
                format!("cannot serialize Proof proofect: {:?}", err),
            )
        })
    })
}

pub fn get_source_id(handle: u32) -> LibvcxResult<String> {
    PROOF_MAP.get(handle, |proof| Ok(proof.get_source_id()))
}

pub fn from_string(proof_data: &str) -> LibvcxResult<u32> {
    let proof: Proofs = serde_json::from_str(proof_data).map_err(|err| {
        LibvcxError::from_msg(
            LibvcxErrorKind::InvalidJson,
            format!("cannot deserialize Proofs proofect: {:?}", err),
        )
    })?;

    match proof {
        Proofs::V3(proof) => PROOF_MAP.add(proof),
    }
}

pub async fn send_proof_request(handle: u32, connection_handle: u32) -> LibvcxResult<()> {
    let mut proof = PROOF_MAP.get_cloned(handle)?;
    let message = proof.mark_presentation_request_sent()?;
    send_message(connection_handle, message.into()).await?;
    PROOF_MAP.insert(handle, proof)
}

pub async fn send_proof_request_nonmediated(
    handle: u32,
    connection_handle: u32,
) -> LibvcxResult<()> {
    let mut proof = PROOF_MAP.get_cloned(handle)?;

    let con = connection::get_cloned_generic_connection(&connection_handle)?;
    let wallet = get_main_wallet()?;

    let send_message: SendClosure = Box::new(|msg: AriesMessage| {
        Box::pin(async move { con.send_message(&wallet, &msg, &HttpClient).await })
    });

    let message = proof.mark_presentation_request_sent()?;
    send_message(message.into()).await?;

    PROOF_MAP.insert(handle, proof)
}

// --- Presentation request ---
pub fn mark_presentation_request_msg_sent(handle: u32) -> LibvcxResult<()> {
    let mut proof = PROOF_MAP.get_cloned(handle)?;
    proof.mark_presentation_request_sent()?;
    PROOF_MAP.insert(handle, proof)
}

pub fn get_presentation_request_attachment(handle: u32) -> LibvcxResult<String> {
    PROOF_MAP.get(handle, |proof| {
        proof
            .get_presentation_request_attachment()
            .map_err(|err| err.into())
    })
}

pub fn get_presentation_request_msg(handle: u32) -> LibvcxResult<String> {
    PROOF_MAP.get(handle, |proof| {
        let msg = AriesMessage::from(proof.get_presentation_request_msg()?);
        Ok(json!(msg).to_string())
    })
}

// --- Presentation ---
pub fn get_presentation_msg(handle: u32) -> LibvcxResult<String> {
    PROOF_MAP.get(handle, |proof| {
        let msg = AriesMessage::from(proof.get_presentation_msg()?);
        Ok(json!(msg).to_string())
    })
}

pub fn get_presentation_attachment(handle: u32) -> LibvcxResult<String> {
    PROOF_MAP.get(handle, |proof| {
        proof
            .get_presentation_attachment()
            .map_err(|err| err.into())
    })
}

pub fn get_verification_status(handle: u32) -> LibvcxResult<VcxPresentationVerificationStatus> {
    PROOF_MAP.get(handle, |proof| Ok(proof.get_verification_status().into()))
}

#[derive(Debug, PartialEq, Eq)]
pub enum VcxPresentationVerificationStatus {
    Valid,
    Invalid,
    Unavailable,
}

impl VcxPresentationVerificationStatus {
    pub fn code(&self) -> u32 {
        match self {
            VcxPresentationVerificationStatus::Unavailable => 0,
            VcxPresentationVerificationStatus::Valid => 1,
            VcxPresentationVerificationStatus::Invalid => 2,
        }
    }
}

impl From<PresentationVerificationStatus> for VcxPresentationVerificationStatus {
    fn from(verification_status: PresentationVerificationStatus) -> Self {
        match verification_status {
            PresentationVerificationStatus::Valid => VcxPresentationVerificationStatus::Valid,
            PresentationVerificationStatus::Invalid => VcxPresentationVerificationStatus::Invalid,
            PresentationVerificationStatus::Unavailable => {
                VcxPresentationVerificationStatus::Unavailable
            }
        }
    }
}

// --- General ---
pub fn get_thread_id(handle: u32) -> LibvcxResult<String> {
    PROOF_MAP.get(handle, |proof| {
        proof.get_thread_id().map_err(|err| err.into())
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
pub mod tests {
    use aries_vcx::utils::{
        constants::{REQUESTED_ATTRS, REQUESTED_PREDICATES, V3_OBJECT_SERIALIZE_VERSION},
        devsetup::SetupMocks,
    };
    use serde_json::Value;

    use super::*;

    async fn create_default_proof() -> u32 {
        create_proof(
            "1".to_string(),
            REQUESTED_ATTRS.to_owned(),
            REQUESTED_PREDICATES.to_owned(),
            r#"{"support_revocation":false}"#.to_string(),
            "Optional".to_owned(),
        )
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn test_get_proof_returns_proof_with_proof_state_invalid() {
        let _setup = SetupMocks::init();
        let handle = create_default_proof().await;
        release(handle).unwrap();
        assert_eq!(
            to_string(handle).unwrap_err().kind,
            LibvcxErrorKind::InvalidHandle
        )
    }

    #[tokio::test]
    async fn test_create_proof_succeeds() {
        let _setup = SetupMocks::init();
        create_default_proof().await;
    }

    #[tokio::test]
    async fn test_revocation_details() {
        let _setup = SetupMocks::init();

        // No Revocation
        create_default_proof().await;

        // Support Revocation Success
        let revocation_details = json!({
            "to": 1234,
        });
        create_proof(
            "1".to_string(),
            REQUESTED_ATTRS.to_owned(),
            REQUESTED_PREDICATES.to_owned(),
            revocation_details.to_string(),
            "Optional".to_owned(),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_to_string_succeeds() {
        let _setup = SetupMocks::init();

        let handle = create_default_proof().await;
        let proof_string = to_string(handle).unwrap();
        let s: Value = serde_json::from_str(&proof_string).unwrap();
        assert_eq!(s["version"], V3_OBJECT_SERIALIZE_VERSION);
        assert!(s["data"]["verifier_sm"].is_object());
        assert!(!proof_string.is_empty());
    }

    #[tokio::test]
    async fn test_from_string_succeeds() {
        let _setup = SetupMocks::init();

        let handle = create_default_proof().await;
        let proof_data = to_string(handle).unwrap();
        let _hnadle2 = from_string(&proof_data).unwrap();
        let proof_data2 = to_string(handle).unwrap();
        assert_eq!(proof_data, proof_data2);
    }

    #[tokio::test]
    async fn test_release_proof() {
        let _setup = SetupMocks::init();

        let handle = create_default_proof().await;
        assert!(release(handle).is_ok());
        assert!(!is_valid_handle(handle));
    }

    #[tokio::test]
    async fn test_get_proof_fails_with_no_proof() {
        let _setup = SetupMocks::init();

        let handle = create_default_proof().await;
        assert!(is_valid_handle(handle));
        assert!(get_presentation_msg(handle).is_err())
    }

    #[tokio::test]
    async fn test_release_all() {
        let _setup = SetupMocks::init();

        let h1 = create_proof(
            "1".to_string(),
            REQUESTED_ATTRS.to_owned(),
            REQUESTED_PREDICATES.to_owned(),
            r#"{"support_revocation":false}"#.to_string(),
            "Optional".to_owned(),
        )
        .await
        .unwrap();
        let h2 = create_proof(
            "1".to_string(),
            REQUESTED_ATTRS.to_owned(),
            REQUESTED_PREDICATES.to_owned(),
            r#"{"support_revocation":false}"#.to_string(),
            "Optional".to_owned(),
        )
        .await
        .unwrap();
        let h3 = create_proof(
            "1".to_string(),
            REQUESTED_ATTRS.to_owned(),
            REQUESTED_PREDICATES.to_owned(),
            r#"{"support_revocation":false}"#.to_string(),
            "Optional".to_owned(),
        )
        .await
        .unwrap();
        release_all();
        assert!(!is_valid_handle(h1));
        assert!(!is_valid_handle(h2));
        assert!(!is_valid_handle(h3));
    }
}
