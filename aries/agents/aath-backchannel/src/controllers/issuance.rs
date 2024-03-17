use std::sync::RwLock;

use actix_web::{get, post, web, Responder};
use anoncreds_types::data_types::identifiers::cred_def_id::CredentialDefinitionId;
use aries_vcx_agent::aries_vcx::{
    handlers::util::OfferInfo,
    messages::msg_fields::protocols::cred_issuance::v1::{
        propose_credential::{ProposeCredentialV1, ProposeCredentialV1Content},
        CredentialPreviewV1 as VcxCredentialPreview, CredentialPreviewV1,
    },
    protocols::issuance::{holder::state_machine::HolderState, issuer::state_machine::IssuerState},
};
use display_as_json::Display;
use uuid::Uuid;

use crate::{
    controllers::Request,
    error::{HarnessError, HarnessErrorType, HarnessResult},
    soft_assert_eq, HarnessAgent, State,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
struct CredentialPreview(VcxCredentialPreview);

impl Default for CredentialPreview {
    fn default() -> Self {
        CredentialPreview(VcxCredentialPreview::new(vec![]))
    }
}

#[derive(Serialize, Deserialize, Default, Debug, Display)]
pub struct CredentialOffer {
    cred_def_id: String,
    credential_preview: CredentialPreview,
    connection_id: String,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, Display)]
pub struct CredentialProposal {
    schema_issuer_did: String,
    issuer_did: String,
    schema_name: String,
    cred_def_id: String,
    schema_version: String,
    credential_proposal: CredentialPreview,
    connection_id: String,
    schema_id: String,
}

#[derive(Serialize, Deserialize, Default)]
pub struct Credential {
    credential_preview: CredentialPreview,
    #[serde(default)]
    comment: Option<String>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct CredentialId {
    credential_id: String,
}

fn to_backchannel_state_issuer(state: IssuerState) -> State {
    match state {
        IssuerState::Initial => State::Initial,
        IssuerState::ProposalReceived | IssuerState::OfferSet => State::ProposalReceived,
        // IssuerState::OfferSent => State::OfferSent, // todo: we used to track "io state" in vcx,
        // now we don't - will this fail some AATH cases?
        IssuerState::RequestReceived => State::RequestReceived,
        IssuerState::CredentialSet => State::CredentialSent,
        IssuerState::Finished => State::Done,
        IssuerState::Failed => State::Failure,
    }
}

fn to_backchannel_state_holder(state: HolderState) -> State {
    match state {
        HolderState::Initial => State::Initial,
        HolderState::ProposalSet => State::ProposalSent,
        HolderState::OfferReceived => State::OfferReceived,
        HolderState::RequestSet => State::RequestSent,
        HolderState::Finished => State::Done,
        HolderState::Failed => State::Failure,
    }
}

async fn download_tails_file(
    tails_base_url: &str,
    rev_reg_id: &str,
    tails_hash: &str,
) -> HarnessResult<()> {
    info!(
        "issuance::download_tails_file >> tails_base_url: {:?}, rev_reg_id: {:?}, tails_hash: {:?}",
        tails_base_url, rev_reg_id, tails_hash
    );
    let url = match tails_base_url.to_string().matches('/').count() {
        0 => format!("{}/{}", tails_base_url, rev_reg_id),
        1.. => tails_base_url.to_string(),
        _ => {
            return Err(HarnessError::from_msg(
                HarnessErrorType::InternalServerError,
                "Negative count",
            ))
        }
    };
    let client = reqwest::Client::new();
    let tails_folder_path = std::env::current_dir()
        .expect("Failed to obtain the current directory path")
        .join("resource")
        .join("tails");
    std::fs::create_dir_all(&tails_folder_path).map_err(|_| {
        HarnessError::from_msg(
            HarnessErrorType::InternalServerError,
            "Failed to create tails folder",
        )
    })?;
    let tails_file_path = tails_folder_path
        .join(tails_hash)
        .to_str()
        .ok_or(HarnessError::from_msg(
            HarnessErrorType::InternalServerError,
            "Failed to convert tails hash to str",
        ))?
        .to_string();
    let res = client.get(&url).send().await?;
    soft_assert_eq!(res.status(), reqwest::StatusCode::OK);
    std::fs::write(tails_file_path, res.bytes().await?)?;
    Ok(())
}

impl HarnessAgent {
    pub async fn send_credential_proposal(
        &self,
        cred_proposal: &CredentialProposal,
    ) -> HarnessResult<String> {
        info!("issuance::send_credential_proposal >>");
        let proposal_id = Uuid::new_v4().to_string();
        let attrs = cred_proposal.credential_proposal.0.attributes.clone();

        let content = ProposeCredentialV1Content::builder()
            .schema_id(cred_proposal.schema_id.clone())
            .cred_def_id(cred_proposal.cred_def_id.clone())
            .credential_proposal(CredentialPreviewV1::new(attrs.clone()))
            .build();
        let proposal_data = ProposeCredentialV1::builder()
            .id(proposal_id.clone())
            .content(content)
            .build();

        let id = self
            .aries_agent
            .holder()
            .send_credential_proposal(&cred_proposal.connection_id, proposal_data)
            .await?;
        let state = to_backchannel_state_holder(self.aries_agent.holder().get_state(&id)?);
        info!("issuance::send_credential_proposal << id: {:?}", id);

        // todo: we are not saving (or creating) the holder
        Ok(json!({ "state": state, "thread_id": id }).to_string())
    }

    pub async fn send_credential_request(&self, thread_id: &str) -> HarnessResult<String> {
        info!("issuance::send_credential_request >> id: {:?}", thread_id);
        self.aries_agent
            .holder()
            .send_credential_request(thread_id)
            .await?;
        let state = to_backchannel_state_holder(self.aries_agent.holder().get_state(thread_id)?);
        Ok(json!({ "state": state, "thread_id": thread_id }).to_string())
    }

    pub async fn send_credential_offer(
        &self,
        cred_offer: &CredentialOffer,
        thread_id: &str,
    ) -> HarnessResult<String> {
        info!(
            "issuance::send_credential_offer >> cred_offer: {}, thread_id: {}",
            cred_offer, thread_id
        );
        let get_tails_rev_id =
            |cred_def_id: &str| -> HarnessResult<(Option<String>, Option<String>)> {
                Ok(
                    if let Some(rev_reg_id) = self
                        .aries_agent
                        .rev_regs()
                        .find_by_cred_def_id(cred_def_id)?
                        .pop()
                    {
                        (
                            Some(self.aries_agent.rev_regs().get_tails_dir(&rev_reg_id)?),
                            Some(rev_reg_id),
                        )
                    } else {
                        (None, None)
                    },
                )
            };

        let connection_id = if cred_offer.connection_id.is_empty() {
            None
        } else {
            Some(cred_offer.connection_id.as_str())
        };
        let (offer_info, id) = if thread_id.is_empty() {
            let credential_preview =
                serde_json::to_string(&cred_offer.credential_preview.0.attributes)?;
            info!(
                "issuance::send_credential_offer >> no thread_id provided, this offer initiates \
                 new didcomm conversation, using credential_preview: {:?}",
                credential_preview
            );
            let (tails_file, rev_reg_id) = get_tails_rev_id(&cred_offer.cred_def_id)?;
            (
                OfferInfo {
                    credential_json: credential_preview,
                    cred_def_id: CredentialDefinitionId::new(&cred_offer.cred_def_id)?,
                    rev_reg_id,
                    tails_file,
                },
                None,
            )
        } else {
            let proposal = self.aries_agent.issuer().get_proposal(thread_id)?;
            info!(
                "issuance::send_credential_offer >> thread_id is available, this offer will be \
                 built based on previous proposal: {:?}",
                proposal
            );
            let (tails_file, rev_reg_id) = get_tails_rev_id(&proposal.content.cred_def_id)?;
            (
                OfferInfo {
                    credential_json: serde_json::to_string(
                        &proposal.content.credential_proposal.attributes,
                    )
                    .unwrap(),
                    cred_def_id: CredentialDefinitionId::new(&proposal.content.cred_def_id)?,
                    rev_reg_id,
                    tails_file,
                },
                Some(thread_id),
            )
        };
        info!(
            "issuance::send_credential_offer >> offer_info: {:?}",
            offer_info
        );
        let id = self
            .aries_agent
            .issuer()
            .send_credential_offer(id, connection_id, offer_info)
            .await?;
        let state = to_backchannel_state_issuer(self.aries_agent.issuer().get_state(&id)?);
        Ok(json!({ "state": state, "thread_id": id }).to_string())
    }

    pub async fn issue_credential(
        &self,
        id: &str,
        _credential: &Credential,
    ) -> HarnessResult<String> {
        info!("issuance::issue_credential >> id: {:?}", id);
        self.aries_agent.issuer().send_credential(id).await?;
        let state = to_backchannel_state_issuer(self.aries_agent.issuer().get_state(id)?);
        Ok(json!({ "state": state }).to_string())
    }

    pub async fn store_credential(&self, id: &str) -> HarnessResult<String> {
        info!("issuance::store_credential >> id: {:?}", id);
        let state = self.aries_agent.holder().get_state(id)?;
        if self.aries_agent.holder().is_revokable(id).await? {
            let rev_reg_id = self.aries_agent.holder().get_rev_reg_id(id).await?;
            let tails_hash = self.aries_agent.holder().get_tails_hash(id).await?;
            let tails_location = self.aries_agent.holder().get_tails_location(id).await?;
            download_tails_file(&tails_location, &rev_reg_id, &tails_hash).await?;
        };
        Ok(json!({ "state": to_backchannel_state_holder(state), "credential_id": id }).to_string())
    }

    pub async fn get_issuer_state(&self, id: &str) -> HarnessResult<String> {
        info!("issuance::get_issuer_state >> id: {:?}", id);
        let state = if self.aries_agent.issuer().exists_by_id(id) {
            to_backchannel_state_issuer(self.aries_agent.issuer().get_state(id)?)
        } else if self.aries_agent.holder().exists_by_id(id) {
            to_backchannel_state_holder(self.aries_agent.holder().get_state(id)?)
        } else {
            return Err(HarnessError::from_kind(HarnessErrorType::NotFoundError));
        };
        Ok(json!({ "state": state }).to_string())
    }

    pub async fn get_credential(&self, id: &str) -> HarnessResult<String> {
        Ok(json!({ "referent": id }).to_string())
    }
}

#[post("/send-proposal")]
pub async fn send_credential_proposal(
    req: web::Json<Request<CredentialProposal>>,
    agent: web::Data<RwLock<HarnessAgent>>,
) -> impl Responder {
    agent
        .read()
        .unwrap()
        .send_credential_proposal(&req.data)
        .await
}

#[post("/send-offer")]
pub async fn send_credential_offer(
    req: web::Json<Request<CredentialOffer>>,
    agent: web::Data<RwLock<HarnessAgent>>,
) -> impl Responder {
    agent
        .read()
        .unwrap()
        .send_credential_offer(&req.data, &req.id)
        .await
}

#[post("/send-request")]
pub async fn send_credential_request(
    req: web::Json<Request<String>>,
    agent: web::Data<RwLock<HarnessAgent>>,
) -> impl Responder {
    agent.read().unwrap().send_credential_request(&req.id).await
}

#[get("/{issuer_id}")]
pub async fn get_issuer_state(
    agent: web::Data<RwLock<HarnessAgent>>,
    path: web::Path<String>,
) -> impl Responder {
    agent
        .read()
        .unwrap()
        .get_issuer_state(&path.into_inner())
        .await
}

#[post("/issue")]
pub async fn issue_credential(
    req: web::Json<Request<Credential>>,
    agent: web::Data<RwLock<HarnessAgent>>,
) -> impl Responder {
    agent
        .read()
        .unwrap()
        .issue_credential(&req.id, &req.data)
        .await
}

#[post("/store")]
pub async fn store_credential(
    req: web::Json<Request<CredentialId>>,
    agent: web::Data<RwLock<HarnessAgent>>,
) -> impl Responder {
    agent.read().unwrap().store_credential(&req.id).await
}

#[get("/{cred_id}")]
pub async fn get_credential(
    agent: web::Data<RwLock<HarnessAgent>>,
    path: web::Path<String>,
) -> impl Responder {
    agent
        .read()
        .unwrap()
        .get_credential(&path.into_inner())
        .await
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/command/issue-credential")
            .service(send_credential_proposal)
            .service(send_credential_offer)
            .service(get_issuer_state)
            .service(send_credential_request)
            .service(issue_credential)
            .service(store_credential),
    )
    .service(web::scope("/command/credential").service(get_credential));
}
