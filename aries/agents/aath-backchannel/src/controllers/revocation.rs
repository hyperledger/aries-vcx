use std::sync::RwLock;

use actix_web::{get, post, web, Responder};

use crate::{controllers::AathRequest, error::HarnessResult, HarnessAgent};

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct CredentialRevocationData {
    cred_rev_id: String,
    rev_registry_id: String,
    publish_immediately: bool,
    notify_connection_id: String,
}

impl HarnessAgent {
    pub async fn revoke_credential(
        &self,
        revocation_data: &CredentialRevocationData,
    ) -> HarnessResult<String> {
        let CredentialRevocationData {
            rev_registry_id,
            cred_rev_id,
            publish_immediately,
            ..
        } = revocation_data;
        self.aries_agent
            .rev_regs()
            .revoke_credential_locally(rev_registry_id, cred_rev_id)
            .await?;
        if *publish_immediately {
            self.aries_agent
                .rev_regs()
                .publish_local_revocations(rev_registry_id)
                .await?;
        };
        Ok("".to_string())
    }

    pub fn get_rev_reg_info_for_credential(&self, id: &str) -> HarnessResult<String> {
        let rev_reg_id = self.aries_agent.issuer().get_rev_reg_id(id)?;
        let rev_id = self.aries_agent.issuer().get_rev_id(id)?;
        Ok(json!({ "revoc_reg_id": rev_reg_id, "revocation_id": rev_id }).to_string())
    }
}

#[post("/revoke")]
pub async fn revoke_credential(
    agent: web::Data<RwLock<HarnessAgent>>,
    req: web::Json<AathRequest<CredentialRevocationData>>,
) -> impl Responder {
    agent.read().unwrap().revoke_credential(&req.data).await
}

#[get("/{cred_id}")]
pub async fn get_rev_reg_info_for_credential(
    agent: web::Data<RwLock<HarnessAgent>>,
    path: web::Path<String>,
) -> impl Responder {
    agent
        .read()
        .unwrap()
        .get_rev_reg_info_for_credential(&path.into_inner())
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/response/revocation-registry").service(get_rev_reg_info_for_credential),
    )
    .service(web::scope("/command/revocation").service(revoke_credential));
}
