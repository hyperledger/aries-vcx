use std::path::Path;

use anoncreds_types::data_types::{
    identifiers::cred_def_id::CredentialDefinitionId,
    ledger::rev_reg_def::RevocationRegistryDefinition,
};
use aries_vcx_anoncreds::anoncreds::base_anoncreds::BaseAnonCreds;
use aries_vcx_ledger::ledger::base_ledger::{AnoncredsLedgerRead, AnoncredsLedgerWrite};
use aries_vcx_wallet::wallet::base_wallet::BaseWallet;
use did_parser_nom::Did;

use super::credential_definition::PublicEntityStateType;
use crate::errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult};

#[derive(Clone, Deserialize, Debug, Serialize)]
pub struct RevocationRegistry {
    cred_def_id: String,
    issuer_did: Did,
    pub rev_reg_id: String,
    rev_reg_def: RevocationRegistryDefinition,
    pub(in crate::common) rev_reg_entry: String,
    pub tails_dir: String,
    pub(in crate::common) max_creds: u32,
    pub(in crate::common) tag: u32,
    rev_reg_def_state: PublicEntityStateType,
    rev_reg_delta_state: PublicEntityStateType,
}

impl RevocationRegistry {
    pub async fn create(
        wallet: &impl BaseWallet,
        anoncreds: &impl BaseAnonCreds,
        issuer_did: &Did,
        cred_def_id: &CredentialDefinitionId,
        tails_dir: &str,
        max_creds: u32,
        tag: u32,
    ) -> VcxResult<RevocationRegistry> {
        trace!(
            "RevocationRegistry::create >>> issuer_did: {}, cred_def_id: {}, tails_dir: {}, \
             max_creds: {}, tag_no: {}",
            issuer_did,
            cred_def_id,
            tails_dir,
            max_creds,
            tag
        );
        let (rev_reg_id, rev_reg_def, rev_reg_entry) = generate_rev_reg(
            wallet,
            anoncreds,
            issuer_did,
            cred_def_id,
            tails_dir,
            max_creds,
            &format!("tag{}", tag),
        )
        .await
        .map_err(|err| {
            AriesVcxError::from_msg(
                AriesVcxErrorKind::SerializationError,
                format!(
                    "Failed to locally create a new Revocation Registry: {:?}",
                    err
                ),
            )
        })?;
        Ok(RevocationRegistry {
            cred_def_id: cred_def_id.to_string(),
            issuer_did: issuer_did.to_owned(),
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
        wallet: &impl BaseWallet,
        ledger: &impl AnoncredsLedgerWrite,
        issuer_did: &Did,
        tails_url: &str,
    ) -> VcxResult<()> {
        trace!(
            "RevocationRegistry::publish_rev_reg_def >>> issuer_did:{}, rev_reg_id: {}, \
             rev_reg_def:{:?}",
            issuer_did,
            &self.rev_reg_id,
            &self.rev_reg_def
        );
        self.rev_reg_def.value.tails_location = String::from(tails_url);
        ledger
            .publish_rev_reg_def(
                wallet,
                serde_json::from_str(&serde_json::to_string(&self.rev_reg_def)?)?,
                issuer_did,
            )
            .await
            .map_err(|err| {
                AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidState,
                    format!("Cannot publish revocation registry definition; {err}"),
                )
            })?;
        self.rev_reg_def_state = PublicEntityStateType::Published;
        Ok(())
    }

    pub async fn publish_rev_reg_delta(
        &mut self,
        wallet: &impl BaseWallet,
        ledger_write: &impl AnoncredsLedgerWrite,
        issuer_did: &Did,
    ) -> VcxResult<()> {
        trace!(
            "RevocationRegistry::publish_rev_reg_delta >>> issuer_did:{}, rev_reg_id: {}",
            issuer_did,
            self.rev_reg_id
        );
        ledger_write
            .publish_rev_reg_delta(
                wallet,
                &self.rev_reg_id.to_string().try_into()?,
                serde_json::from_str(&self.rev_reg_entry)?,
                issuer_did,
            )
            .await
            .map_err(|err| {
                AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidRevocationEntry,
                    format!("Cannot publish revocation entry; {err}"),
                )
            })?;
        self.rev_reg_delta_state = PublicEntityStateType::Published;
        Ok(())
    }

    pub async fn publish_revocation_primitives(
        &mut self,
        wallet: &impl BaseWallet,
        ledger_write: &impl AnoncredsLedgerWrite,
        tails_url: &str,
    ) -> VcxResult<()> {
        trace!(
            "RevocationRegistry::publish_revocation_primitives >>> tails_url: {}",
            tails_url
        );
        self.publish_built_rev_reg_def(wallet, ledger_write, tails_url)
            .await?;
        self.publish_built_rev_reg_delta(wallet, ledger_write).await
    }

    async fn publish_built_rev_reg_delta(
        &mut self,
        wallet: &impl BaseWallet,
        ledger_write: &impl AnoncredsLedgerWrite,
    ) -> VcxResult<()> {
        let issuer_did = &self.issuer_did.clone();
        if self.was_rev_reg_delta_published() {
            info!("No unpublished revocation registry delta found, nothing to publish")
        } else {
            self.publish_rev_reg_delta(wallet, ledger_write, issuer_did)
                .await?;
        }
        Ok(())
    }

    async fn publish_built_rev_reg_def(
        &mut self,
        wallet: &impl BaseWallet,
        ledger_write: &impl AnoncredsLedgerWrite,
        tails_url: &str,
    ) -> VcxResult<()> {
        let issuer_did = &self.issuer_did.clone();
        if self.was_rev_reg_def_published() {
            info!("No unpublished revocation registry definition found, nothing to publish")
        } else {
            self.publish_rev_reg_def(wallet, ledger_write, issuer_did, tails_url)
                .await?;
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

    pub async fn revoke_credential_local(
        &self,
        wallet: &impl BaseWallet,
        anoncreds: &impl BaseAnonCreds,
        ledger: &impl AnoncredsLedgerRead,
        cred_rev_id: u32,
    ) -> VcxResult<()> {
        #[allow(deprecated)] // TODO - https://github.com/hyperledger/aries-vcx/issues/1309
        let rev_reg_delta_json = ledger
            .get_rev_reg_delta_json(&self.rev_reg_id.to_string().try_into()?, None, None)
            .await?
            .0;
        anoncreds
            .revoke_credential_local(
                wallet,
                &self.rev_reg_id.to_owned().try_into()?,
                cred_rev_id,
                rev_reg_delta_json,
            )
            .await
            .map_err(|err| err.into())
    }

    pub async fn publish_local_revocations(
        &self,
        wallet: &impl BaseWallet,
        anoncreds: &impl BaseAnonCreds,
        ledger_write: &impl AnoncredsLedgerWrite,
        submitter_did: &Did,
    ) -> VcxResult<()> {
        if let Some(delta) = anoncreds
            .get_rev_reg_delta(wallet, &self.rev_reg_id.to_owned().try_into()?)
            .await?
        {
            ledger_write
                .publish_rev_reg_delta(
                    wallet,
                    &self.rev_reg_id.to_string().try_into()?,
                    delta,
                    submitter_did,
                )
                .await?;

            info!(
                "publish_local_revocations >>> rev_reg_delta published for rev_reg_id {}",
                self.rev_reg_id
            );

            match anoncreds
                .clear_rev_reg_delta(wallet, &self.rev_reg_id.to_owned().try_into()?)
                .await
            {
                Ok(_) => {
                    info!(
                        "publish_local_revocations >>> rev_reg_delta storage cleared for \
                         rev_reg_id {}",
                        self.rev_reg_id
                    );
                    Ok(())
                }
                Err(err) => Err(AriesVcxError::from_msg(
                    AriesVcxErrorKind::RevDeltaFailedToClear,
                    format!(
                        "Failed to clear revocation delta storage for rev_reg_id: {}, error: {err}",
                        self.rev_reg_id
                    ),
                )),
            }
        } else {
            Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::RevDeltaNotFound,
                format!(
                    "Failed to publish revocation delta for revocation registry {}, no delta \
                     found. Possibly already published?",
                    self.rev_reg_id
                ),
            ))
        }
    }
}

pub async fn generate_rev_reg(
    wallet: &impl BaseWallet,
    anoncreds: &impl BaseAnonCreds,
    issuer_did: &Did,
    cred_def_id: &CredentialDefinitionId,
    tails_dir: &str,
    max_creds: u32,
    tag: &str,
) -> VcxResult<(String, RevocationRegistryDefinition, String)> {
    trace!(
        "generate_rev_reg >>> issuer_did: {}, cred_def_id: {}, tails_file: {}, max_creds: {}, \
         tag: {}",
        issuer_did,
        cred_def_id,
        tails_dir,
        max_creds,
        tag
    );

    let (rev_reg_id, rev_reg_def_json, rev_reg_entry_json) = anoncreds
        .issuer_create_and_store_revoc_reg(
            wallet,
            issuer_did,
            cred_def_id,
            Path::new(tails_dir),
            max_creds,
            tag,
        )
        .await?;

    Ok((
        rev_reg_id.to_string(),
        rev_reg_def_json,
        serde_json::to_string(&rev_reg_entry_json)?,
    ))
}
