use anoncreds_types::data_types::{ledger::schema::Schema, identifiers::schema_id::SchemaId};
use aries_vcx_core::{
    anoncreds::base_anoncreds::BaseAnonCreds,
    errors::error::AriesVcxCoreErrorKind,
    global::settings::DEFAULT_SERIALIZE_VERSION,
    ledger::base_ledger::{AnoncredsLedgerRead, AnoncredsLedgerWrite},
    wallet::base_wallet::BaseWallet,
};
use did_parser::Did;

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

#[derive(Clone, Deserialize, Debug, Serialize, PartialEq, Eq, Default)]
pub struct CredentialDef {
    #[serde(alias = "cred_def_id")]
    id: String,
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

#[derive(Clone, Debug, Deserialize, Serialize, Builder, Default)]
#[builder(setter(into), default)]
pub struct CredentialDefConfig {
    issuer_did: Did,
    schema_id: SchemaId,
    tag: String,
}

#[derive(Clone, Deserialize, Debug, Serialize, Builder, Default)]
#[builder(setter(into, strip_option), default)]
pub struct RevocationDetails {
    pub support_revocation: Option<bool>,
    pub tails_dir: Option<String>,
    pub max_creds: Option<u32>,
}

async fn _try_get_cred_def_from_ledger(
    ledger: &impl AnoncredsLedgerRead,
    issuer_did: &Did,
    cred_def_id: &str,
) -> VcxResult<Option<String>> {
    match ledger.get_cred_def(cred_def_id, Some(issuer_did)).await {
        Ok(cred_def) => Ok(Some(cred_def)),
        Err(err) if err.kind() == AriesVcxCoreErrorKind::LedgerItemNotFound => Ok(None),
        Err(err) => Err(AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidLedgerResponse,
            format!(
                "Failed to check presence of credential definition id {} on the ledger\nError: {}",
                cred_def_id, err
            ),
        )),
    }
}
impl CredentialDef {
    pub async fn create(
        wallet: &impl BaseWallet,
        ledger_read: &impl AnoncredsLedgerRead,
        anoncreds: &impl BaseAnonCreds,
        source_id: String,
        config: CredentialDefConfig,
        support_revocation: bool,
    ) -> VcxResult<Self> {
        trace!(
            "CredentialDef::create >>> source_id: {}, config: {:?}",
            source_id,
            config
        );
        let CredentialDefConfig {
            issuer_did,
            schema_id,
            tag,
        } = config;
        let schema_json = ledger_read
            .get_schema(&schema_id, Some(&issuer_did))
            .await?;
        let (cred_def_id, cred_def_json) = generate_cred_def(
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
            id: cred_def_id,
            cred_def_json,
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

    pub fn get_cred_def_json(&self) -> &str {
        &self.cred_def_json
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
        ledger_write
            .publish_cred_def(wallet, &self.cred_def_json, &self.issuer_did)
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

    pub fn get_cred_def_id(&self) -> String {
        self.id.clone()
    }

    pub fn get_schema_id(&self) -> String {
        self.schema_id.to_string()
    }

    pub fn set_source_id(&mut self, source_id: String) {
        self.source_id = source_id;
    }

    pub async fn update_state(&mut self, ledger: &impl AnoncredsLedgerRead) -> VcxResult<u32> {
        if (ledger.get_cred_def(&self.id, None).await).is_ok() {
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
    sig_type: Option<&str>,
    support_revocation: Option<bool>,
) -> VcxResult<(String, String)> {
    trace!(
        "generate_cred_def >>> issuer_did: {}, schema_json: {:?}, tag: {}, sig_type: {:?}, \
         support_revocation: {:?}",
        issuer_did,
        schema_json,
        tag,
        sig_type,
        support_revocation
    );

    let config_json =
        json!({"support_revocation": support_revocation.unwrap_or(false)}).to_string();

    anoncreds
        .issuer_create_and_store_credential_def(
            wallet,
            issuer_did,
            &schema_id.to_string(),
            schema_json,
            tag,
            sig_type,
            &config_json,
        )
        .await
        .map_err(|err| err.into())
}
