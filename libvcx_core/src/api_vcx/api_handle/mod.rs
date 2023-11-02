use aries_vcx::{
    handlers::mediated_connection::MediatedConnectionState,
    protocols::{
        issuance::{holder::state_machine::HolderState, issuer::state_machine::IssuerState},
        mediated_connection::{
            invitee::state_machine::MediatedInviteeState,
            inviter::state_machine::MediatedInviterState,
        },
        proof_presentation::{
            prover::state_machine::ProverState, verifier::state_machine::VerifierState,
        },
    },
};

pub mod connection;
pub mod credential;
pub mod credential_def;
pub mod disclosed_proof;
pub mod issuer_credential;
pub mod mediated_connection;
pub mod object_cache;
pub mod out_of_band;
pub mod proof;
pub mod revocation_registry;
pub mod schema;

trait ToU32 {
    fn to_u32(&self) -> u32;
}

impl ToU32 for MediatedConnectionState {
    fn to_u32(&self) -> u32 {
        match self {
            MediatedConnectionState::Inviter(inviter_state) => match inviter_state {
                MediatedInviterState::Initial => 0,
                MediatedInviterState::Invited => 1,
                MediatedInviterState::Requested => 2,
                MediatedInviterState::Responded => 3,
                MediatedInviterState::Completed => 4,
            },
            MediatedConnectionState::Invitee(invitee_state) => match invitee_state {
                MediatedInviteeState::Initial => 0,
                MediatedInviteeState::Invited => 1,
                MediatedInviteeState::Requested => 2,
                MediatedInviteeState::Responded => 3,
                MediatedInviteeState::Completed => 4,
            },
        }
    }
}

impl ToU32 for HolderState {
    fn to_u32(&self) -> u32 {
        match self {
            HolderState::Initial => 0,
            HolderState::ProposalSet => 1,
            HolderState::OfferReceived => 2,
            HolderState::RequestSet => 3,
            HolderState::Finished => 4,
            HolderState::Failed => 5,
        }
    }
}

impl ToU32 for IssuerState {
    fn to_u32(&self) -> u32 {
        match self {
            IssuerState::Initial => 0,
            IssuerState::ProposalReceived => 1,
            IssuerState::OfferSet => 2,
            IssuerState::RequestReceived => 4,
            IssuerState::CredentialSet => 5,
            IssuerState::Finished => 6,
            IssuerState::Failed => 7,
        }
    }
}

impl ToU32 for ProverState {
    fn to_u32(&self) -> u32 {
        match self {
            ProverState::Initial => 0,
            ProverState::PresentationProposalSent => 1,
            ProverState::PresentationRequestReceived => 2,
            ProverState::PresentationPrepared => 3,
            ProverState::PresentationPreparationFailed => 4,
            ProverState::PresentationSent => 5,
            ProverState::Finished => 6,
            ProverState::Failed => 7,
        }
    }
}

impl ToU32 for VerifierState {
    fn to_u32(&self) -> u32 {
        match self {
            VerifierState::Initial => 0,
            VerifierState::PresentationRequestSet => 1,
            VerifierState::PresentationProposalReceived => 2,
            VerifierState::PresentationRequestSent => 3,
            VerifierState::Finished => 4,
            VerifierState::Failed => 5,
        }
    }
}
