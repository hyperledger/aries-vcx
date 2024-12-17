use std::str::FromStr;

use chrono::{DateTime, Utc};
use did_resolver::{
    did_doc::schema::{
        contexts,
        did_doc::DidDocument,
        service::Service,
        types::uri::Uri,
        utils::OneOrList,
        verification_method::{PublicKeyField, VerificationMethod, VerificationMethodType},
    },
    did_parser_nom::Did,
    shared_types::{did_document_metadata::DidDocumentMetadata, did_resource::DidResourceMetadata},
};
use serde_json::json;

use crate::{
    error::{DidCheqdError, DidCheqdResult},
    proto::cheqd::{
        did::v2::{
            DidDoc as CheqdDidDoc, Metadata as CheqdDidDocMetadata, Service as CheqdService,
            VerificationMethod as CheqdVerificationMethod,
        },
        resource::v2::Metadata as CheqdResourceMetadata,
    },
};

impl TryFrom<CheqdDidDoc> for DidDocument {
    type Error = DidCheqdError;

    fn try_from(value: CheqdDidDoc) -> Result<Self, Self::Error> {
        let mut doc = DidDocument::new(value.id.parse()?);
        let mut context = value.context;

        // insert default context
        if !context.iter().any(|ctx| ctx == contexts::W3C_DID_V1) {
            context.push(contexts::W3C_DID_V1.to_owned());
        }

        let controller: Vec<_> = value
            .controller
            .into_iter()
            .map(Did::parse)
            .collect::<Result<_, _>>()?;
        if !controller.is_empty() {
            doc.set_controller(OneOrList::from(controller));
        }

        for vm in value.verification_method {
            let vm = VerificationMethod::try_from(vm)?;
            let vm_ctx = vm.verification_method_type().context_for_type();
            if !context.iter().any(|ctx| ctx == vm_ctx) {
                context.push(vm_ctx.to_owned());
            }

            doc.add_verification_method(vm);
        }

        for vm_id in value.authentication {
            doc.add_authentication_ref(vm_id.parse()?);
        }
        for vm_id in value.assertion_method {
            doc.add_assertion_method_ref(vm_id.parse()?);
        }
        for vm_id in value.capability_invocation {
            doc.add_capability_invocation_ref(vm_id.parse()?);
        }
        for vm_id in value.capability_delegation {
            doc.add_capability_delegation_ref(vm_id.parse()?);
        }
        for vm_id in value.key_agreement {
            doc.add_key_agreement_ref(vm_id.parse()?);
        }

        for svc in value.service {
            let svc = Service::try_from(svc)?;
            doc.add_service(svc);
        }

        let aka: Vec<_> = value
            .also_known_as
            .iter()
            .map(|aka| Uri::from_str(aka))
            .collect::<Result<_, _>>()?;
        doc.set_also_known_as(aka);

        // add in all contexts
        doc.set_extra_field(String::from("@context"), json!(context));

        Ok(doc)
    }
}

impl TryFrom<CheqdVerificationMethod> for VerificationMethod {
    type Error = DidCheqdError;

    fn try_from(value: CheqdVerificationMethod) -> Result<Self, Self::Error> {
        let vm_type: VerificationMethodType =
            serde_json::from_value(json!(value.verification_method_type))?;

        let vm_key_encoded = value.verification_material;

        let pk = match vm_type {
            VerificationMethodType::Ed25519VerificationKey2020 => PublicKeyField::Multibase {
                public_key_multibase: vm_key_encoded,
            },
            VerificationMethodType::JsonWebKey2020 => PublicKeyField::Jwk {
                public_key_jwk: serde_json::from_str(&vm_key_encoded)?,
            },
            VerificationMethodType::Ed25519VerificationKey2018 => PublicKeyField::Base58 {
                public_key_base58: vm_key_encoded,
            },
            // https://w3c.github.io/vc-di-bbs/contexts/v1/
            VerificationMethodType::Bls12381G1Key2020 => PublicKeyField::Base58 {
                public_key_base58: vm_key_encoded,
            },
            // https://w3c.github.io/vc-di-bbs/contexts/v1/
            VerificationMethodType::Bls12381G2Key2020 => PublicKeyField::Base58 {
                public_key_base58: vm_key_encoded,
            },
            // https://ns.did.ai/suites/x25519-2019/v1/
            VerificationMethodType::X25519KeyAgreementKey2019 => PublicKeyField::Base58 {
                public_key_base58: vm_key_encoded,
            },
            // https://ns.did.ai/suites/x25519-2020/v1/
            VerificationMethodType::X25519KeyAgreementKey2020 => PublicKeyField::Multibase {
                public_key_multibase: vm_key_encoded,
            },
            // https://w3c.github.io/vc-data-integrity/contexts/multikey/v1.jsonld
            VerificationMethodType::Multikey => PublicKeyField::Multibase {
                public_key_multibase: vm_key_encoded,
            },
            // https://w3id.org/pgp/v1
            VerificationMethodType::PgpVerificationKey2021 => PublicKeyField::Pgp {
                public_key_pgp: vm_key_encoded,
            },
            // cannot infer encoding type from vm type, as multiple are supported: https://ns.did.ai/suites/secp256k1-2019/v1/
            VerificationMethodType::EcdsaSecp256k1VerificationKey2019 => {
                return Err(DidCheqdError::InvalidDidDocument(
                    "DidDocument uses VM type of EcdsaSecp256k1VerificationKey2019, cannot process"
                        .into(),
                ))
            }
            // cannot infer encoding type from vm type: https://identity.foundation/EcdsaSecp256k1RecoverySignature2020/lds-ecdsa-secp256k1-recovery2020-0.0.jsonld
            VerificationMethodType::EcdsaSecp256k1RecoveryMethod2020 => {
                return Err(DidCheqdError::InvalidDidDocument(
                    "DidDocument uses VM type of EcdsaSecp256k1RecoveryMethod2020, cannot process"
                        .into(),
                ))
            }
        };

        let vm = VerificationMethod::builder()
            .id(value.id.parse()?)
            .verification_method_type(vm_type)
            .controller(value.controller.parse()?)
            .public_key(pk)
            .build();

        Ok(vm)
    }
}

impl TryFrom<CheqdService> for Service {
    type Error = DidCheqdError;

    fn try_from(value: CheqdService) -> Result<Self, Self::Error> {
        // TODO #1301 - fix mapping: https://github.com/hyperledger/aries-vcx/issues/1301
        let endpoint =
            value
                .service_endpoint
                .into_iter()
                .next()
                .ok_or(DidCheqdError::InvalidDidDocument(
                    "DID Document Service is missing an endpoint".into(),
                ))?;

        let svc = Service::new(
            Uri::from_str(&value.id)?,
            endpoint.parse()?,
            serde_json::from_value(json!(value.service_type))?,
            Default::default(),
        );

        Ok(svc)
    }
}

impl TryFrom<CheqdDidDocMetadata> for DidDocumentMetadata {
    type Error = DidCheqdError;

    fn try_from(value: CheqdDidDocMetadata) -> Result<Self, Self::Error> {
        let mut builder = DidDocumentMetadata::builder();
        if let Some(timestamp) = value.created {
            builder = builder.created(prost_timestamp_to_dt(timestamp)?);
        }
        if let Some(timestamp) = value.updated {
            builder = builder.updated(prost_timestamp_to_dt(timestamp)?);
        }
        builder = builder
            .deactivated(value.deactivated)
            .version_id(value.version_id)
            .next_version_id(value.next_version_id);

        Ok(builder.build())
    }
}

pub(super) struct CheqdResourceMetadataWithUri {
    pub uri: String,
    pub meta: CheqdResourceMetadata,
}

impl TryFrom<CheqdResourceMetadataWithUri> for DidResourceMetadata {
    type Error = DidCheqdError;

    fn try_from(value: CheqdResourceMetadataWithUri) -> Result<Self, Self::Error> {
        let uri = value.uri;
        let value = value.meta;

        let Some(created) = value.created else {
            return Err(DidCheqdError::InvalidDidDocument(format!(
                "created field missing from resource: {value:?}"
            )))?;
        };

        let version = (!value.version.trim().is_empty()).then_some(value.version);
        let previous_version_id =
            (!value.previous_version_id.trim().is_empty()).then_some(value.previous_version_id);
        let next_version_id =
            (!value.next_version_id.trim().is_empty()).then_some(value.next_version_id);

        let also_known_as = value
            .also_known_as
            .into_iter()
            .map(|aka| {
                json!({
                    "uri": aka.uri,
                    "description": aka.description
                })
            })
            .collect();

        Ok(DidResourceMetadata::builder()
            .resource_uri(uri)
            .resource_collection_id(value.collection_id)
            .resource_id(value.id)
            .resource_name(value.name)
            .resource_type(value.resource_type)
            .resource_version(version)
            .also_known_as(Some(also_known_as))
            .media_type(value.media_type)
            .created(prost_timestamp_to_dt(created)?)
            .updated(None)
            .checksum(value.checksum)
            .previous_version_id(previous_version_id)
            .next_version_id(next_version_id)
            .build())
    }
}

fn prost_timestamp_to_dt(mut timestamp: prost_types::Timestamp) -> DidCheqdResult<DateTime<Utc>> {
    timestamp.normalize();
    DateTime::from_timestamp(timestamp.seconds, timestamp.nanos.try_into()?).ok_or(
        DidCheqdError::Other(format!("Unknown error, bad timestamp: {timestamp:?}").into()),
    )
}
