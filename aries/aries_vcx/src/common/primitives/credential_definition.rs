use anoncreds_types::data_types::{
    identifiers::{cred_def_id::CredentialDefinitionId, schema_id::SchemaId},
    ledger::{
        cred_def::{CredentialDefinition, SignatureType},
        schema::Schema,
    },
    messages::cred_definition_config::CredentialDefinitionConfig,
};
use aries_vcx_anoncreds::{
    anoncreds::base_anoncreds::BaseAnonCreds, constants::DEFAULT_SERIALIZE_VERSION,
};
use aries_vcx_ledger::{
    errors::error::VcxLedgerError,
    ledger::base_ledger::{AnoncredsLedgerRead, AnoncredsLedgerWrite},
};
use aries_vcx_wallet::wallet::base_wallet::BaseWallet;
use did_parser_nom::Did;

use crate::{
    errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult},
    utils::serialization::ObjectWithVersion,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Default)]
#[serde(try_from = "u8")]
#[repr(u8)]
pub enum PublicEntityStateType {
    Built = 0,
    #[default]
    Published = 1,
}

impl ::serde::Serialize for PublicEntityStateType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ::serde::Serializer,
    {
        serializer.serialize_u8(*self as u8)
    }
}

impl TryFrom<u8> for PublicEntityStateType {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(PublicEntityStateType::Built),
            1 => Ok(PublicEntityStateType::Published),
            _ => Err(format!(
                "unknown {} value: {}",
                stringify!(PublicEntityStateType),
                value
            )),
        }
    }
}

// TODO: Deduplicate the data
#[derive(Deserialize, Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CredentialDef {
    #[serde(alias = "cred_def_id")]
    id: CredentialDefinitionId,
    tag: String,
    source_id: String,
    issuer_did: Did,
    cred_def_json: String,
    support_revocation: bool,
    #[serde(default)]
    schema_id: SchemaId,
    #[serde(default)]
    pub state: PublicEntityStateType,
}

async fn _try_get_cred_def_from_ledger(
    ledger: &impl AnoncredsLedgerRead,
    issuer_did: &Did,
    cred_def_id: &CredentialDefinitionId,
) -> VcxResult<Option<String>> {
    match ledger.get_cred_def(cred_def_id, Some(issuer_did)).await {
        Ok(cred_def) => Ok(Some(serde_json::to_string(&cred_def)?)),
        Err(err) => match err {
            VcxLedgerError::LedgerItemNotFound => Ok(None),
            _ => Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidLedgerResponse,
                format!(
                    "Failed to check presence of credential definition id {} on the \
                     ledger\nError: {}",
                    cred_def_id, err
                ),
            )),
        },
    }
}

impl CredentialDef {
    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        wallet: &impl BaseWallet,
        ledger_read: &impl AnoncredsLedgerRead,
        anoncreds: &impl BaseAnonCreds,
        source_id: String,
        issuer_did: Did,
        schema_id: SchemaId,
        tag: String,
        support_revocation: bool,
    ) -> VcxResult<Self> {
        trace!("CredentialDef::create >>> source_id: {}", source_id);
        let schema_json = ledger_read
            .get_schema(&schema_id, Some(&issuer_did))
            .await?;
        let cred_def = generate_cred_def(
            wallet,
            anoncreds,
            &issuer_did,
            &schema_id,
            schema_json,
            &tag,
            None,
            Some(support_revocation),
        )
        .await?;
        Ok(Self {
            source_id,
            tag,
            id: cred_def.id.to_owned(),
            cred_def_json: serde_json::to_string(&cred_def)?,
            issuer_did,
            support_revocation,
            schema_id,
            state: PublicEntityStateType::Built,
        })
    }

    pub fn was_published(&self) -> bool {
        self.state == PublicEntityStateType::Published
    }

    pub fn get_support_revocation(&self) -> bool {
        self.support_revocation
    }

    pub fn get_cred_def_json(&self) -> CredentialDefinition {
        serde_json::from_str(&self.cred_def_json).unwrap()
    }

    pub async fn publish_cred_def(
        self,
        wallet: &impl BaseWallet,
        ledger_read: &impl AnoncredsLedgerRead,
        ledger_write: &impl AnoncredsLedgerWrite,
    ) -> VcxResult<Self> {
        trace!(
            "publish_cred_def >>> issuer_did: {}, cred_def_id: {}",
            self.issuer_did,
            self.id
        );
        if let Some(ledger_cred_def_json) =
            _try_get_cred_def_from_ledger(ledger_read, &self.issuer_did, &self.id).await?
        {
            return Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::CredDefAlreadyCreated,
                format!(
                    "Credential definition with id {} already exists on the ledger: {}",
                    self.id, ledger_cred_def_json
                ),
            ));
        }
        let cred_def_json = serde_json::from_str(&self.cred_def_json)?;
        ledger_write
            .publish_cred_def(wallet, cred_def_json, &self.issuer_did)
            .await?;
        Ok(Self {
            state: PublicEntityStateType::Published,
            ..self
        })
    }

    pub fn from_string(data: &str) -> VcxResult<Self> {
        ObjectWithVersion::deserialize(data)
            .map(|obj: ObjectWithVersion<Self>| obj.data)
            .map_err(|err| {
                AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidJson,
                    format!("Cannot deserialize CredentialDefinition: {}", err),
                )
            })
    }

    pub fn to_string(&self) -> VcxResult<String> {
        ObjectWithVersion::new(DEFAULT_SERIALIZE_VERSION, self.to_owned())
            .serialize()
            .map_err(|err: AriesVcxError| err.extend("Cannot serialize CredentialDefinition"))
    }

    pub fn get_data_json(&self) -> VcxResult<String> {
        serde_json::to_string(&self).map_err(|_| {
            AriesVcxError::from_msg(
                AriesVcxErrorKind::SerializationError,
                "Failed to serialize credential definition",
            )
        })
    }

    pub fn get_source_id(&self) -> &String {
        &self.source_id
    }

    pub fn get_cred_def_id(&self) -> &CredentialDefinitionId {
        &self.id
    }

    pub fn get_schema_id(&self) -> &SchemaId {
        &self.schema_id
    }

    pub fn set_source_id(&mut self, source_id: String) {
        self.source_id = source_id;
    }

    pub async fn update_state(&mut self, ledger: &impl AnoncredsLedgerRead) -> VcxResult<u32> {
        if (ledger
            .get_cred_def(&self.id.to_string().try_into()?, None)
            .await)
            .is_ok()
        {
            self.state = PublicEntityStateType::Published
        }
        Ok(self.state as u32)
    }

    pub fn get_state(&self) -> u32 {
        self.state as u32
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn generate_cred_def(
    wallet: &impl BaseWallet,
    anoncreds: &impl BaseAnonCreds,
    issuer_did: &Did,
    schema_id: &SchemaId,
    schema_json: Schema,
    tag: &str,
    // TODO: These should not be options
    sig_type: Option<&str>,
    support_revocation: Option<bool>,
) -> VcxResult<CredentialDefinition> {
    trace!(
        "generate_cred_def >>> issuer_did: {}, schema_json: {:?}, tag: {}, sig_type: {:?}, \
         support_revocation: {:?}",
        issuer_did,
        schema_json,
        tag,
        sig_type,
        support_revocation
    );

    let config_json = CredentialDefinitionConfig {
        support_revocation: support_revocation.unwrap_or_default(),
        tag: tag.to_string(),
        signature_type: SignatureType::CL,
    };

    let cred_def = anoncreds
        .issuer_create_and_store_credential_def(
            wallet,
            issuer_did,
            schema_id,
            schema_json,
            config_json,
        )
        .await?;
    Ok(cred_def)
}
