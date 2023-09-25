use std::{collections::HashMap, marker::PhantomData, sync::Arc};

use aries_vcx_core::anoncreds::base_anoncreds::BaseAnonCreds;
use async_trait::async_trait;

use super::IssuerCredentialIssuanceFormat;
use crate::{
    errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult},
    protocols::issuance_v2::messages::RequestCredentialV2,
    utils::openssl::encode,
};

pub struct AnoncredsIssuerCredentialIssuanceFormat<'a> {
    _marker: &'a PhantomData<()>,
}

pub struct AnoncredsCreateOfferInput<'a> {
    pub anoncreds: &'a Arc<dyn BaseAnonCreds>,
    pub cred_def_id: String,
}

pub struct AnoncredsCreatedOfferMetadata {
    pub offer_json: String,
}

pub struct AnoncredsCreateCredentialInput<'a> {
    pub anoncreds: &'a Arc<dyn BaseAnonCreds>,
    pub credential_attributes: HashMap<String, String>,
    pub revocation_info: Option<AnoncredsCreateCredentialRevocationInfoInput>,
}

pub struct AnoncredsCreateCredentialRevocationInfoInput {
    pub registry_id: String,
    pub tails_directory: String,
}

pub struct AnoncredsCreatedCredentialMetadata {
    pub credential_revocation_id: Option<String>,
}

#[async_trait]
impl<'a> IssuerCredentialIssuanceFormat for AnoncredsIssuerCredentialIssuanceFormat<'a> {
    type CreateOfferInput = AnoncredsCreateOfferInput<'a>;
    type CreatedOfferMetadata = AnoncredsCreatedOfferMetadata;

    type CreateCredentialInput = AnoncredsCreateCredentialInput<'a>;
    type CreatedCredentialMetadata = AnoncredsCreatedCredentialMetadata;

    fn supports_request_independent_of_offer() -> bool {
        false
    }

    fn supports_multi_credential_issuance() -> bool {
        false
    }

    fn get_offer_attachment_format() -> String {
        String::from("anoncreds/credential-offer@v1.0")
    }
    fn get_credential_attachment_format() -> String {
        String::from("anoncreds/credential@v1.0")
    }

    // https://github.com/hyperledger/aries-rfcs/blob/main/features/0771-anoncreds-attachments/README.md#credential-offer-format
    async fn create_offer_attachment_content(
        data: &AnoncredsCreateOfferInput,
    ) -> VcxResult<(Vec<u8>, AnoncredsCreatedOfferMetadata)> {
        let cred_offer = data
            .anoncreds
            .issuer_create_credential_offer(&data.cred_def_id)
            .await?;

        Ok((
            cred_offer.clone().into_bytes(),
            AnoncredsCreatedOfferMetadata {
                offer_json: cred_offer,
            },
        ))
    }

    // https://github.com/hyperledger/aries-rfcs/blob/main/features/0771-anoncreds-attachments/README.md#credential-format
    async fn create_credential_attachment_content(
        offer_metadata: &AnoncredsCreatedOfferMetadata,
        request_message: &RequestCredentialV2,
        data: &AnoncredsCreateCredentialInput,
    ) -> VcxResult<(Vec<u8>, AnoncredsCreatedCredentialMetadata)> {
        let offer = &offer_metadata.offer_json;

        _ = request_message;
        let request = String::from("extract from msg");

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
                &request,
                &encoded_credential_attributes_json,
                rev_reg_id,
                tails_dir,
            )
            .await?;

        let metadata = AnoncredsCreatedCredentialMetadata {
            credential_revocation_id: cred_rev_id,
        };

        Ok((credential.into_bytes(), metadata))
    }

    async fn create_credential_attachment_content_independent_of_offer(
        _: &RequestCredentialV2,
        _: &Self::CreateCredentialInput,
    ) -> VcxResult<(Vec<u8>, AnoncredsCreatedCredentialMetadata)> {
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
