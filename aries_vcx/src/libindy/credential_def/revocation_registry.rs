use crate::error::{VcxErrorKind, VcxResult};
use crate::libindy::credential_def::PublicEntityStateType;
use crate::libindy::utils::anoncreds;
use crate::libindy::utils::anoncreds::RevocationRegistryDefinition;

#[derive(Clone, Deserialize, Debug, Serialize, PartialEq)]
pub struct RevocationRegistry {
    pub rev_reg_id: String,
    pub(super) rev_reg_def: RevocationRegistryDefinition,
    pub(super) rev_reg_entry: String,
    pub(super) tails_dir: String,
    pub(super) max_creds: u32,
    pub(super) tag: u32,
    rev_reg_def_state: PublicEntityStateType,
    rev_reg_delta_state: PublicEntityStateType,
}


impl RevocationRegistry {
    pub async fn create(issuer_did: &str, cred_def_id: &str, tails_dir: &str, max_creds: u32, tag: u32) -> VcxResult<RevocationRegistry> {
        trace!("RevocationRegistry::create >>> issuer_did: {}, cred_def_id: {}, tails_dir: {}, max_creds: {}, tag_no: {}",  issuer_did, cred_def_id, tails_dir, max_creds, tag);
        let (rev_reg_id, rev_reg_def, rev_reg_entry) =
            anoncreds::generate_rev_reg(&issuer_did, &cred_def_id, tails_dir, max_creds, &format!("{}", tag))
                .await
                .map_err(|err| err.map(VcxErrorKind::CreateRevRegDef, "Cannot create Revocation Registry"))?;
        Ok(RevocationRegistry {
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

    pub fn was_rev_reg_def_published(&self) -> bool {
        self.rev_reg_def_state == PublicEntityStateType::Published
    }

    pub fn was_rev_reg_delta_published(&self) -> bool {
        self.rev_reg_delta_state == PublicEntityStateType::Published
    }

    pub async fn publish_rev_reg_def(&mut self, issuer_did: &str, tails_url: &str) -> VcxResult<()> {
        trace!("RevocationRegistry::publish_rev_reg_def >>> issuer_did:{}, rev_reg_id: {}, rev_reg_def:{:?}", issuer_did, &self.rev_reg_id, &self.rev_reg_def);
        self.rev_reg_def.value.tails_location = String::from(tails_url);
        anoncreds::publish_rev_reg_def(&issuer_did, &self.rev_reg_def)
            .await
            .map_err(|err| err.map(VcxErrorKind::InvalidState, "Cannot publish revocation registry definition"))?;
        self.rev_reg_def_state = PublicEntityStateType::Published;
        Ok(())
    }

    pub async fn publish_rev_reg_delta(&mut self, issuer_did: &str) -> VcxResult<()> {
        trace!("RevocationRegistry::publish_rev_reg_delta >>> issuer_did:{}, rev_reg_id: {}", issuer_did, self.rev_reg_id);
        anoncreds::publish_rev_reg_delta(issuer_did, &self.rev_reg_id, &self.rev_reg_entry)
            .await
            .map_err(|err| err.map(VcxErrorKind::InvalidRevocationEntry, "Cannot post RevocationEntry"))?;
        self.rev_reg_delta_state = PublicEntityStateType::Published;
        Ok(())
    }
}
