use serde_json;

use crate::api_lib::global::profile::get_main_profile;
use aries_vcx::error::{VcxError, VcxErrorKind, VcxResult};
use aries_vcx::handlers::proof_presentation::verifier::Verifier;
use aries_vcx::common::proofs::proof_request::PresentationRequestData;
use aries_vcx::messages::a2a::A2AMessage;
use aries_vcx::utils::error;

use crate::api_lib::api_handle::mediated_connection;
use crate::api_lib::api_handle::object_cache::ObjectCache;

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
) -> VcxResult<u32> {
    let profile = get_main_profile()?;
    let presentation_request = PresentationRequestData::create(&profile, &name)
        .await?
        .set_requested_attributes_as_string(requested_attrs)?
        .set_requested_predicates_as_string(requested_predicates)?
        .set_not_revoked_interval(revocation_details)?;
    let verifier = Verifier::create_from_request(source_id, &presentation_request)?;
    PROOF_MAP
        .add(verifier)
        .or(Err(VcxError::from(VcxErrorKind::CreateProof)))
}

pub async fn is_valid_handle(handle: u32) -> bool {
    PROOF_MAP.has_handle(handle)
}

pub async fn update_state(handle: u32, message: Option<&str>, connection_handle: u32) -> VcxResult<u32> {
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
    let send_message = mediated_connection::send_message_closure(connection_handle).await?;
    let profile = get_main_profile()?;

    if let Some(message) = message {
        let message: A2AMessage = serde_json::from_str(message).map_err(|err| {
            VcxError::from_msg(
                VcxErrorKind::InvalidOption,
                format!(
                    "Cannot updated state with message: Message deserialization failed: {:?}",
                    err
                ),
            )
        })?;
        trace!("proof::update_state >>> updating using message {:?}", message);
        proof
            .handle_message(
                &profile,
                message.into(),
                Some(send_message),
            )
            .await?;
    } else {
        let messages = mediated_connection::get_messages(connection_handle).await?;
        trace!("proof::update_state >>> found messages: {:?}", messages);
        if let Some((uid, message)) = proof.find_message_to_handle(messages) {
            proof
                .handle_message(
                    &profile,
                    message.into(),
                    Some(send_message),
                )
                .await?;
            mediated_connection::update_message_status(connection_handle, &uid).await?;
        };
    }
    let state: u32 = proof.get_state().into();
    PROOF_MAP.insert(handle, proof)?;
    Ok(state)
}

pub async fn get_state(handle: u32) -> VcxResult<u32> {
    PROOF_MAP.get(handle, |proof| Ok(proof.get_state().into()))
}

pub async fn get_proof_state(handle: u32) -> VcxResult<u32> {
    PROOF_MAP.get(handle, |proof| Ok(proof.get_presentation_status().code()))
}

pub fn release(handle: u32) -> VcxResult<()> {
    PROOF_MAP
        .release(handle)
        .or(Err(VcxError::from(VcxErrorKind::InvalidProofHandle)))
}

pub fn release_all() {
    PROOF_MAP.drain().ok();
}

pub async fn to_string(handle: u32) -> VcxResult<String> {
    PROOF_MAP.get(handle, |proof| {
        serde_json::to_string(&Proofs::V3(proof.clone())).map_err(|err| {
            VcxError::from_msg(
                VcxErrorKind::InvalidState,
                format!("cannot serialize Proof proofect: {:?}", err),
            )
        })
    })
}

pub fn get_source_id(handle: u32) -> VcxResult<String> {
    PROOF_MAP.get(handle, |proof| Ok(proof.get_source_id()))
}

pub async fn from_string(proof_data: &str) -> VcxResult<u32> {
    let proof: Proofs = serde_json::from_str(proof_data).map_err(|err| {
        VcxError::from_msg(
            VcxErrorKind::InvalidJson,
            format!("cannot deserialize Proofs proofect: {:?}", err),
        )
    })?;

    match proof {
        Proofs::V3(proof) => PROOF_MAP.add(proof),
    }
}

pub async fn send_proof_request(handle: u32, connection_handle: u32) -> VcxResult<u32> {
    let mut proof = PROOF_MAP.get_cloned(handle)?;
    proof
        .send_presentation_request(mediated_connection::send_message_closure(connection_handle).await?)
        .await?;
    PROOF_MAP.insert(handle, proof)?;
    Ok(error::SUCCESS.code_num)
}

pub async fn mark_presentation_request_msg_sent(handle: u32) -> VcxResult<()> {
    let mut proof = PROOF_MAP.get_cloned(handle)?;
    proof.mark_presentation_request_msg_sent()?;
    PROOF_MAP.insert(handle, proof)
}

pub async fn get_presentation_request_msg(handle: u32) -> VcxResult<String> {
    PROOF_MAP.get(handle, |proof| {
        proof.get_presentation_request_msg().map_err(|err| err.into())
    })
}

pub async fn get_presentation_msg(handle: u32) -> VcxResult<String> {
    PROOF_MAP.get(handle, |proof| proof.get_presentation_msg().map_err(|err| err.into()))
}

pub async fn get_thread_id(handle: u32) -> VcxResult<String> {
    PROOF_MAP.get(handle, |proof| proof.get_thread_id().map_err(|err| err.into()))
}

#[cfg(test)]
pub mod tests {
    use serde_json::Value;

    use aries_vcx::agency_client::testing::mocking::HttpClientMockResponse;
    use aries_vcx::utils::constants::{
        PROOF_REJECT_RESPONSE_STR_V2, REQUESTED_ATTRS, REQUESTED_PREDICATES, V3_OBJECT_SERIALIZE_VERSION,
    };
    use aries_vcx::utils::devsetup::SetupMocks;
    use aries_vcx::utils::mockdata::mock_settings::MockBuilder;
    use aries_vcx::utils::mockdata::mockdata_proof;

    use crate::api_lib::api_handle::mediated_connection::tests::build_test_connection_inviter_requested;
    use crate::api_lib::api_handle::proof;
    use crate::aries_vcx::protocols::proof_presentation::verifier::state_machine::VerifierState;

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
    #[cfg(feature = "general_test")]
    async fn test_create_proof_succeeds() {
        let _setup = SetupMocks::init();
        create_default_proof().await;
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
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
    #[cfg(feature = "general_test")]
    async fn test_to_string_succeeds() {
        let _setup = SetupMocks::init();

        let handle = create_default_proof().await;
        let proof_string = to_string(handle).await.unwrap();
        let s: Value = serde_json::from_str(&proof_string).unwrap();
        assert_eq!(s["version"], V3_OBJECT_SERIALIZE_VERSION);
        assert!(s["data"]["verifier_sm"].is_object());
        assert!(!proof_string.is_empty());
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_from_string_succeeds() {
        let _setup = SetupMocks::init();

        let handle = create_default_proof().await;
        let proof_data = to_string(handle).await.unwrap();
        let _hnadle2 = from_string(&proof_data).await.unwrap();
        let proof_data2 = to_string(handle).await.unwrap();
        assert_eq!(proof_data, proof_data2);
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_release_proof() {
        let _setup = SetupMocks::init();

        let handle = create_default_proof().await;
        assert!(release(handle).is_ok());
        assert!(!is_valid_handle(handle).await);
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_send_proof_request() {
        let _setup = SetupMocks::init();

        let handle_conn = build_test_connection_inviter_requested().await;

        let handle_proof = create_default_proof().await;
        assert_eq!(
            send_proof_request(handle_proof, handle_conn).await.unwrap(),
            error::SUCCESS.code_num
        );
        assert_eq!(
            get_state(handle_proof).await.unwrap(),
            VerifierState::PresentationRequestSent as u32
        );
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_get_proof_fails_with_no_proof() {
        let _setup = SetupMocks::init();

        let handle = create_default_proof().await;
        assert!(is_valid_handle(handle).await);
        assert!(get_presentation_msg(handle).await.is_err())
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_proof_update_state_v2() {
        let _setup = SetupMocks::init();
        let _mock_builder = MockBuilder::init().set_mock_result_for_validate_indy_proof(Ok(true));

        let handle_conn = build_test_connection_inviter_requested().await;
        let handle_proof = create_default_proof().await;

        send_proof_request(handle_proof, handle_conn).await.unwrap();
        assert_eq!(
            get_state(handle_proof).await.unwrap(),
            VerifierState::PresentationRequestSent as u32
        );

        mediated_connection::release(handle_conn).unwrap();
        let handle_conn = build_test_connection_inviter_requested().await;

        update_state(
            handle_proof,
            Some(mockdata_proof::ARIES_PROOF_PRESENTATION),
            handle_conn,
        )
        .await
        .unwrap();

        assert_eq!(get_state(handle_proof).await.unwrap(), VerifierState::Finished as u32);
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_update_state() {
        let _setup = SetupMocks::init();
        let _mock_builder = MockBuilder::init().set_mock_result_for_validate_indy_proof(Ok(true));

        let handle_conn = build_test_connection_inviter_requested().await;
        let handle_proof = create_default_proof().await;

        send_proof_request(handle_proof, handle_conn).await.unwrap();
        assert_eq!(
            get_state(handle_proof).await.unwrap(),
            VerifierState::PresentationRequestSent as u32
        );

        update_state(
            handle_proof,
            Some(mockdata_proof::ARIES_PROOF_PRESENTATION),
            handle_conn,
        )
        .await
        .unwrap();
        assert_eq!(get_state(handle_proof).await.unwrap(), VerifierState::Finished as u32);
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_proof_validation_with_predicate() {
        let _setup = SetupMocks::init();
        let _mock_builder = MockBuilder::init().set_mock_result_for_validate_indy_proof(Ok(true));

        let handle_conn = build_test_connection_inviter_requested().await;
        let handle_proof = create_default_proof().await;

        send_proof_request(handle_proof, handle_conn).await.unwrap();
        assert_eq!(
            get_state(handle_proof).await.unwrap(),
            VerifierState::PresentationRequestSent as u32
        );

        update_state(
            handle_proof,
            Some(mockdata_proof::ARIES_PROOF_PRESENTATION),
            handle_conn,
        )
        .await
        .unwrap();
        assert_eq!(get_state(handle_proof).await.unwrap(), VerifierState::Finished as u32);
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_update_state_with_reject_message() {
        let _setup = SetupMocks::init();

        let handle_conn = build_test_connection_inviter_requested().await;
        let handle_proof = create_default_proof().await;

        send_proof_request(handle_proof, handle_conn).await.unwrap();

        update_state(handle_proof, Some(PROOF_REJECT_RESPONSE_STR_V2), handle_conn)
            .await
            .unwrap();
        assert_eq!(get_state(handle_proof).await.unwrap(), VerifierState::Failed as u32);
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_send_presentation_request() {
        let _setup = SetupMocks::init();

        let handle_conn = build_test_connection_inviter_requested().await;
        let handle_proof = create_default_proof().await;

        send_proof_request(handle_proof, handle_conn).await.unwrap();
        assert_eq!(
            get_state(handle_proof).await.unwrap(),
            VerifierState::PresentationRequestSent as u32
        );
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_get_proof() {
        let _setup = SetupMocks::init();
        let _mock_builder = MockBuilder::init().set_mock_result_for_validate_indy_proof(Ok(true));

        let handle_conn = build_test_connection_inviter_requested().await;
        let handle_proof = create_default_proof().await;

        send_proof_request(handle_proof, handle_conn).await.unwrap();
        assert_eq!(
            get_state(handle_proof).await.unwrap(),
            VerifierState::PresentationRequestSent as u32
        );

        update_state(
            handle_proof,
            Some(mockdata_proof::ARIES_PROOF_PRESENTATION),
            handle_conn,
        )
        .await
        .unwrap();
        assert_eq!(get_state(handle_proof).await.unwrap(), VerifierState::Finished as u32);

        let proof_str = get_presentation_msg(handle_proof).await.unwrap();
        assert_eq!(
            proof_str,
            mockdata_proof::ARIES_PROOF_PRESENTATION
                .replace("\n", "")
                .replace(" ", "")
        );
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
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
        let h4 = create_proof(
            "1".to_string(),
            REQUESTED_ATTRS.to_owned(),
            REQUESTED_PREDICATES.to_owned(),
            r#"{"support_revocation":false}"#.to_string(),
            "Optional".to_owned(),
        )
        .await
        .unwrap();
        let h5 = create_proof(
            "1".to_string(),
            REQUESTED_ATTRS.to_owned(),
            REQUESTED_PREDICATES.to_owned(),
            r#"{"support_revocation":false}"#.to_string(),
            "Optional".to_owned(),
        )
        .await
        .unwrap();
        release_all();
        assert_eq!(release(h1).unwrap_err().kind(), VcxErrorKind::InvalidProofHandle);
        assert_eq!(release(h2).unwrap_err().kind(), VcxErrorKind::InvalidProofHandle);
        assert_eq!(release(h3).unwrap_err().kind(), VcxErrorKind::InvalidProofHandle);
        assert_eq!(release(h4).unwrap_err().kind(), VcxErrorKind::InvalidProofHandle);
        assert_eq!(release(h5).unwrap_err().kind(), VcxErrorKind::InvalidProofHandle);
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_send_proof_request_can_be_retried() {
        let _setup = SetupMocks::init();

        let handle_conn = build_test_connection_inviter_requested().await;
        let handle_proof = create_default_proof().await;

        let _request = get_presentation_request_msg(handle_proof).await.unwrap();
        assert_eq!(get_state(handle_proof).await.unwrap(), 1);

        HttpClientMockResponse::set_next_response(aries_vcx::agency_client::error::AgencyClientResult::Err(
            aries_vcx::agency_client::error::AgencyClientError::from_msg(
                aries_vcx::agency_client::error::AgencyClientErrorKind::IOError,
                "Sending message timeout.",
            ),
        ));
        assert_eq!(
            send_proof_request(handle_proof, handle_conn).await.unwrap_err().kind(),
            VcxErrorKind::IOError
        );
        assert_eq!(get_state(handle_proof).await.unwrap(), 1);

        // Retry sending proof request
        assert_eq!(send_proof_request(handle_proof, handle_conn).await.unwrap(), 0);
        assert_eq!(
            get_state(handle_proof).await.unwrap(),
            VerifierState::PresentationRequestSent as u32
        );
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_proof_accepted() {
        let _setup = SetupMocks::init();
        let _mock_builder = MockBuilder::init().set_mock_result_for_validate_indy_proof(Ok(true));

        let handle_conn = build_test_connection_inviter_requested().await;
        let handle_proof = create_default_proof().await;

        let _request = get_presentation_request_msg(handle_proof).await.unwrap();
        send_proof_request(handle_proof, handle_conn).await.unwrap();
        update_state(
            handle_proof,
            Some(mockdata_proof::ARIES_PROOF_PRESENTATION),
            handle_conn,
        )
        .await
        .unwrap();
        assert_eq!(
            proof::get_state(handle_proof).await.unwrap(),
            VerifierState::Finished as u32
        );
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_proof_errors() {
        let _setup = SetupMocks::init();

        let handle_conn = build_test_connection_inviter_requested().await;
        let handle_proof = create_default_proof().await;

        let bad_handle = 100000;
        let empty = r#""#;

        assert_eq!(
            send_proof_request(bad_handle, handle_conn).await.unwrap_err().kind(),
            VcxErrorKind::InvalidHandle
        );
        assert_eq!(get_proof_state(handle_proof).await.unwrap(), 0);
        assert_eq!(
            create_proof(
                "my source id".to_string(),
                empty.to_string(),
                "{}".to_string(),
                r#"{"support_revocation":false}"#.to_string(),
                "my name".to_string()
            )
            .await
            .unwrap_err()
            .kind(),
            VcxErrorKind::InvalidJson
        );
        assert_eq!(
            to_string(bad_handle).await.unwrap_err().kind(),
            VcxErrorKind::InvalidHandle
        );
        assert_eq!(
            get_source_id(bad_handle).unwrap_err().kind(),
            VcxErrorKind::InvalidHandle
        );
        assert_eq!(from_string(empty).await.unwrap_err().kind(), VcxErrorKind::InvalidJson);
    }
}
