use std::{collections::HashMap, sync::RwLock};

use actix_web::{get, post, web, Responder};
use anoncreds_types::data_types::messages::pres_request::{
    AttributeInfo, NonRevokedInterval, PredicateInfo, PresentationRequestPayload,
};
use aries_vcx_agent::aries_vcx::{
    aries_vcx_core::anoncreds::base_anoncreds::BaseAnonCreds,
    handlers::util::PresentationProposalData,
    messages::msg_fields::protocols::present_proof::v1::propose::{Predicate, PresentationAttr},
    protocols::proof_presentation::{
        prover::state_machine::ProverState,
        verifier::{
            state_machine::VerifierState, verification_status::PresentationVerificationStatus,
        },
    },
};

use crate::{
    controllers::AathRequest,
    error::{HarnessError, HarnessErrorType, HarnessResult},
    soft_assert_eq, HarnessAgent, State,
};

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct PresentationRequestWrapper {
    connection_id: String,
    presentation_request: PresentationRequest,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct PresentationProposalWrapper {
    connection_id: String,
    presentation_proposal: PresentationProposal,
}

// TODO: Remove these structs
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
pub struct PresentationRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    pub proof_request: ProofRequestDataWrapper,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct ProofRequestDataWrapper {
    pub data: ProofRequestData,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct PresentationProposal {
    comment: String,
    attributes: Vec<PresentationAttr>,
    predicates: Vec<Predicate>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct ProofRequestData {
    pub requested_attributes: Option<HashMap<String, AttributeInfo>>,
    pub requested_predicates: Option<HashMap<String, PredicateInfo>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub non_revoked: Option<NonRevokedInterval>,
}

fn to_backchannel_state_prover(state: ProverState) -> State {
    match state {
        ProverState::Initial => State::Initial,
        ProverState::PresentationRequestReceived => State::RequestReceived,
        ProverState::PresentationProposalSent => State::ProposalSent,
        ProverState::PresentationSent => State::PresentationSent,
        ProverState::PresentationPreparationFailed
        | ProverState::Finished
        | ProverState::Failed => State::Done,
        _ => State::Unknown,
    }
}

fn to_backchannel_state_verifier(state: VerifierState) -> State {
    match state {
        VerifierState::Initial => State::Initial,
        VerifierState::PresentationRequestSet => State::RequestSet,
        VerifierState::PresentationProposalReceived => State::ProposalReceived,
        VerifierState::PresentationRequestSent => State::RequestSent,
        VerifierState::Finished | VerifierState::Failed => State::Done,
    }
}

impl HarnessAgent {
    pub async fn send_proof_request(
        &self,
        presentation_request: &PresentationRequestWrapper,
    ) -> HarnessResult<String> {
        let req_data = presentation_request
            .presentation_request
            .proof_request
            .data
            .clone();
        let nonce = self.aries_agent.anoncreds().generate_nonce().await?;
        let request = PresentationRequestPayload::builder()
            .requested_attributes(req_data.requested_attributes.unwrap_or_default())
            .requested_predicates(req_data.requested_predicates.unwrap_or_default())
            .non_revoked(req_data.non_revoked)
            .nonce(nonce)
            .name("test proof".to_string())
            .build();
        let id = self
            .aries_agent
            .verifier()
            .send_proof_request(&presentation_request.connection_id, request.into(), None)
            .await?;
        let state = self.aries_agent.verifier().get_state(&id)?;
        Ok(json!({ "state": to_backchannel_state_verifier(state), "thread_id": id }).to_string())
    }

    pub async fn send_proof_proposal(
        &self,
        presentation_proposal: &PresentationProposalWrapper,
    ) -> HarnessResult<String> {
        let mut proposal_data = PresentationProposalData::default();
        for attr in presentation_proposal
            .presentation_proposal
            .attributes
            .clone()
            .into_iter()
        {
            proposal_data.attributes.push(attr.clone());
        }
        let id = self
            .aries_agent
            .prover()
            .send_proof_proposal(&presentation_proposal.connection_id, proposal_data)
            .await?;
        let state = self.aries_agent.prover().get_state(&id)?;
        Ok(json!({ "state": to_backchannel_state_prover(state), "thread_id": id }).to_string())
    }

    pub async fn send_presentation(&self, id: &str) -> HarnessResult<String> {
        let state = self.aries_agent.prover().get_state(id)?;
        soft_assert_eq!(state, ProverState::PresentationRequestReceived);
        let tails_dir = if self.aries_agent.prover().is_secondary_proof_requested(id)? {
            Some(
                std::env::current_dir()
                    .unwrap()
                    .join("resource")
                    .join("tails")
                    .to_str()
                    .unwrap()
                    .to_string(),
            )
        } else {
            None
        };
        self.aries_agent
            .prover()
            .send_proof_prentation(id, tails_dir.as_deref())
            .await?;
        let state = self.aries_agent.prover().get_state(id)?;
        Ok(json!({"state": to_backchannel_state_prover(state), "thread_id": id}).to_string())
    }

    pub async fn verify_presentation(&self, id: &str) -> HarnessResult<String> {
        let verified = self.aries_agent.verifier().get_presentation_status(id)?
            == PresentationVerificationStatus::Valid;
        let state = self.aries_agent.verifier().get_state(id)?;
        Ok(
            json!({ "state": to_backchannel_state_verifier(state), "verified": verified })
                .to_string(),
        )
    }

    pub async fn get_proof_state(&self, id: &str) -> HarnessResult<String> {
        let state = if self.aries_agent.verifier().exists_by_id(id) {
            to_backchannel_state_verifier(self.aries_agent.verifier().get_state(id)?)
        } else if self.aries_agent.prover().exists_by_id(id) {
            to_backchannel_state_prover(self.aries_agent.prover().get_state(id)?)
        } else {
            return Err(HarnessError::from_kind(HarnessErrorType::NotFoundError));
        };
        Ok(json!({ "state": state }).to_string())
    }
}

#[post("/send-request")]
pub async fn send_proof_request(
    req: web::Json<AathRequest<PresentationRequestWrapper>>,
    agent: web::Data<RwLock<HarnessAgent>>,
) -> impl Responder {
    agent.read().unwrap().send_proof_request(&req.data).await
}

#[post("/send-proposal")]
pub async fn send_proof_proposal(
    req: web::Json<AathRequest<PresentationProposalWrapper>>,
    agent: web::Data<RwLock<HarnessAgent>>,
) -> impl Responder {
    agent.read().unwrap().send_proof_proposal(&req.data).await
}

#[post("/send-presentation")]
pub async fn send_presentation(
    req: web::Json<AathRequest<serde_json::Value>>,
    agent: web::Data<RwLock<HarnessAgent>>,
) -> impl Responder {
    agent.read().unwrap().send_presentation(&req.id).await
}

#[post("/verify-presentation")]
pub async fn verify_presentation(
    req: web::Json<AathRequest<serde_json::Value>>,
    agent: web::Data<RwLock<HarnessAgent>>,
) -> impl Responder {
    agent.read().unwrap().verify_presentation(&req.id).await
}

#[get("/{proof_id}")]
pub async fn get_proof_state(
    agent: web::Data<RwLock<HarnessAgent>>,
    path: web::Path<String>,
) -> impl Responder {
    agent
        .read()
        .unwrap()
        .get_proof_state(&path.into_inner())
        .await
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/command/proof")
            .service(send_proof_request)
            .service(send_proof_proposal)
            .service(send_presentation)
            .service(verify_presentation)
            .service(get_proof_state),
    );
}
