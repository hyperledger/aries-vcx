use std::{collections::HashMap, marker::PhantomData, sync::Arc};

use aries_vcx_core::anoncreds::base_anoncreds::BaseAnonCreds;
use async_trait::async_trait;
use messages::msg_fields::protocols::cred_issuance::v2::{
    issue_credential::IssueCredentialAttachmentFormatType,
    offer_credential::OfferCredentialAttachmentFormatType,
    propose_credential::{ProposeCredentialAttachmentFormatType, ProposeCredentialV2},
    request_credential::{RequestCredentialAttachmentFormatType, RequestCredentialV2},
};
use shared_vcx::maybe_known::MaybeKnown;

use super::IssuerCredentialIssuanceFormat;
use crate::{
    errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult},
    protocols::issuance_v2::formats::holder::hyperledger_indy::HyperledgerIndyCredentialFilter,
    utils::openssl::encode,
};

// https://github.com/hyperledger/aries-rfcs/blob/b3a3942ef052039e73cd23d847f42947f8287da2/features/0592-indy-attachments/README.md#cred-filter-format

pub struct HyperledgerIndyIssuerCredentialIssuanceFormat<'a> {
    _marker: &'a PhantomData<()>,
}

pub struct HyperledgerIndyCreateOfferInput<'a> {
    pub anoncreds: &'a Arc<dyn BaseAnonCreds>,
    pub cred_def_id: String,
}

#[derive(Clone)]
pub struct HyperledgerIndyCreatedOfferMetadata {
    pub offer_json: String,
}

pub struct HyperledgerIndyCreateCredentialInput<'a> {
    pub anoncreds: &'a Arc<dyn BaseAnonCreds>,
    pub credential_attributes: HashMap<String, String>,
    pub revocation_info: Option<HyperledgerIndyCreateCredentialRevocationInfoInput>,
}

#[derive(Clone)]
pub struct HyperledgerIndyCreateCredentialRevocationInfoInput {
    pub registry_id: String,
    pub tails_directory: String,
}

#[derive(Clone)]
pub struct HyperledgerIndyCreatedCredentialMetadata {
    pub credential_revocation_id: Option<String>,
}

#[async_trait]
impl<'a> IssuerCredentialIssuanceFormat for HyperledgerIndyIssuerCredentialIssuanceFormat<'a> {
    type ProposalDetails = HyperledgerIndyCredentialFilter;

    type CreateOfferInput = HyperledgerIndyCreateOfferInput<'a>;
    type CreatedOfferMetadata = HyperledgerIndyCreatedOfferMetadata;

    type CreateCredentialInput = HyperledgerIndyCreateCredentialInput<'a>;
    type CreatedCredentialMetadata = HyperledgerIndyCreatedCredentialMetadata;

    fn supports_request_independent_of_offer() -> bool {
        false
    }

    fn get_proposal_attachment_format() -> MaybeKnown<ProposeCredentialAttachmentFormatType> {
        MaybeKnown::Known(ProposeCredentialAttachmentFormatType::HyperledgerIndyCredentialFilter2_0)
    }

    fn get_request_attachment_format() -> MaybeKnown<RequestCredentialAttachmentFormatType> {
        MaybeKnown::Known(
            RequestCredentialAttachmentFormatType::HyperledgerIndyCredentialRequest2_0,
        )
    }
    fn get_offer_attachment_format() -> MaybeKnown<OfferCredentialAttachmentFormatType> {
        MaybeKnown::Known(OfferCredentialAttachmentFormatType::HyperledgerIndyCredentialAbstract2_0)
    }
    fn get_credential_attachment_format() -> MaybeKnown<IssueCredentialAttachmentFormatType> {
        MaybeKnown::Known(IssueCredentialAttachmentFormatType::HyperledgerIndyCredential2_0)
    }

    fn extract_proposal_details(
        proposal_message: &ProposeCredentialV2,
    ) -> VcxResult<HyperledgerIndyCredentialFilter> {
        let attachment = Self::extract_proposal_attachment_content(proposal_message)?;

        Ok(serde_json::from_slice(&attachment)?)
    }

    async fn create_offer_attachment_content(
        data: &HyperledgerIndyCreateOfferInput,
    ) -> VcxResult<(Vec<u8>, HyperledgerIndyCreatedOfferMetadata)> {
        let cred_offer = data
            .anoncreds
            .issuer_create_credential_offer(&data.cred_def_id)
            .await?;

        Ok((
            cred_offer.clone().into_bytes(),
            HyperledgerIndyCreatedOfferMetadata {
                offer_json: cred_offer,
            },
        ))
    }

    async fn create_credential_attachment_content(
        offer_metadata: &HyperledgerIndyCreatedOfferMetadata,
        request_message: &RequestCredentialV2,
        data: &HyperledgerIndyCreateCredentialInput,
    ) -> VcxResult<(Vec<u8>, HyperledgerIndyCreatedCredentialMetadata)> {
        let offer = &offer_metadata.offer_json;

        let request_bytes = Self::extract_request_attachment_content(&request_message)?;
        let request_payload = String::from_utf8(request_bytes).map_err(|_| {
            AriesVcxError::from_msg(
                AriesVcxErrorKind::EncodeError,
                "Expected payload to be a utf8 string",
            )
        })?;

        let encoded_credential_attributes = encode_attributes(&data.credential_attributes)?;
        let encoded_credential_attributes_json =
            serde_json::to_string(&encoded_credential_attributes)?;

        let (rev_reg_id, tails_dir) = data.revocation_info.as_ref().map_or((None, None), |info| {
            (
                Some(info.registry_id.to_owned()),
                Some(info.tails_directory.to_owned()),
            )
        });

        let (credential, cred_rev_id, _) = data
            .anoncreds
            .issuer_create_credential(
                offer,
                &request_payload,
                &encoded_credential_attributes_json,
                rev_reg_id,
                tails_dir,
            )
            .await?;

        let metadata = HyperledgerIndyCreatedCredentialMetadata {
            credential_revocation_id: cred_rev_id,
        };

        Ok((credential.into_bytes(), metadata))
    }

    async fn create_credential_attachment_content_independent_of_offer(
        _: &RequestCredentialV2,
        _: &Self::CreateCredentialInput,
    ) -> VcxResult<(Vec<u8>, HyperledgerIndyCreatedCredentialMetadata)> {
        return Err(AriesVcxError::from_msg(
            AriesVcxErrorKind::ActionNotSupported,
            "Creating a credential independent of an offer is unsupported for this format",
        ));
    }
}

fn encode_attributes(
    attributes: &HashMap<String, String>,
) -> VcxResult<HashMap<String, RawAndEncoded>> {
    let mut encoded = HashMap::<String, RawAndEncoded>::new();
    for (k, v) in attributes.into_iter() {
        encoded.insert(
            k.to_owned(),
            RawAndEncoded {
                raw: v.to_owned(),
                encoded: encode(&v)?,
            },
        );
    }

    Ok(encoded)
}

#[derive(Serialize)]
struct RawAndEncoded {
    raw: String,
    encoded: String,
}
