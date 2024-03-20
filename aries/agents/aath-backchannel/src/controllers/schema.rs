use std::sync::RwLock;

use actix_web::{get, post, web, Responder};

use crate::{
    controllers::AathRequest,
    error::{HarnessError, HarnessErrorType, HarnessResult},
    soft_assert_eq, HarnessAgent,
};

#[derive(Serialize, Deserialize, Default)]
pub struct Schema {
    schema_name: String,
    schema_version: String,
    attributes: Vec<String>,
}

impl HarnessAgent {
    fn schema_id(&self, schema: &Schema) -> String {
        let did = self.aries_agent.issuer_did();
        let &Schema {
            schema_name,
            schema_version,
            ..
        } = &schema;
        format!("{}:2:{}:{}", did, schema_name, schema_version)
    }

    async fn schema_published(&self, id: &str) -> bool {
        self.aries_agent.schemas().schema_json(id).await.is_ok()
    }

    pub async fn create_schema(&self, schema: &Schema) -> HarnessResult<String> {
        let id = self.schema_id(schema);
        if !self.schema_published(&id).await {
            soft_assert_eq!(
                self.aries_agent
                    .schemas()
                    .create_schema(
                        &schema.schema_name,
                        &schema.schema_version,
                        schema.attributes.clone(),
                    )
                    .await?,
                id
            );
            self.aries_agent.schemas().publish_schema(&id).await?;
        };
        Ok(json!({ "schema_id": id }).to_string())
    }

    pub async fn get_schema(&self, id: &str) -> HarnessResult<String> {
        self.aries_agent
            .schemas()
            .schema_json(id)
            .await
            .map_err(|err| err.into())
    }
}

#[post("")]
pub async fn create_schema(
    req: web::Json<AathRequest<Schema>>,
    agent: web::Data<RwLock<HarnessAgent>>,
) -> impl Responder {
    agent.read().unwrap().create_schema(&req.data).await
}

#[get("/{schema_id}")]
pub async fn get_schema(
    agent: web::Data<RwLock<HarnessAgent>>,
    path: web::Path<String>,
) -> impl Responder {
    agent.read().unwrap().get_schema(&path.into_inner()).await
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/command/schema")
            .service(create_schema)
            .service(get_schema),
    );
}
