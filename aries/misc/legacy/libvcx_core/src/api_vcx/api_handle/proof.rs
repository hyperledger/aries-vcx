use std::collections::HashMap;

use anoncreds_types::data_types::messages::{
    nonce::Nonce, pres_request::PresentationRequestPayload,
};
use aries_vcx::{
    handlers::{
        proof_presentation::verifier::Verifier,
        util::{matches_opt_thread_id, matches_thread_id},
    },
    messages::{
        msg_fields::protocols::present_proof::{v1::PresentProofV1, PresentProof},
        AriesMessage,
    },
    protocols::{
        proof_presentation::verifier::{
            state_machine::VerifierState, verification_status::PresentationVerificationStatus,
        },
        SendClosure,
    },
};
use serde_json;

use crate::{
    api_vcx::{
        api_global::profile::{get_main_anoncreds, get_main_ledger_read, get_main_wallet},
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
use crate::api_vcx::api_handle::ToU32;

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
    let requested_attrs = serde_json::from_str(&requested_attrs)?;
    let requested_predicates = serde_json::from_str(&requested_predicates)?;
    let revocation_details = serde_json::from_str(&revocation_details)?;
    let presentation_request = PresentationRequestPayload::builder()
        .name(name)
        .requested_attributes(requested_attrs)
        .requested_predicates(requested_predicates)
        .non_revoked(revocation_details)
        .nonce(Nonce::new()?)
        .version("1.0".to_string())
        .build();
    let verifier = Verifier::create_from_request(source_id, &presentation_request.into())?;
    PROOF_MAP.add(verifier)
}

pub fn is_valid_handle(handle: u32) -> bool {
    PROOF_MAP.has_handle(handle)
}

pub fn verifier_find_message_to_handle(
    sm: &Verifier,
    messages: HashMap<String, AriesMessage>,
) -> Option<(String, AriesMessage)> {
    trace!(
        "verifier_find_message_to_handle >>> messages: {:?}",
        messages
    );
    for (uid, message) in messages {
        match sm.get_state() {
            VerifierState::Initial => match &message {
                AriesMessage::PresentProof(PresentProof::V1(
                    PresentProofV1::ProposePresentation(_),
                )) => {
                    return Some((uid, message));
                }
                AriesMessage::PresentProof(PresentProof::V1(
                    PresentProofV1::RequestPresentation(_),
                )) => {
                    return Some((uid, message));
                }
                _ => {}
            },
            VerifierState::PresentationRequestSent => match &message {
                AriesMessage::PresentProof(PresentProof::V1(PresentProofV1::Presentation(
                    presentation,
                ))) => {
                    if matches_thread_id!(presentation, sm.get_thread_id().unwrap().as_str()) {
                        return Some((uid, message));
                    }
                }
                AriesMessage::PresentProof(PresentProof::V1(
                    PresentProofV1::ProposePresentation(proposal),
                )) => {
                    if matches_opt_thread_id!(proposal, sm.get_thread_id().unwrap().as_str()) {
                        return Some((uid, message));
                    }
                }
                AriesMessage::ReportProblem(problem_report) => {
                    if matches_opt_thread_id!(problem_report, sm.get_thread_id().unwrap().as_str())
                    {
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
    let mut proof = PROOF_MAP.get_cloned(handle)?;
    trace!(
        "proof::update_state >>> handle: {}, message: {:?}, connection_handle: {}",
        handle,
        message,
        connection_handle
    );
    if !proof.progressable_by_message() {
        return Ok(proof.get_state().to_u32());
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
                get_main_ledger_read()?.as_ref(),
                get_main_anoncreds()?.as_ref(),
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
                    get_main_ledger_read()?.as_ref(),
                    get_main_anoncreds()?.as_ref(),
                    message,
                )
                .await?
            {
                send_message(connection_handle, message).await?;
            }
            mediated_connection::update_message_status(connection_handle, &uid).await?;
        };
    }
    let state: u32 = proof.get_state().to_u32();
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
        return Ok(proof.get_state().to_u32());
    }

    let con = connection::get_cloned_generic_connection(&connection_handle)?;
    let wallet = get_main_wallet()?;

    let send_message: SendClosure = Box::new(|msg: AriesMessage| {
        Box::pin(async move { con.send_message(wallet.as_ref(), &msg, &HttpClient).await })
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
            get_main_ledger_read()?.as_ref(),
            get_main_anoncreds()?.as_ref(),
            message,
        )
        .await?
    {
        send_message(message).await?;
    }

    let state: u32 = proof.get_state().to_u32();
    PROOF_MAP.insert(handle, proof)?;
    Ok(state)
}

pub fn get_state(handle: u32) -> LibvcxResult<u32> {
    PROOF_MAP.get(handle, |proof| Ok(proof.get_state().to_u32()))
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
        Box::pin(async move { con.send_message(wallet.as_ref(), &msg, &HttpClient).await })
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
