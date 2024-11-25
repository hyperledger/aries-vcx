use std::str::FromStr;

use did_resolver::{
    did_doc::schema::{
        did_doc::DidDocument,
        types::uri::Uri,
        utils::OneOrList,
        verification_method::{PublicKeyField, VerificationMethod, VerificationMethodType},
    },
    did_parser_nom::Did,
};
use serde_json::json;

use crate::{
    error::DidCheqdError,
    proto::cheqd::did::v2::{DidDoc as CheqdDidDoc, VerificationMethod as CheqdVerificationMethod},
};

impl TryFrom<CheqdDidDoc> for DidDocument {
    type Error = DidCheqdError;

    fn try_from(value: CheqdDidDoc) -> Result<Self, Self::Error> {
        let mut doc = DidDocument::new(value.id.parse()?);
        doc.set_extra_field(String::from("@context"), json!(value.context));

        let controller: Vec<_> = value
            .controller
            .into_iter()
            .map(Did::parse)
            .collect::<Result<_, _>>()?;
        doc.set_controller(OneOrList::from(controller));

        for vm in value.verification_method {
            let vm = VerificationMethod::try_from(vm)?;
            doc.add_verification_method(vm);
            // TODO - would be nice to append relevant contexts too
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

        let aka: Vec<_> = value
            .also_known_as
            .iter()
            .map(|aka| Uri::from_str(aka))
            .collect::<Result<_, _>>()?;
        doc.set_also_known_as(aka);

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
            // cannot infer encoding type from vm type, as multiple are supported: https://ns.did.ai/suites/secp256k1-2019/v1/
            VerificationMethodType::EcdsaSecp256k1VerificationKey2019 => todo!(),
            // not supported
            VerificationMethodType::PgpVerificationKey2021 => todo!(),
            // not supported
            VerificationMethodType::RsaVerificationKey2018 => todo!(),
            // cannot infer encoding type from vm type: https://identity.foundation/EcdsaSecp256k1RecoverySignature2020/lds-ecdsa-secp256k1-recovery2020-0.0.jsonld
            VerificationMethodType::EcdsaSecp256k1RecoveryMethod2020 => todo!(),
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
