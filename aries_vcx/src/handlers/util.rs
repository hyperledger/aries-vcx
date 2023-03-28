use messages2::{
    msg_fields::protocols::{
        connection::{invitation::Invitation, Connection},
        cred_issuance::CredentialIssuance,
        discover_features::DiscoverFeatures,
        out_of_band::OutOfBand,
        present_proof::PresentProof,
        revocation::Revocation,
        trust_ping::TrustPing,
    },
    AriesMessage,
};

use crate::errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult};

pub fn verify_thread_id(thread_id: &str, message: &AriesMessage) -> VcxResult<()> {
    let msg_thread_id = match message {
        AriesMessage::BasicMessage(msg) => msg
            .decorators
            .thread
            .as_ref()
            .map(|t| t.thid.as_str())
            .unwrap_or(msg.id.as_str()),
        AriesMessage::Connection(Connection::Invitation(Invitation::Public(msg))) => msg.id.as_str(),
        AriesMessage::Connection(Connection::Invitation(Invitation::Pairwise(msg))) => msg.id.as_str(),
        AriesMessage::Connection(Connection::Invitation(Invitation::PairwiseDID(msg))) => msg.id.as_str(),
        AriesMessage::Connection(Connection::ProblemReport(msg)) => msg.decorators.thread.thid.as_str(),
        AriesMessage::Connection(Connection::Request(msg)) => msg
            .decorators
            .thread
            .as_ref()
            .map(|t| t.thid.as_str())
            .unwrap_or(msg.id.as_str()),
        AriesMessage::Connection(Connection::Response(msg)) => msg.decorators.thread.thid.as_str(),
        AriesMessage::CredentialIssuance(CredentialIssuance::Ack(msg)) => msg.decorators.thread.thid.as_str(),
        AriesMessage::CredentialIssuance(CredentialIssuance::IssueCredential(msg)) => {
            msg.decorators.thread.thid.as_str()
        }
        AriesMessage::CredentialIssuance(CredentialIssuance::OfferCredential(msg)) => msg
            .decorators
            .thread
            .as_ref()
            .map(|t| t.thid.as_str())
            .unwrap_or(msg.id.as_str()),
        AriesMessage::CredentialIssuance(CredentialIssuance::ProposeCredential(msg)) => msg
            .decorators
            .thread
            .as_ref()
            .map(|t| t.thid.as_str())
            .unwrap_or(msg.id.as_str()),
        AriesMessage::CredentialIssuance(CredentialIssuance::RequestCredential(msg)) => msg
            .decorators
            .thread
            .as_ref()
            .map(|t| t.thid.as_str())
            .unwrap_or(msg.id.as_str()),
        AriesMessage::DiscoverFeatures(DiscoverFeatures::Query(msg)) => msg.id.as_str(),
        AriesMessage::DiscoverFeatures(DiscoverFeatures::Disclose(msg)) => msg.decorators.thread.thid.as_str(),
        AriesMessage::Notification(msg) => msg.decorators.thread.thid.as_str(),
        AriesMessage::OutOfBand(OutOfBand::Invitation(msg)) => msg.id.as_str(),
        AriesMessage::OutOfBand(OutOfBand::HandshakeReuse(msg)) => msg.decorators.thread.thid.as_str(),
        AriesMessage::OutOfBand(OutOfBand::HandshakeReuseAccepted(msg)) => msg.decorators.thread.thid.as_str(),
        AriesMessage::PresentProof(PresentProof::Ack(msg)) => msg.decorators.thread.thid.as_str(),
        AriesMessage::PresentProof(PresentProof::Presentation(msg)) => msg.decorators.thread.thid.as_str(),
        AriesMessage::PresentProof(PresentProof::ProposePresentation(msg)) => msg
            .decorators
            .thread
            .as_ref()
            .map(|t| t.thid.as_str())
            .unwrap_or(msg.id.as_str()),
        AriesMessage::PresentProof(PresentProof::RequestPresentation(msg)) => msg
            .decorators
            .thread
            .as_ref()
            .map(|t| t.thid.as_str())
            .unwrap_or(msg.id.as_str()),
        AriesMessage::ReportProblem(msg) => msg
            .decorators
            .thread
            .as_ref()
            .map(|t| t.thid.as_str())
            .unwrap_or(msg.id.as_str()),
        AriesMessage::Revocation(Revocation::Revoke(msg)) => msg
            .decorators
            .thread
            .as_ref()
            .map(|t| t.thid.as_str())
            .unwrap_or(msg.id.as_str()),
        AriesMessage::Revocation(Revocation::Ack(msg)) => msg.decorators.thread.thid.as_str(),
        AriesMessage::Routing(msg) => msg.id.as_str(),
        AriesMessage::TrustPing(TrustPing::Ping(msg)) => msg
            .decorators
            .thread
            .as_ref()
            .map(|t| t.thid.as_str())
            .unwrap_or(msg.id.as_str()),
        AriesMessage::TrustPing(TrustPing::PingResponse(msg)) => msg.decorators.thread.thid.as_str(),
    };

    if msg_thread_id != thread_id {
        return Err(AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidJson,
            format!(
                "Cannot handle message {:?}: thread id does not match, expected {:?}",
                message, thread_id
            ),
        ));
    };

    Ok(())
}
