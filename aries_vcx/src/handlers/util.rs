use base64::{engine::general_purpose, Engine};
use messages::{
    decorators::attachment::{Attachment, AttachmentType},
    msg_fields::protocols::{
        connection::{invitation::Invitation, Connection},
        cred_issuance::{v1::CredentialIssuanceV1, v2::CredentialIssuanceV2, CredentialIssuance},
        discover_features::DiscoverFeatures,
        notification::Notification,
        out_of_band::{invitation::Invitation as OobInvitation, OutOfBand},
        present_proof::{
            propose::{Predicate, PresentationAttr},
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

macro_rules! get_thread_id_or_message_id {
    ($msg:expr) => {
        $msg.decorators
            .thread
            .as_ref()
            .map_or($msg.id.clone(), |t| t.thid.clone())
    };
}

macro_rules! matches_thread_id {
    ($msg:expr, $id:expr) => {
        $msg.decorators.thread.thid == $id || $msg.decorators.thread.pthid.as_deref() == Some($id)
    };
}

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
        let __attach = $attachments.get(0).as_ref().map(|a| &a.data.content);
        let err_fn = |attach: Option<&messages::decorators::attachment::Attachment>| {
            Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::SerializationError,
                format!("Attachment is not base 64 encoded JSON: {:?}", attach),
            ))
        };

        let Some(messages::decorators::attachment::AttachmentType::Base64(encoded_attach)) = __attach else { return err_fn($attachments.get(0)); };
        let Ok(bytes) = base64::engine::Engine::decode(&base64::engine::general_purpose::STANDARD, &encoded_attach) else { return err_fn($attachments.get(0)); };
        let Ok(attach_string) = String::from_utf8(bytes) else { return err_fn($attachments.get(0)); };

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
        attach.mime_type = Some(messages::misc::MimeType::Json);
        attach
    }};
}

pub(crate) use get_attach_as_string;
pub(crate) use get_thread_id_or_message_id;
pub(crate) use make_attach_from_str;
pub(crate) use matches_opt_thread_id;
pub(crate) use matches_thread_id;

/// Extract/decode the inner data of an [Attachment] as a [Vec<u8>], regardless of whether the inner
/// data is encoded as base64 or JSON.
pub fn extract_attachment_data(attachment: &Attachment) -> VcxResult<Vec<u8>> {
    let data = match &attachment.data.content {
        AttachmentType::Base64(encoded_attach) => general_purpose::URL_SAFE
            .decode(encoded_attach)
            .map_err(|_| {
                AriesVcxError::from_msg(
                    AriesVcxErrorKind::EncodeError,
                    format!("Message attachment is not base64 as expected: {attachment:?}"),
                )
            })?,
        AttachmentType::Json(json_attach) => serde_json::to_vec(json_attach)?,
        _ => {
            return Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidMessageFormat,
                format!("Message attachment is not base64 or JSON: {attachment:?}"),
            ))
        }
    };

    Ok(data)
}

/// Retrieve the first [Attachment] from a list, where the [Attachment] as an `id` matching the
/// supplied id. Returning an error if no attachment is found.
pub fn get_attachment_with_id<'a>(
    attachments: &'a Vec<Attachment>,
    id: &String,
) -> VcxResult<&'a Attachment> {
    attachments
        .iter()
        .find(|attachment| attachment.id.as_ref() == Some(id))
        .ok_or(AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidMessageFormat,
            format!("Message is missing an attachment with the expected ID : {id}."),
        ))
}

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
        AriesMessage::PresentProof(PresentProof::Ack(msg)) => matches_thread_id!(msg, thread_id),
        AriesMessage::PresentProof(PresentProof::Presentation(msg)) => {
            matches_thread_id!(msg, thread_id)
        }
        AriesMessage::PresentProof(PresentProof::ProposePresentation(msg)) => {
            matches_opt_thread_id!(msg, thread_id)
        }
        AriesMessage::PresentProof(PresentProof::RequestPresentation(msg)) => {
            matches_opt_thread_id!(msg, thread_id)
        }
        AriesMessage::PresentProof(PresentProof::ProblemReport(msg)) => {
            matches_opt_thread_id!(msg, thread_id)
        }
        AriesMessage::ReportProblem(msg) => matches_opt_thread_id!(msg, thread_id),
        AriesMessage::Revocation(Revocation::Revoke(msg)) => matches_opt_thread_id!(msg, thread_id),
        AriesMessage::Revocation(Revocation::Ack(msg)) => matches_thread_id!(msg, thread_id),
        AriesMessage::Routing(msg) => msg.id == thread_id,
        AriesMessage::TrustPing(TrustPing::Ping(msg)) => matches_opt_thread_id!(msg, thread_id),
        AriesMessage::TrustPing(TrustPing::PingResponse(msg)) => matches_thread_id!(msg, thread_id),
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
    pub cred_def_id: String,
    pub rev_reg_id: Option<String>,
    pub tails_file: Option<String>,
}

impl OfferInfo {
    pub fn new(
        credential_json: String,
        cred_def_id: String,
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
