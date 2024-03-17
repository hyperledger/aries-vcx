use std::sync::RwLock;

use actix_web::{get, post, web, Responder};
use anoncreds_types::data_types::identifiers::{
    cred_def_id::CredentialDefinitionId, schema_id::SchemaId,
};
use aries_vcx_agent::aries_vcx::did_parser_nom::Did;
use reqwest::multipart;

use crate::{
    controllers::Request,
    error::{HarnessError, HarnessErrorType, HarnessResult},
    soft_assert_eq, HarnessAgent,
};

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct CredentialDefinition {
    support_revocation: bool,
    schema_id: String,
    tag: String,
}

async fn upload_tails_file(tails_url: &str, tails_file: &str) -> HarnessResult<()> {
    info!("Going to upload tails file {} to {}", tails_file, tails_url);
    let client = reqwest::Client::new();
    let genesis_file = std::env::var("GENESIS_FILE").unwrap_or(
        std::env::current_dir()
            .expect("Failed to obtain the current directory path")
            .join("resource")
            .join("genesis_file.txn")
            .to_str()
            .ok_or(HarnessError::from_msg(
                HarnessErrorType::InternalServerError,
                "Failed to convert path to str",
            ))?
            .to_string(),
    );
    let genesis_file_data = std::fs::read(genesis_file)?;
    let tails_file_data = std::fs::read(tails_file)?;
    let genesis_part = multipart::Part::bytes(genesis_file_data);
    let tails_part = multipart::Part::bytes(tails_file_data);
    let form = multipart::Form::new()
        .part("genesis", genesis_part)
        .part("tails", tails_part);
    let res = client.put(tails_url).multipart(form).send().await?;
    soft_assert_eq!(res.status(), reqwest::StatusCode::OK);
    Ok(())
}

impl HarnessAgent {
    pub async fn create_credential_definition(
        &self,
        cred_def: &CredentialDefinition,
    ) -> HarnessResult<String> {
        let tails_base_url = std::env::var("TAILS_SERVER_URL")
            .unwrap_or("https://tails-server-test.pathfinder.gov.bc.ca".to_string());
        let cred_def_ids = self
            .aries_agent
            .cred_defs()
            .find_by_schema_id(&cred_def.schema_id)?;
        let cred_def_id = if cred_def_ids.is_empty() {
            let cred_def_id = self
                .aries_agent
                .cred_defs()
                .create_cred_def(
                    Did::parse(self.aries_agent.issuer_did())?,
                    SchemaId::new(&cred_def.schema_id)?,
                    cred_def.tag.clone(),
                )
                .await?;
            self.aries_agent
                .cred_defs()
                .publish_cred_def(&cred_def_id)
                .await?;

            if cred_def.support_revocation {
                let rev_reg_id = self
                    .aries_agent
                    .rev_regs()
                    .create_rev_reg(&CredentialDefinitionId::new(cred_def_id.clone())?, 50)
                    .await?;
                let tails_url = format!("{}/{}", tails_base_url, rev_reg_id);
                self.aries_agent
                    .rev_regs()
                    .publish_rev_reg(&rev_reg_id, &tails_url)
                    .await?;

                let tails_file = self.aries_agent.rev_regs().tails_file_path(&rev_reg_id)?;
                upload_tails_file(&tails_url, &tails_file).await?;
            }
            cred_def_id
        } else if cred_def_ids.len() == 1 {
            cred_def_ids.last().unwrap().clone()
        } else {
            return Err(HarnessError::from_kind(
                HarnessErrorType::MultipleCredDefinitions,
            ));
        };
        Ok(json!({ "credential_definition_id": cred_def_id }).to_string())
    }

    pub fn get_credential_definition(&self, id: &str) -> HarnessResult<String> {
        self.aries_agent
            .cred_defs()
            .cred_def_json(id)
            .map_err(|err| err.into())
    }
}

#[post("")]
pub async fn create_credential_definition(
    req: web::Json<Request<CredentialDefinition>>,
    agent: web::Data<RwLock<HarnessAgent>>,
) -> impl Responder {
    agent
        .read()
        .unwrap()
        .create_credential_definition(&req.data)
        .await
}

#[get("/{cred_def_id}")]
pub async fn get_credential_definition(
    agent: web::Data<RwLock<HarnessAgent>>,
    path: web::Path<String>,
) -> impl Responder {
    agent
        .read()
        .unwrap()
        .get_credential_definition(&path.into_inner())
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/command/credential-definition")
            .service(create_credential_definition)
            .service(get_credential_definition),
    );
}
