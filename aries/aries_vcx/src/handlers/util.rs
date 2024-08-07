use anoncreds_types::data_types::identifiers::cred_def_id::CredentialDefinitionId;
use messages::{
    msg_fields::protocols::{
        connection::{invitation::Invitation, Connection},
        coordinate_mediation::CoordinateMediation,
        cred_issuance::{v1::CredentialIssuanceV1, v2::CredentialIssuanceV2, CredentialIssuance},
        did_exchange::{v1_0::DidExchangeV1_0, v1_1::DidExchangeV1_1, DidExchange},
        discover_features::DiscoverFeatures,
        notification::Notification,
        out_of_band::{invitation::Invitation as OobInvitation, OutOfBand},
        pickup::Pickup,
        present_proof::{
            v1::{
                propose::{Predicate, PresentationAttr},
                PresentProofV1,
            },
            v2::PresentProofV2,
            PresentProof,
        },
        report_problem::ProblemReport,
        revocation::Revocation,
        trust_ping::TrustPing,
    },
    AriesMessage,
};
use strum_macros::{AsRefStr, EnumString};

use crate::errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult};

#[macro_export]
macro_rules! matches_thread_id {
    ($msg:expr, $id:expr) => {
        $msg.decorators.thread.thid == $id || $msg.decorators.thread.pthid.as_deref() == Some($id)
    };
}

#[macro_export]
macro_rules! matches_opt_thread_id {
    ($msg:expr, $id:expr) => {
        match $msg.decorators.thread.as_ref() {
            Some(t) => t.thid == $id || t.pthid.as_deref() == Some($id),
            None => true,
        }
    };
}

#[rustfmt::skip] // This macro results in some false positives and formatting makes it harder to read
macro_rules! get_attach_as_string {
    ($attachments:expr) => {{
        let __attach = $attachments.first().as_ref().map(|a| &a.data.content);
        let err_fn = |attach: Option<&messages::decorators::attachment::Attachment>| {
            Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::SerializationError,
                format!("Attachment is not base 64 encoded JSON: {:?}", attach),
            ))
        };

        let Some(messages::decorators::attachment::AttachmentType::Base64(encoded_attach)) = __attach else { return err_fn($attachments.first()); };
        let Ok(bytes) = base64::engine::Engine::decode(&base64::engine::general_purpose::STANDARD, &encoded_attach) else { return err_fn($attachments.first()); };
        let Ok(attach_string) = String::from_utf8(bytes) else { return err_fn($attachments.first()); };

        attach_string
    }};
}

macro_rules! make_attach_from_str {
    ($str_attach:expr, $id:expr) => {{
        let attach_type = messages::decorators::attachment::AttachmentType::Base64(
            base64::engine::Engine::encode(&base64::engine::general_purpose::STANDARD, $str_attach),
        );
        let attach_data = messages::decorators::attachment::AttachmentData::builder()
            .content(attach_type)
            .build();
        let mut attach = messages::decorators::attachment::Attachment::builder()
            .data(attach_data)
            .build();
        attach.id = Some($id);
        attach.mime_type = Some(shared::maybe_known::MaybeKnown::Known(
            messages::misc::MimeType::Json,
        ));
        attach
    }};
}

pub(crate) use get_attach_as_string;
pub(crate) use make_attach_from_str;
pub use matches_opt_thread_id;
pub use matches_thread_id;

pub fn verify_thread_id(thread_id: &str, message: &AriesMessage) -> VcxResult<()> {
    let is_match = match message {
        AriesMessage::BasicMessage(msg) => matches_opt_thread_id!(msg, thread_id),
        AriesMessage::Connection(Connection::Invitation(msg)) => msg.id == thread_id,
        AriesMessage::Connection(Connection::ProblemReport(msg)) => {
            matches_thread_id!(msg, thread_id)
        }
        AriesMessage::Connection(Connection::Request(msg)) => {
            matches_opt_thread_id!(msg, thread_id)
        }
        AriesMessage::Connection(Connection::Response(msg)) => matches_thread_id!(msg, thread_id),
        AriesMessage::CredentialIssuance(CredentialIssuance::V1(CredentialIssuanceV1::Ack(
            msg,
        ))) => {
            matches_thread_id!(msg, thread_id)
        }
        AriesMessage::CredentialIssuance(CredentialIssuance::V1(
            CredentialIssuanceV1::IssueCredential(msg),
        )) => {
            matches_thread_id!(msg, thread_id)
        }
        AriesMessage::CredentialIssuance(CredentialIssuance::V1(
            CredentialIssuanceV1::OfferCredential(msg),
        )) => {
            matches_opt_thread_id!(msg, thread_id)
        }
        AriesMessage::CredentialIssuance(CredentialIssuance::V1(
            CredentialIssuanceV1::ProposeCredential(msg),
        )) => {
            matches_opt_thread_id!(msg, thread_id)
        }
        AriesMessage::CredentialIssuance(CredentialIssuance::V1(
            CredentialIssuanceV1::RequestCredential(msg),
        )) => {
            matches_opt_thread_id!(msg, thread_id)
        }
        AriesMessage::CredentialIssuance(CredentialIssuance::V1(
            CredentialIssuanceV1::ProblemReport(msg),
        )) => {
            matches_opt_thread_id!(msg, thread_id)
        }
        AriesMessage::CredentialIssuance(CredentialIssuance::V2(CredentialIssuanceV2::Ack(
            msg,
        ))) => {
            matches_thread_id!(msg, thread_id)
        }
        AriesMessage::CredentialIssuance(CredentialIssuance::V2(
            CredentialIssuanceV2::IssueCredential(msg),
        )) => {
            matches_thread_id!(msg, thread_id)
        }
        AriesMessage::CredentialIssuance(CredentialIssuance::V2(
            CredentialIssuanceV2::OfferCredential(msg),
        )) => {
            matches_opt_thread_id!(msg, thread_id)
        }
        AriesMessage::CredentialIssuance(CredentialIssuance::V2(
            CredentialIssuanceV2::ProposeCredential(msg),
        )) => {
            matches_opt_thread_id!(msg, thread_id)
        }
        AriesMessage::CredentialIssuance(CredentialIssuance::V2(
            CredentialIssuanceV2::RequestCredential(msg),
        )) => {
            matches_opt_thread_id!(msg, thread_id)
        }
        AriesMessage::CredentialIssuance(CredentialIssuance::V2(
            CredentialIssuanceV2::ProblemReport(msg),
        )) => {
            matches_opt_thread_id!(msg, thread_id)
        }
        AriesMessage::DiscoverFeatures(DiscoverFeatures::Query(msg)) => msg.id == thread_id,
        AriesMessage::DiscoverFeatures(DiscoverFeatures::Disclose(msg)) => {
            matches_thread_id!(msg, thread_id)
        }
        AriesMessage::Notification(Notification::Ack(msg)) => matches_thread_id!(msg, thread_id),
        AriesMessage::Notification(Notification::ProblemReport(msg)) => {
            matches_opt_thread_id!(msg, thread_id)
        }
        AriesMessage::OutOfBand(OutOfBand::Invitation(msg)) => msg.id == thread_id,
        AriesMessage::OutOfBand(OutOfBand::HandshakeReuse(msg)) => {
            matches_thread_id!(msg, thread_id)
        }
        AriesMessage::OutOfBand(OutOfBand::HandshakeReuseAccepted(msg)) => {
            matches_thread_id!(msg, thread_id)
        }
        AriesMessage::PresentProof(PresentProof::V1(PresentProofV1::Ack(msg))) => {
            matches_thread_id!(msg, thread_id)
        }
        AriesMessage::PresentProof(PresentProof::V1(PresentProofV1::Presentation(msg))) => {
            matches_thread_id!(msg, thread_id)
        }
        AriesMessage::PresentProof(PresentProof::V1(PresentProofV1::ProposePresentation(msg))) => {
            matches_opt_thread_id!(msg, thread_id)
        }
        AriesMessage::PresentProof(PresentProof::V1(PresentProofV1::RequestPresentation(msg))) => {
            matches_opt_thread_id!(msg, thread_id)
        }
        AriesMessage::PresentProof(PresentProof::V1(PresentProofV1::ProblemReport(msg))) => {
            matches_opt_thread_id!(msg, thread_id)
        }
        AriesMessage::PresentProof(PresentProof::V2(PresentProofV2::Ack(msg))) => {
            matches_thread_id!(msg, thread_id)
        }
        AriesMessage::PresentProof(PresentProof::V2(PresentProofV2::Presentation(msg))) => {
            matches_thread_id!(msg, thread_id)
        }
        AriesMessage::PresentProof(PresentProof::V2(PresentProofV2::ProposePresentation(msg))) => {
            matches_opt_thread_id!(msg, thread_id)
        }
        AriesMessage::PresentProof(PresentProof::V2(PresentProofV2::RequestPresentation(msg))) => {
            matches_opt_thread_id!(msg, thread_id)
        }
        AriesMessage::PresentProof(PresentProof::V2(PresentProofV2::ProblemReport(msg))) => {
            matches_opt_thread_id!(msg, thread_id)
        }
        AriesMessage::ReportProblem(msg) => matches_opt_thread_id!(msg, thread_id),
        AriesMessage::Revocation(Revocation::Revoke(msg)) => matches_opt_thread_id!(msg, thread_id),
        AriesMessage::Revocation(Revocation::Ack(msg)) => matches_thread_id!(msg, thread_id),
        AriesMessage::Routing(msg) => msg.id == thread_id,
        AriesMessage::TrustPing(TrustPing::Ping(msg)) => matches_opt_thread_id!(msg, thread_id),
        AriesMessage::TrustPing(TrustPing::PingResponse(msg)) => matches_thread_id!(msg, thread_id),
        AriesMessage::Pickup(Pickup::Status(msg)) => matches_opt_thread_id!(msg, thread_id),
        AriesMessage::Pickup(Pickup::StatusRequest(msg)) => matches_opt_thread_id!(msg, thread_id),
        AriesMessage::Pickup(Pickup::Delivery(msg)) => matches_opt_thread_id!(msg, thread_id),
        AriesMessage::Pickup(Pickup::DeliveryRequest(msg)) => {
            matches_opt_thread_id!(msg, thread_id)
        }

        AriesMessage::Pickup(Pickup::MessagesReceived(msg)) => {
            matches_opt_thread_id!(msg, thread_id)
        }
        AriesMessage::Pickup(Pickup::LiveDeliveryChange(msg)) => {
            matches_opt_thread_id!(msg, thread_id)
        }
        AriesMessage::CoordinateMediation(CoordinateMediation::MediateRequest(msg)) => {
            msg.id == thread_id
        }
        AriesMessage::CoordinateMediation(CoordinateMediation::MediateDeny(msg)) => {
            matches_opt_thread_id!(msg, thread_id)
        }
        AriesMessage::CoordinateMediation(CoordinateMediation::MediateGrant(msg)) => {
            matches_opt_thread_id!(msg, thread_id)
        }
        AriesMessage::CoordinateMediation(CoordinateMediation::KeylistUpdate(msg)) => {
            msg.id == thread_id
        }
        AriesMessage::CoordinateMediation(CoordinateMediation::KeylistUpdateResponse(msg)) => {
            matches_opt_thread_id!(msg, thread_id)
        }
        AriesMessage::CoordinateMediation(CoordinateMediation::KeylistQuery(msg)) => {
            msg.id == thread_id
        }
        AriesMessage::CoordinateMediation(CoordinateMediation::Keylist(msg)) => {
            matches_opt_thread_id!(msg, thread_id)
        }
        AriesMessage::DidExchange(DidExchange::V1_0(DidExchangeV1_0::Request(msg)))
        | AriesMessage::DidExchange(DidExchange::V1_1(DidExchangeV1_1::Request(msg))) => {
            matches_opt_thread_id!(msg, thread_id)
        }
        AriesMessage::DidExchange(DidExchange::V1_0(DidExchangeV1_0::Response(msg))) => {
            matches_thread_id!(msg, thread_id)
        }
        AriesMessage::DidExchange(DidExchange::V1_0(DidExchangeV1_0::Complete(msg)))
        | AriesMessage::DidExchange(DidExchange::V1_1(DidExchangeV1_1::Complete(msg))) => {
            matches_thread_id!(msg, thread_id)
        }
        AriesMessage::DidExchange(DidExchange::V1_0(DidExchangeV1_0::ProblemReport(msg)))
        | AriesMessage::DidExchange(DidExchange::V1_1(DidExchangeV1_1::ProblemReport(msg))) => {
            matches_thread_id!(msg, thread_id)
        }
        AriesMessage::DidExchange(DidExchange::V1_1(DidExchangeV1_1::Response(msg))) => {
            matches_thread_id!(msg, thread_id)
        }
    };

    if !is_match {
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

#[derive(Debug, Clone, AsRefStr, EnumString, PartialEq)]
pub enum AttachmentId {
    #[strum(serialize = "libindy-cred-offer-0")]
    CredentialOffer,
    #[strum(serialize = "libindy-cred-request-0")]
    CredentialRequest,
    #[strum(serialize = "libindy-cred-0")]
    Credential,
    #[strum(serialize = "libindy-request-presentation-0")]
    PresentationRequest,
    #[strum(serialize = "libindy-presentation-0")]
    Presentation,
}

/// For retro-fitting the new messages.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum AnyInvitation {
    Con(Invitation),
    Oob(OobInvitation),
}

impl AnyInvitation {
    pub fn id(&self) -> &str {
        match self {
            AnyInvitation::Con(invitation) => &invitation.id,
            AnyInvitation::Oob(invitation) => &invitation.id,
        }
    }
}

// TODO: post-rebase check if this is applicable version, else delete
// impl AnyInvitation {
//     pub fn get_id(&self) -> &str {
//         match self {
//             AnyInvitation::Con(Invitation::Public(msg)) => &msg.id,
//             AnyInvitation::Con(Invitation::Pairwise(msg)) => &msg.id,
//             AnyInvitation::Con(Invitation::PairwiseDID(msg)) => &msg.id,
//             AnyInvitation::Oob(msg) => &msg.id,
//         }
//     }
// }

// todo: this is shared by multiple protocols to express different things - needs to be split
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Status {
    Undefined,
    Success,
    Failed(ProblemReport),
    Declined(ProblemReport),
}

impl Status {
    pub fn code(&self) -> u32 {
        match self {
            Status::Undefined => 0,
            Status::Success => 1,
            Status::Failed(_) => 2,
            Status::Declined(_) => 3,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Default)]
pub struct CredentialData {
    pub schema_id: String,
    pub cred_def_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rev_reg_id: Option<String>,
    pub values: serde_json::Value,
    pub signature: serde_json::Value,
    pub signature_correctness_proof: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rev_reg: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub witness: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct OfferInfo {
    pub credential_json: String,
    pub cred_def_id: CredentialDefinitionId,
    pub rev_reg_id: Option<String>,
    pub tails_file: Option<String>,
}

impl OfferInfo {
    pub fn new(
        credential_json: String,
        cred_def_id: CredentialDefinitionId,
        rev_reg_id: Option<String>,
        tails_file: Option<String>,
    ) -> Self {
        Self {
            credential_json,
            cred_def_id,
            rev_reg_id,
            tails_file,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Default)]
pub struct PresentationProposalData {
    pub attributes: Vec<PresentationAttr>,
    pub predicates: Vec<Predicate>,
    pub comment: Option<String>,
}
