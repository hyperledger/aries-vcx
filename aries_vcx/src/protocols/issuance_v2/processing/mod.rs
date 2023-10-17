use base64::{engine::general_purpose, Engine};
use messages::{
    decorators::attachment::{Attachment, AttachmentData, AttachmentType},
    misc::MimeType,
    msg_fields::protocols::cred_issuance::v2::AttachmentFormatSpecifier,
};
use shared_vcx::maybe_known::MaybeKnown;
use uuid::Uuid;

pub mod holder;
pub mod issuer;

fn create_attachments_and_formats<T>(
    attachments_format_and_data: Vec<(MaybeKnown<T>, Vec<u8>)>,
) -> (Vec<Attachment>, Vec<AttachmentFormatSpecifier<T>>) {
    let mut attachments = vec![];
    let mut formats = vec![];

    for (format, attachment_data) in attachments_format_and_data {
        let attachment_content =
            AttachmentType::Base64(general_purpose::URL_SAFE.encode(&attachment_data));
        let attach_id = Uuid::new_v4().to_string();
        let attachment = Attachment::builder()
            .id(attach_id.clone())
            .mime_type(MimeType::Json)
            .data(
                AttachmentData::builder()
                    .content(attachment_content)
                    .build(),
            )
            .build();

        let format_specifier = AttachmentFormatSpecifier::builder()
            .attach_id(attach_id)
            .format(format)
            .build();

        attachments.push(attachment);
        formats.push(format_specifier);
    }

    (attachments, formats)
}
