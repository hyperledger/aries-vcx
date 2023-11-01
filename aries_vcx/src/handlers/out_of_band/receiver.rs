use std::{clone::Clone, fmt::Display, str::FromStr, sync::Arc};

use aries_vcx_core::ledger::base_ledger::IndyLedgerRead;
use base64::{engine::general_purpose, Engine};
use did_doc_sov::{service::ServiceSov, DidDocumentSov};
use did_parser::Did;
use did_resolver::traits::resolvable::resolution_output::DidResolutionOutput;
use did_resolver_registry::ResolverRegistry;
use diddoc_legacy::aries::{diddoc::AriesDidDoc, service::AriesService};
use messages::{
    decorators::attachment::{Attachment, AttachmentType},
    msg_fields::protocols::{
        cred_issuance::v1::offer_credential::OfferCredentialV1,
        out_of_band::{
            invitation::{Invitation, OobService},
            OutOfBand,
        },
        present_proof::v1::request::RequestPresentationV1,
    },
    AriesMessage,
};
use serde::Deserialize;
use serde_json::Value;

use crate::{
    common::ledger::transactions::resolve_service, errors::error::prelude::*,
    handlers::util::AttachmentId, utils::from_legacy_service_to_service_sov,
};

const DID_KEY_PREFIX: &str = "did:key:";
const ED25519_MULTIBASE_CODEC: [u8; 2] = [0xed, 0x01];

#[derive(Debug, PartialEq, Clone)]
pub struct OutOfBandReceiver {
    pub oob: Invitation,
}

impl OutOfBandReceiver {
    pub fn create_from_a2a_msg(msg: &AriesMessage) -> VcxResult<Self> {
        trace!("OutOfBandReceiver::create_from_a2a_msg >>> msg: {:?}", msg);
        match msg {
            AriesMessage::OutOfBand(OutOfBand::Invitation(oob)) => {
                Ok(OutOfBandReceiver { oob: oob.clone() })
            }
            m => Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidMessageFormat,
                format!(
                    "Expected OutOfBandInvitation message to create OutOfBandReceiver, but \
                     received message of unknown type: {:?}",
                    m
                ),
            )),
        }
    }

    pub fn get_id(&self) -> String {
        self.oob.id.clone()
    }

    // TODO: There may be multiple A2AMessages in a single OoB msg
    pub fn extract_a2a_message(&self) -> VcxResult<Option<AriesMessage>> {
        trace!("OutOfBandReceiver::extract_a2a_message >>>");
        if let Some(attach) = self
            .oob
            .content
            .requests_attach
            .as_ref()
            .and_then(|v| v.get(0))
        {
            attachment_to_aries_message(attach)
        } else {
            Ok(None)
        }
    }

    pub fn to_aries_message(&self) -> AriesMessage {
        self.oob.clone().into()
    }

    pub fn from_string(oob_data: &str) -> VcxResult<Self> {
        Ok(Self {
            oob: serde_json::from_str(oob_data)?,
        })
    }
}

impl Display for OutOfBandReceiver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", json!(AriesMessage::from(self.oob.clone())))
    }
}

fn attachment_to_aries_message(attach: &Attachment) -> VcxResult<Option<AriesMessage>> {
    let AttachmentType::Base64(encoded_attach) = &attach.data.content else {
        return Err(AriesVcxError::from_msg(
            AriesVcxErrorKind::SerializationError,
            format!("Attachment is not base 64 encoded JSON: {attach:?}"),
        ));
    };

    let Ok(bytes) = general_purpose::STANDARD.decode(encoded_attach) else {
        return Err(AriesVcxError::from_msg(
            AriesVcxErrorKind::SerializationError,
            format!("Attachment is not base 64 encoded JSON: {attach:?}"),
        ));
    };

    let attach_json: Value = serde_json::from_slice(&bytes).map_err(|_| {
        AriesVcxError::from_msg(
            AriesVcxErrorKind::SerializationError,
            format!("Attachment is not base 64 encoded JSON: {attach:?}"),
        )
    })?;

    let attach_id = if let Some(attach_id) = attach.id.as_deref() {
        AttachmentId::from_str(attach_id).map_err(|err| {
            AriesVcxError::from_msg(
                AriesVcxErrorKind::SerializationError,
                format!("Failed to deserialize attachment ID: {}", err),
            )
        })
    } else {
        Err(AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidMessageFormat,
            format!("Missing attachment ID on attach: {attach:?}"),
        ))
    }?;

    match attach_id {
        AttachmentId::CredentialOffer => {
            let offer = OfferCredentialV1::deserialize(&attach_json).map_err(|_| {
                AriesVcxError::from_msg(
                    AriesVcxErrorKind::SerializationError,
                    format!("Failed to deserialize attachment: {attach_json:?}"),
                )
            })?;
            Ok(Some(offer.into()))
        }
        AttachmentId::PresentationRequest => {
            let request = RequestPresentationV1::deserialize(&attach_json).map_err(|_| {
                AriesVcxError::from_msg(
                    AriesVcxErrorKind::SerializationError,
                    format!("Failed to deserialize attachment: {attach_json:?}"),
                )
            })?;
            Ok(Some(request.into()))
        }
        _ => Err(AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidMessageFormat,
            format!("unexpected attachment type: {:?}", attach_id),
        )),
    }
}

pub async fn oob_invitation_to_legacy_did_doc(
    indy_ledger: &impl IndyLedgerRead,
    invitation: &Invitation,
) -> VcxResult<AriesDidDoc> {
    let mut did_doc: AriesDidDoc = AriesDidDoc::default();
    let (service_endpoint, recipient_keys, routing_keys) = {
        did_doc.set_id(invitation.id.clone());
        let service = resolve_service(indy_ledger, &invitation.content.services[0])
            .await
            .unwrap_or_else(|err| {
                error!("Failed to obtain service definition from the ledger: {err}");
                AriesService::default()
            });
        let recipient_keys =
            normalize_keys_as_naked(&service.recipient_keys).unwrap_or_else(|err| {
                error!(
                    "Failed to normalize keys of service {} as naked keys: {err}",
                    &service
                );
                Vec::new()
            });
        (
            service.service_endpoint,
            recipient_keys,
            service.routing_keys,
        )
    };
    did_doc.set_service_endpoint(service_endpoint);
    did_doc.set_recipient_keys(recipient_keys);
    did_doc.set_routing_keys(routing_keys);
    Ok(did_doc)
}

fn normalize_keys_as_naked(keys_list: &Vec<String>) -> VcxResult<Vec<String>> {
    let mut result = Vec::new();
    for key in keys_list {
        if let Some(stripped_didkey) = key.strip_prefix(DID_KEY_PREFIX) {
            let stripped = if let Some(stripped) = stripped_didkey.strip_prefix('z') {
                stripped
            } else {
                Err(AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidDid,
                    format!("z prefix is missing: {}", key),
                ))?
            };
            let decoded_value = bs58::decode(stripped).into_vec().map_err(|_| {
                AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidDid,
                    format!(
                        "Could not decode base58: {} as portion of {}",
                        stripped, key
                    ),
                )
            })?;
            let verkey = if let Some(public_key_bytes) =
                decoded_value.strip_prefix(&ED25519_MULTIBASE_CODEC)
            {
                Ok(bs58::encode(public_key_bytes).into_string())
            } else {
                Err(AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidDid,
                    format!(
                        "Only Ed25519-based did:keys are currently supported, got key: {}",
                        key
                    ),
                ))
            }?;
            result.push(verkey);
        } else {
            result.push(key.clone());
        }
    }
    Ok(result)
}

pub async fn oob_invitation_to_diddoc(
    resolver_registry: &Arc<ResolverRegistry>,
    invitation: Invitation,
) -> VcxResult<DidDocumentSov> {
    let mut builder = DidDocumentSov::builder(Default::default());

    let mut resolved_services = vec![];
    let mut resolved_vms = vec![];
    let mut resolved_kas = vec![];
    let mut resolved_dids = vec![];

    for service in invitation.content.services {
        match service {
            OobService::SovService(service) => {
                builder = builder.add_service(service.clone());
            }
            OobService::Did(did) => {
                let parsed_did = Did::parse(did)?;
                let DidResolutionOutput { did_document, .. } = resolver_registry
                    .resolve(&parsed_did, &Default::default())
                    .await?;
                resolved_services.extend(
                    did_document
                        .service()
                        .iter()
                        .map(|s| ServiceSov::try_from(s.clone()))
                        .collect::<Result<Vec<_>, _>>()?,
                );
                resolved_vms.extend_from_slice(did_document.verification_method());
                resolved_kas.extend_from_slice(did_document.key_agreement());
                resolved_dids.push(parsed_did);
            }
            OobService::AriesService(service) => {
                resolved_services.push(from_legacy_service_to_service_sov(service.clone())?)
            }
        }
    }

    for service in resolved_services {
        builder = builder.add_service(service);
    }

    for vm in resolved_vms {
        builder = builder.add_verification_method(vm.clone());
    }

    for ka in resolved_kas {
        builder = builder.add_key_agreement(ka.clone());
    }

    for did in resolved_dids {
        builder = builder.add_controller(did);
    }

    Ok(builder.build())
}
