use anoncreds::{
    data_types::{
        cred_def::{
            CredentialDefinition as AnoncredsCredentialDefinition,
            CredentialDefinitionData as AnoncredsCredentialDefinitionData,
            CredentialDefinitionId as AnoncredsCredentialDefinitionId,
        },
        issuer_id::IssuerId as AnoncredsIssuerId,
        rev_reg_def::RevocationRegistryDefinitionValue as AnoncredsRevocationRegistryDefinitionValue,
        schema::{Schema as AnoncredsSchema, SchemaId as AnoncredsSchemaId},
    },
    types::{
        AttributeNames as AnoncredsAttributeNames, CredentialOffer as AnoncredsCredentialOffer,
        CredentialRequest as AnoncredsCredentialRequest,
        RevocationRegistry as AnoncredsRevocationRegistry,
        RevocationRegistryDefinition as AnoncredsRevocationRegistryDefinition,
    },
};
use anoncreds_types::data_types::{
    identifiers::{
        cred_def_id::CredentialDefinitionId as OurCredentialDefinitionId,
        issuer_id::IssuerId as OurIssuerId,
        rev_reg_def_id::RevocationRegistryDefinitionId as OurRevocationRegistryDefinitionId,
        schema_id::SchemaId as OurSchemaId,
    },
    ledger::{
        cred_def::{
            CredentialDefinition as OurCredentialDefinition,
            CredentialDefinitionData as OurCredentialDefinitionData, SignatureType,
        },
        rev_reg::RevocationRegistry as OurRevocationRegistry,
        rev_reg_def::{
            RevocationRegistryDefinition as OurRevocationRegistryDefinition,
            RevocationRegistryDefinitionValue as OurRevocationRegistryDefinitionValue,
        },
        schema::{AttributeNames as OurAttributeNames, Schema as OurSchema},
    },
    messages::{
        cred_offer::CredentialOffer as OurCredentialOffer,
        cred_request::CredentialRequest as OurCredentialRequest,
    },
};

pub trait Convert {
    type Args;
    type Target;
    type Error;

    fn convert(self, args: Self::Args) -> Result<Self::Target, Self::Error>;
}

impl Convert for OurIssuerId {
    type Args = ();
    type Target = AnoncredsIssuerId;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, _args: Self::Args) -> Result<Self::Target, Self::Error> {
        AnoncredsIssuerId::new(self.to_string()).map_err(|e| e.into())
    }
}

impl Convert for OurSchema {
    type Args = ();
    type Target = AnoncredsSchema;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, _: Self::Args) -> Result<Self::Target, Self::Error> {
        Ok(AnoncredsSchema {
            name: self.name,
            version: self.version,
            attr_names: AnoncredsAttributeNames(self.attr_names.into()),
            issuer_id: self.issuer_id.convert(())?,
        })
    }
}

impl Convert for AnoncredsSchema {
    type Args = (String,);
    type Target = OurSchema;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, (schema_id,): Self::Args) -> Result<Self::Target, Self::Error> {
        Ok(OurSchema {
            id: OurSchemaId::new(schema_id)?,
            seq_no: None,
            name: self.name,
            version: self.version,
            attr_names: OurAttributeNames(self.attr_names.into()),
            issuer_id: OurIssuerId::new(self.issuer_id.to_string())?,
        })
    }
}

impl Convert for AnoncredsCredentialDefinition {
    type Args = (String,);
    type Target = OurCredentialDefinition;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, (cred_def_id,): Self::Args) -> Result<Self::Target, Self::Error> {
        Ok(OurCredentialDefinition {
            id: OurCredentialDefinitionId::new(cred_def_id)?,
            schema_id: OurSchemaId::new_unchecked(self.schema_id.to_string()),
            signature_type: SignatureType::CL,
            tag: self.tag,
            value: OurCredentialDefinitionData {
                primary: self.value.primary,
                revocation: self.value.revocation,
            },
            issuer_id: OurIssuerId::new(self.issuer_id.to_string())?,
        })
    }
}

impl Convert for OurCredentialDefinition {
    type Args = ();
    type Target = AnoncredsCredentialDefinition;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, (): Self::Args) -> Result<Self::Target, Self::Error> {
        Ok(AnoncredsCredentialDefinition {
            schema_id: AnoncredsSchemaId::new_unchecked(self.schema_id.to_string()),
            signature_type: anoncreds::types::SignatureType::CL,
            tag: self.tag,
            value: AnoncredsCredentialDefinitionData {
                primary: self.value.primary,
                revocation: self.value.revocation,
            },
            issuer_id: AnoncredsIssuerId::new(self.issuer_id.to_string())?,
        })
    }
}

// impl Convert for OurCredentialRequest {
//     type Args = ();
//     type Target = AnoncredsCredentialRequest;
//     type Error = Box<dyn std::error::Error>;
//
//     fn convert(self, (): Self::Args) -> Result<Self::Target, Self::Error> {
//         Ok(AnoncredsCredentialRequest::new(
//             self.entropy.as_deref(),
//             self.prover_did.as_deref(),
//             AnoncredsCredentialDefinitionId::new(self.cred_def_id.to_string())?,
//             self.blinded_ms,
//             self.blinded_ms_correctness_proof,
//             AnoncredsNonce::from_bytes(self.nonce.as_bytes())?,
//         )?)
//     }
// }
//
// impl Convert for OurCredentialOffer {
//     type Args = ();
//     type Target = AnoncredsCredentialOffer;
//     type Error = Box<dyn std::error::Error>;
//
//     fn convert(self, (): Self::Args) -> Result<Self::Target, Self::Error> {
//         Ok(AnoncredsCredentialOffer {
//             schema_id: AnoncredsSchemaId::new_unchecked(self.schema_id.to_string()),
//             cred_def_id: AnoncredsCredentialDefinitionId::new(self.cred_def_id.0)?,
//             key_correctness_proof: self.key_correctness_proof,
//             nonce: AnoncredsNonce::from_bytes(self.nonce.as_bytes())?,
//             method_name: self.method_name,
//         })
//     }
// }

impl Convert for OurCredentialRequest {
    type Args = ();
    type Target = AnoncredsCredentialRequest;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, (): Self::Args) -> Result<Self::Target, Self::Error> {
        Ok(serde_json::from_str(&serde_json::to_string(&self)?)?)
    }
}

impl Convert for OurCredentialOffer {
    type Args = ();
    type Target = AnoncredsCredentialOffer;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, (): Self::Args) -> Result<Self::Target, Self::Error> {
        Ok(serde_json::from_str(&serde_json::to_string(&self)?)?)
    }
}

impl Convert for AnoncredsCredentialOffer {
    type Args = ();
    type Target = OurCredentialOffer;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, (): Self::Args) -> Result<Self::Target, Self::Error> {
        Ok(serde_json::from_str(&serde_json::to_string(&self)?)?)
    }
}

impl Convert for OurRevocationRegistryDefinition {
    type Args = (String,);
    type Target = AnoncredsRevocationRegistryDefinition;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, (issuer_id,): Self::Args) -> Result<Self::Target, Self::Error> {
        // TODO: Obtain issuer id from cred def id instead
        Ok(AnoncredsRevocationRegistryDefinition {
            issuer_id: AnoncredsIssuerId::new(issuer_id)?,
            revoc_def_type: anoncreds::types::RegistryType::CL_ACCUM,
            tag: self.tag,
            cred_def_id: AnoncredsCredentialDefinitionId::new(self.cred_def_id.to_string())?,
            value: AnoncredsRevocationRegistryDefinitionValue {
                max_cred_num: self.value.max_cred_num,
                public_keys: serde_json::from_value(serde_json::to_value(self.value.public_keys)?)?,
                tails_hash: self.value.tails_hash,
                tails_location: self.value.tails_location,
            },
        })
    }
}

impl Convert for AnoncredsRevocationRegistryDefinition {
    type Args = (String,);
    type Target = OurRevocationRegistryDefinition;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, (rev_reg_def_id,): Self::Args) -> Result<Self::Target, Self::Error> {
        Ok(OurRevocationRegistryDefinition {
            id: OurRevocationRegistryDefinitionId::new(rev_reg_def_id)?,
            revoc_def_type:
                anoncreds_types::data_types::ledger::rev_reg_def::RegistryType::CL_ACCUM,
            tag: self.tag,
            cred_def_id: OurCredentialDefinitionId::new(self.cred_def_id.to_string())?,
            value: OurRevocationRegistryDefinitionValue {
                max_cred_num: self.value.max_cred_num,
                public_keys: serde_json::from_value(serde_json::to_value(self.value.public_keys)?)?,
                tails_hash: self.value.tails_hash,
                tails_location: self.value.tails_location,
            },
        })
    }
}

impl Convert for AnoncredsRevocationRegistry {
    type Args = ();
    type Target = OurRevocationRegistry;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, (): Self::Args) -> Result<Self::Target, Self::Error> {
        todo!()
    }
}
