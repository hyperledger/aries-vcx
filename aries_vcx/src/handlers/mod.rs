use crate::handlers::connection::connection::ConnectionState;
use crate::handlers::connection::invitee::state_machine::InviteeState;
use crate::handlers::connection::inviter::state_machine::InviterState;
use crate::handlers::issuance::holder::holder::HolderState;
use crate::handlers::issuance::issuer::issuer::IssuerState;
use crate::handlers::proof_presentation::prover::prover::ProverState;
use crate::handlers::proof_presentation::verifier::verifier::VerifierState;

pub mod connection;
pub mod issuance;
pub mod proof_presentation;
pub mod out_of_band;

impl From<ConnectionState> for u32 {
    fn from(state: ConnectionState) -> u32 {
        match state {
            ConnectionState::Inviter(inviter_state) => {
                match inviter_state {
                    InviterState::Null => 0,
                    InviterState::Invited => 1,
                    InviterState::Requested => 2,
                    InviterState::Responded => 3,
                    InviterState::Completed => 4,
                }
            }
            ConnectionState::Invitee(invitee_state) => {
                match invitee_state {
                    InviteeState::Null => 0,
                    InviteeState::Invited => 1,
                    InviteeState::Requested => 2,
                    InviteeState::Responded => 3,
                    InviteeState::Completed => 4,
                }
            }
        }
    }
}

impl From<HolderState> for u32 {
    fn from(state: HolderState) -> u32 {
        match state {
            HolderState::Initial => 0,
            HolderState::ProposalSent => 1,
            HolderState::OfferReceived => 2,
            HolderState::RequestSent => 3,
            HolderState::Finished => 4,
            HolderState::Failed => 5
        }
    }
}

impl From<IssuerState> for u32 {
    fn from(state: IssuerState) -> u32 {
        match state {
            IssuerState::Initial => 0,
            IssuerState::ProposalReceived => 1,
            IssuerState::OfferSent => 2,
            IssuerState::RequestReceived => 3,
            IssuerState::CredentialSent => 4,
            IssuerState::Finished => 5,
            IssuerState::Failed => 6
        }
    }
}

impl From<ProverState> for u32 {
    fn from(state: ProverState) -> u32 {
        match state {
            ProverState::Initial => 1,
            ProverState::PresentationRequestReceived => 1,
            ProverState::PresentationPrepared => 2,
            ProverState::PresentationPreparationFailed => 3,
            ProverState::PresentationSent => 4,
            ProverState::Finished => 5,
            ProverState::Failed => 6
        }
    }
}

impl From<VerifierState> for u32 {
    fn from(state: VerifierState) -> u32 {
        match state {
            VerifierState::Initial => 0,
            VerifierState::PresentationRequestSent => 1,
            VerifierState::Finished => 2,
            VerifierState::Failed => 3
        }
    }
}
