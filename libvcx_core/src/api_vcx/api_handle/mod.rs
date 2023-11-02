use aries_vcx::{
    handlers::mediated_connection::ConnectionState,
    protocols::{
        issuance::{holder::state_machine::HolderState, issuer::state_machine::IssuerState},
        mediated_connection::{
            invitee::state_machine::InviteeState,
            inviter::state_machine::InviterState,
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

impl ToU32 for ConnectionState {
    fn to_u32(&self) -> u32 {
        match self {
            ConnectionState::Inviter(inviter_state) => match inviter_state {
                InviterState::Initial => 0,
                InviterState::Invited => 1,
                InviterState::Requested => 2,
                InviterState::Responded => 3,
                InviterState::Completed => 4,
            },
            ConnectionState::Invitee(invitee_state) => match invitee_state {
                InviteeState::Initial => 0,
                InviteeState::Invited => 1,
                InviteeState::Requested => 2,
                InviteeState::Responded => 3,
                InviteeState::Completed => 4,
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
