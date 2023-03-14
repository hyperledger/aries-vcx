use std::sync::Arc;

use super::credential_definition::PublicEntityStateType;
use crate::{
    core::profile::profile::Profile,
    errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult},
    global::settings,
    utils::constants::REV_REG_ID,
};

#[derive(Clone, Deserialize, Debug, Serialize, PartialEq, Eq)]
pub struct RevocationRegistry {
    cred_def_id: String,
    issuer_did: String,
    pub rev_reg_id: String,
    pub(in crate::common) rev_reg_def: RevocationRegistryDefinition,
    pub(in crate::common) rev_reg_entry: String,
    pub(in crate::common) tails_dir: String,
    pub(in crate::common) max_creds: u32,
    pub(in crate::common) tag: u32,
    rev_reg_def_state: PublicEntityStateType,
    rev_reg_delta_state: PublicEntityStateType,
}

impl RevocationRegistry {
    pub async fn create(
        profile: &Arc<dyn Profile>,
        issuer_did: &str,
        cred_def_id: &str,
        tails_dir: &str,
        max_creds: u32,
        tag: u32,
    ) -> VcxResult<RevocationRegistry> {
        trace!(
            "RevocationRegistry::create >>> issuer_did: {}, cred_def_id: {}, tails_dir: {}, max_creds: {}, tag_no: {}",
            issuer_did,
            cred_def_id,
            tails_dir,
            max_creds,
            tag
        );
        let (rev_reg_id, rev_reg_def, rev_reg_entry) = generate_rev_reg(
            profile,
            issuer_did,
            cred_def_id,
            tails_dir,
            max_creds,
            &format!("tag{}", tag),
        )
        .await
        .map_err(|err| err.map(AriesVcxErrorKind::CreateRevRegDef, "Cannot create Revocation Registry"))?;
        Ok(RevocationRegistry {
            cred_def_id: cred_def_id.to_string(),
            issuer_did: issuer_did.to_string(),
            rev_reg_id,
            rev_reg_def,
            rev_reg_entry,
            tails_dir: tails_dir.to_string(),
            max_creds,
            tag,
            rev_reg_def_state: PublicEntityStateType::Built,
            rev_reg_delta_state: PublicEntityStateType::Built,
        })
    }

    pub fn get_rev_reg_id(&self) -> String {
        self.rev_reg_id.clone()
    }

    pub fn get_cred_def_id(&self) -> String {
        self.cred_def_id.clone()
    }

    pub fn get_rev_reg_def(&self) -> RevocationRegistryDefinition {
        self.rev_reg_def.clone()
    }

    pub fn get_tails_dir(&self) -> String {
        self.tails_dir.clone()
    }

    pub fn was_rev_reg_def_published(&self) -> bool {
        self.rev_reg_def_state == PublicEntityStateType::Published
    }

    pub fn was_rev_reg_delta_published(&self) -> bool {
        self.rev_reg_delta_state == PublicEntityStateType::Published
    }

    pub async fn publish_rev_reg_def(
        &mut self,
        profile: &Arc<dyn Profile>,
        issuer_did: &str,
        tails_url: &str,
    ) -> VcxResult<()> {
        trace!(
            "RevocationRegistry::publish_rev_reg_def >>> issuer_did:{}, rev_reg_id: {}, rev_reg_def:{:?}",
            issuer_did,
            &self.rev_reg_id,
            &self.rev_reg_def
        );
        self.rev_reg_def.value.tails_location = String::from(tails_url);
        let ledger = Arc::clone(profile).inject_ledger();
        ledger
            .publish_rev_reg_def(&self.rev_reg_def, issuer_did)
            .await
            .map_err(|err| {
                err.map(
                    AriesVcxErrorKind::InvalidState,
                    "Cannot publish revocation registry definition",
                )
            })?;
        self.rev_reg_def_state = PublicEntityStateType::Published;
        Ok(())
    }

    pub async fn publish_rev_reg_delta(&mut self, profile: &Arc<dyn Profile>, issuer_did: &str) -> VcxResult<()> {
        trace!(
            "RevocationRegistry::publish_rev_reg_delta >>> issuer_did:{}, rev_reg_id: {}",
            issuer_did,
            self.rev_reg_id
        );
        let ledger = Arc::clone(profile).inject_ledger();
        ledger
            .publish_rev_reg_delta(&self.rev_reg_id, &self.rev_reg_entry, issuer_did)
            .await
            .map_err(|err| err.map(AriesVcxErrorKind::InvalidRevocationEntry, "Cannot post RevocationEntry"))?;
        self.rev_reg_delta_state = PublicEntityStateType::Published;
        Ok(())
    }

    pub async fn publish_revocation_primitives(
        &mut self,
        profile: &Arc<dyn Profile>,
        tails_url: &str,
    ) -> VcxResult<()> {
        trace!(
            "RevocationRegistry::publish_revocation_primitives >>> tails_url: {}",
            tails_url
        );
        self.publish_built_rev_reg_def(profile, tails_url).await?;
        self.publish_built_rev_reg_delta(profile).await
    }

    async fn publish_built_rev_reg_delta(&mut self, profile: &Arc<dyn Profile>) -> VcxResult<()> {
        let issuer_did = &self.issuer_did.clone();
        if self.was_rev_reg_delta_published() {
            info!("No unpublished revocation registry delta found, nothing to publish")
        } else {
            self.publish_rev_reg_delta(profile, issuer_did).await?;
        }
        Ok(())
    }

    async fn publish_built_rev_reg_def(&mut self, profile: &Arc<dyn Profile>, tails_url: &str) -> VcxResult<()> {
        let issuer_did = &self.issuer_did.clone();
        if self.was_rev_reg_def_published() {
            info!("No unpublished revocation registry definition found, nothing to publish")
        } else {
            self.publish_rev_reg_def(profile, issuer_did, tails_url).await?;
        }
        Ok(())
    }

    pub fn to_string(&self) -> VcxResult<String> {
        serde_json::to_string(&self).map_err(|err| {
            AriesVcxError::from_msg(
                AriesVcxErrorKind::SerializationError,
                format!("Cannot serialize revocation registry: {:?}", err),
            )
        })
    }

    pub fn from_string(rev_reg_data: &str) -> VcxResult<Self> {
        serde_json::from_str(rev_reg_data).map_err(|err| {
            AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidJson,
                format!("Cannot deserialize revocation registry: {:?}", err),
            )
        })
    }

    pub async fn revoke_credential_local(&self, profile: &Arc<dyn Profile>, cred_rev_id: &str) -> VcxResult<()> {
        let anoncreds = Arc::clone(profile).inject_anoncreds();
        anoncreds
            .revoke_credential_local(&self.tails_dir, &self.rev_reg_id, cred_rev_id)
            .await
    }

    pub async fn publish_local_revocations(&self, profile: &Arc<dyn Profile>, submitter_did: &str) -> VcxResult<()> {
        let anoncreds = Arc::clone(profile).inject_anoncreds();

        anoncreds
            .publish_local_revocations(submitter_did, &self.rev_reg_id)
            .await
    }
}

#[derive(Clone, Deserialize, Debug, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct RevocationRegistryDefinitionValue {
    pub issuance_type: String,
    pub max_cred_num: u32,
    pub public_keys: serde_json::Value,
    pub tails_hash: String,
    pub tails_location: String,
}

#[derive(Clone, Deserialize, Debug, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct RevocationRegistryDefinition {
    pub id: String,
    pub revoc_def_type: String,
    pub tag: String,
    pub cred_def_id: String,
    pub value: RevocationRegistryDefinitionValue,
    pub ver: String,
}
pub async fn generate_rev_reg(
    profile: &Arc<dyn Profile>,
    issuer_did: &str,
    cred_def_id: &str,
    tails_dir: &str,
    max_creds: u32,
    tag: &str,
) -> VcxResult<(String, RevocationRegistryDefinition, String)> {
    trace!(
        "generate_rev_reg >>> issuer_did: {}, cred_def_id: {}, tails_file: {}, max_creds: {}, tag: {}",
        issuer_did,
        cred_def_id,
        tails_dir,
        max_creds,
        tag
    );
    if settings::indy_mocks_enabled() {
        debug!("generate_rev_reg >>> returning mocked value");
        return Ok((
            REV_REG_ID.to_string(),
            RevocationRegistryDefinition::default(),
            "".to_string(),
        ));
    }

    let anoncreds = Arc::clone(profile).inject_anoncreds();

    let (rev_reg_id, rev_reg_def_json, rev_reg_entry_json) = anoncreds
        .issuer_create_and_store_revoc_reg(issuer_did, cred_def_id, tails_dir, max_creds, tag)
        .await?;

    let rev_reg_def: RevocationRegistryDefinition = serde_json::from_str(&rev_reg_def_json).map_err(|err| {
        AriesVcxError::from_msg(
            AriesVcxErrorKind::SerializationError,
            format!(
                "Failed to deserialize rev_reg_def: {:?}, error: {:?}",
                rev_reg_def_json, err
            ),
        )
    })?;

    Ok((rev_reg_id, rev_reg_def, rev_reg_entry_json))
}

// consider impl revoke_credential_local in a generic (non-vdrtools) fashion
// consider impl publish_local_revocations in a generic (non-vdrtools) fashion
