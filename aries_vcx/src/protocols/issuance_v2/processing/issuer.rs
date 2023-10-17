use messages::{
    decorators::{
        please_ack::{AckOn, PleaseAck},
        thread::Thread,
    },
    msg_fields::protocols::cred_issuance::v2::{
        issue_credential::{
            IssueCredentialAttachmentFormatType, IssueCredentialV2, IssueCredentialV2Content,
            IssueCredentialV2Decorators,
        },
        offer_credential::{
            OfferCredentialAttachmentFormatType, OfferCredentialV2, OfferCredentialV2Content,
            OfferCredentialV2Decorators,
        },
        CredentialPreviewV2,
    },
};
use shared_vcx::maybe_known::MaybeKnown;
use uuid::Uuid;

use super::create_attachments_and_formats;

pub fn create_offer_message_from_attachments(
    attachments_format_and_data: Vec<(MaybeKnown<OfferCredentialAttachmentFormatType>, Vec<u8>)>,
    preview: CredentialPreviewV2,
    replacement_id: Option<String>,
    thread_id: Option<String>,
) -> OfferCredentialV2 {
    let (attachments, formats) = create_attachments_and_formats(attachments_format_and_data);

    let content = OfferCredentialV2Content::builder()
        .credential_preview(preview)
        .formats(formats)
        .offers_attach(attachments)
        .replacement_id(replacement_id)
        .build();

    let decorators = OfferCredentialV2Decorators::builder()
        .thread(thread_id.map(|id| Thread::builder().thid(id).build()))
        .build();

    OfferCredentialV2::builder()
        .id(Uuid::new_v4().to_string())
        .content(content)
        .decorators(decorators)
        .build()
}

pub fn create_credential_message_from_attachments(
    attachments_format_and_data: Vec<(MaybeKnown<IssueCredentialAttachmentFormatType>, Vec<u8>)>,
    please_ack: bool,
    thread_id: String,
    replacement_id: Option<String>,
) -> IssueCredentialV2 {
    let (attachments, formats) = create_attachments_and_formats(attachments_format_and_data);

    let content = IssueCredentialV2Content::builder()
        .formats(formats)
        .credentials_attach(attachments)
        .replacement_id(replacement_id)
        .build();

    let decorators = IssueCredentialV2Decorators::builder()
        .thread(Thread::builder().thid(thread_id).build())
        .please_ack(please_ack.then_some(PleaseAck::builder().on(vec![AckOn::Outcome]).build()))
        .build();

    IssueCredentialV2::builder()
        .id(Uuid::new_v4().to_string())
        .content(content)
        .decorators(decorators)
        .build()
}
