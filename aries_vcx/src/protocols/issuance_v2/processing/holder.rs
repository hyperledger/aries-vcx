use messages::{
    decorators::thread::Thread,
    msg_fields::protocols::cred_issuance::v2::{
        propose_credential::{
            ProposeCredentialAttachmentFormatType, ProposeCredentialV2, ProposeCredentialV2Content,
            ProposeCredentialV2Decorators,
        },
        request_credential::{
            RequestCredentialAttachmentFormatType, RequestCredentialV2, RequestCredentialV2Content,
            RequestCredentialV2Decorators,
        },
        CredentialPreviewV2,
    },
};
use shared_vcx::maybe_known::MaybeKnown;
use uuid::Uuid;

use super::create_attachments_and_formats;

pub fn create_proposal_message_from_attachments(
    attachments_format_and_data: Vec<(MaybeKnown<ProposeCredentialAttachmentFormatType>, Vec<u8>)>,
    preview: Option<CredentialPreviewV2>,
    thread_id: Option<String>,
) -> ProposeCredentialV2 {
    let (attachments, formats) = create_attachments_and_formats(attachments_format_and_data);

    let content = ProposeCredentialV2Content::builder()
        .formats(formats)
        .filters_attach(attachments)
        .credential_preview(preview)
        .build();

    let decorators = ProposeCredentialV2Decorators::builder()
        .thread(thread_id.map(|id| Thread::builder().thid(id).build()))
        .build();

    ProposeCredentialV2::builder()
        .id(Uuid::new_v4().to_string())
        .content(content)
        .decorators(decorators)
        .build()
}

pub fn create_request_message_from_attachments(
    attachments_format_and_data: Vec<(MaybeKnown<RequestCredentialAttachmentFormatType>, Vec<u8>)>,
    thread_id: Option<String>,
) -> RequestCredentialV2 {
    let (attachments, formats) = create_attachments_and_formats(attachments_format_and_data);

    let content = RequestCredentialV2Content::builder()
        .formats(formats)
        .requests_attach(attachments)
        .build();

    let decorators = RequestCredentialV2Decorators::builder()
        .thread(thread_id.map(|id| Thread::builder().thid(id).build()))
        .build();

    RequestCredentialV2::builder()
        .id(Uuid::new_v4().to_string())
        .content(content)
        .decorators(decorators)
        .build()
}
