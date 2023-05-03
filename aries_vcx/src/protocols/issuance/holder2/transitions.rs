use std::sync::Arc;

use messages::{
    decorators::thread::Thread,
    msg_fields::protocols::{
        cred_issuance::{
            ack::{AckCredential, AckCredentialContent},
            issue_credential::IssueCredential,
            offer_credential::OfferCredential,
            propose_credential::{ProposeCredential, ProposeCredentialContent, ProposeCredentialDecorators},
            request_credential::{RequestCredential, RequestCredentialContent, RequestCredentialDecorators},
        },
        notification::{AckDecorators, AckStatus},
    },
};

use crate::{
    core::profile::profile::Profile,
    errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult},
    global::settings,
    handlers::util::{make_attach_from_str, matches_thread_id, AttachmentId, get_attach_as_string},
    protocols::issuance::holder2::states::ack_prepared::AckPrepared,
    utils::uuid::uuid,
};

use super::{
    states::{
        failed::Failed, finished::Finished, offer_received::OfferReceived, proposal_prepared::ProposalPrepared,
        request_prepared::RequestPrepared, HolderReceiveCredentialNextState,
    },
    Holder,
};

impl Holder<ProposalPrepared> {
    pub fn create_from_proposal_data(proposal_data: ProposeCredentialContent) -> Self {
        let id = uuid();
        let decoratators = ProposeCredentialDecorators::default();
        let proposal_message = ProposeCredential::with_decorators(id.clone(), proposal_data, decoratators);

        let state = ProposalPrepared::new(proposal_message);

        Holder { thread_id: id, state }
    }

    pub async fn receive_offer(self, offer: OfferCredential) -> Result<Holder<OfferReceived>, Holder<Failed>> {
        let expected_thread_id = self.thread_id();
        let thread_matches = offer
            .decorators
            .thread
            .as_ref()
            .map(|thread| thread.thid == expected_thread_id)
            .unwrap_or(false);

        if !thread_matches {
            let error = AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidJson,
                format!(
                    "Cannot handle message {:?}: thread id does not match, expected {:?}",
                    offer,
                    self.thread_id()
                ),
            );
            return Err(Holder {
                thread_id: self.thread_id,
                state: Failed::from_error(error),
            });
        };

        let state = OfferReceived::new(offer);
        Ok(Holder {
            thread_id: self.thread_id,
            state,
        })
    }
}

impl Holder<OfferReceived> {
    pub fn create_from_offer(offer: OfferCredential) -> Self {
        let thread_id = offer
            .decorators
            .thread
            .as_ref()
            .map_or(offer.id.to_owned(), |t| t.thid.to_owned());

        let state = OfferReceived::new(offer);
        Holder { thread_id, state }
    }

    pub fn prepare_proposal(self, proposal_data: ProposeCredentialContent) -> Holder<ProposalPrepared> {
        let thread_id = self.thread_id;
        let id = uuid();
        let mut decoratators = ProposeCredentialDecorators::default();
        decoratators.thread = Some(Thread::new(thread_id.clone()));
        let proposal_message = ProposeCredential::with_decorators(id.clone(), proposal_data, decoratators);

        let state = ProposalPrepared::new(proposal_message);

        Holder { thread_id, state }
    }

    pub async fn prepare_request(
        self,
        profile: &Arc<dyn Profile>,
        prover_did: String,
    ) -> Result<Holder<RequestPrepared>, (Holder<OfferReceived>, AriesVcxError)> {
        let thread_id = &self.thread_id;
        let offer = &self.state.offer_message;
        match make_credential_request(profile, thread_id.to_owned(), prover_did, offer).await {
            Ok((request_msg, cred_request_metadata, cred_def)) => {
                let state = RequestPrepared::new(request_msg, cred_request_metadata, cred_def);
                let thread_id = self.thread_id;
                Ok(Holder { thread_id, state })
            }
            Err(e) => Err((self, e)),
        }
    }

    // TODO - this name some what implies that the action will be taken... - maybe `prepare_decline_offer`?
    pub fn decline_offer(self, comment: Option<String>) -> Holder<Failed> {
        Holder {
            thread_id: self.thread_id,
            state: Failed::from_other_reason(comment.unwrap_or_default()),
        }
    }
}

impl Holder<RequestPrepared> {
    pub async fn receive_issue_credential(
        self,
        profile: &Arc<dyn Profile>,
        credential: IssueCredential,
    ) -> Result<HolderReceiveCredentialNextState, (Holder<RequestPrepared>, AriesVcxError)> {
        let expected_thread_id = self.thread_id();
        let thread_matches = matches_thread_id!(credential, expected_thread_id);

        if !thread_matches {
            let error = AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidJson,
                format!(
                    "Cannot handle message {:?}: thread id does not match, expected {:?}",
                    credential,
                    self.thread_id()
                ),
            );
            // TODO - soft fail even tho it is a hard failure?!
            return Err((self, error));
        };

        // TODO - check thread_id
        let credential_request_metdata = &self.state.credential_request_metadata;
        let cred_def_json = &self.state.credential_definition;
        let thread_id = self.thread_id.clone();
        let (credential_id, revocation_registry_definition) =
            match store_credential(profile, &credential, credential_request_metdata, cred_def_json).await {
                Ok((credential_id, revocation_registry_definition)) => {
                    Ok((credential_id, revocation_registry_definition))
                }
                Err(e) => Err((self, e)),
            }?;

        if credential.decorators.please_ack.is_some() {
            let ack = build_credential_ack(&thread_id);
            let state = AckPrepared::new(ack);
            let holder = Holder { thread_id, state };
            return Ok(HolderReceiveCredentialNextState::AckPrepared(holder));
        }

        let state = Finished::new(credential_id);
        let holder = Holder { thread_id, state };
        Ok(HolderReceiveCredentialNextState::Finished(holder))
    }
}

async fn make_credential_request(
    profile: &Arc<dyn Profile>,
    thread_id: String,
    my_pw_did: String,
    offer: &OfferCredential,
) -> VcxResult<(RequestCredential, String, String)> {
    trace!(
        "Holder::_make_credential_request >>> my_pw_did: {:?}, offer: {:?}",
        my_pw_did,
        offer
    );

    let cred_offer = get_attach_as_string!(&offer.content.offers_attach);

    trace!("Parsed cred offer attachment: {}", cred_offer);
    let cred_def_id = parse_cred_def_id_from_cred_offer(&cred_offer)?;
    let (req, req_meta, _cred_def_id, cred_def_json) =
        create_credential_request(profile, &cred_def_id, &my_pw_did, &cred_offer).await?;
    trace!("Created cred def json: {}", cred_def_json);
    let credential_request_msg = build_credential_request_msg(req, &thread_id)?;

    Ok((credential_request_msg, req_meta, cred_def_json))
}

// TODO - idk where to put these functions

fn parse_cred_def_id_from_cred_offer(cred_offer: &str) -> VcxResult<String> {
    trace!(
        "Holder::parse_cred_def_id_from_cred_offer >>> cred_offer: {:?}",
        cred_offer
    );

    let parsed_offer: serde_json::Value = serde_json::from_str(cred_offer).map_err(|err| {
        AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidJson,
            format!("Invalid Credential Offer Json: {:?}", err),
        )
    })?;

    let cred_def_id = parsed_offer["cred_def_id"].as_str().ok_or_else(|| {
        AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidJson,
            "Invalid Credential Offer Json: cred_def_id not found",
        )
    })?;

    Ok(cred_def_id.to_string())
}

async fn create_credential_request(
    profile: &Arc<dyn Profile>,
    cred_def_id: &str,
    prover_did: &str,
    cred_offer: &str,
) -> VcxResult<(String, String, String, String)> {
    let ledger = Arc::clone(profile).inject_ledger();
    let anoncreds = Arc::clone(profile).inject_anoncreds();
    let cred_def_json = ledger.get_cred_def(cred_def_id, None).await?;

    let master_secret_id = settings::DEFAULT_LINK_SECRET_ALIAS;
    anoncreds
        .prover_create_credential_req(prover_did, cred_offer, &cred_def_json, master_secret_id)
        .await
        .map_err(|err| err.extend("Cannot create credential request"))
        .map(|(s1, s2)| (s1, s2, cred_def_id.to_string(), cred_def_json))
        .map_err(AriesVcxError::from)
}

fn build_credential_request_msg(credential_request_attach: String, thread_id: &str) -> VcxResult<RequestCredential> {
    let content = RequestCredentialContent::new(vec![make_attach_from_str!(
        &credential_request_attach,
        AttachmentId::CredentialRequest.as_ref().to_string()
    )]);

    let mut decorators = RequestCredentialDecorators::default();

    let thread = Thread::new(thread_id.to_owned());

    decorators.thread = Some(thread);

    Ok(RequestCredential::with_decorators(uuid(), content, decorators))
}

async fn store_credential(
    profile: &Arc<dyn Profile>,
    issue_credential_message: &IssueCredential,
    credential_request_metdata: &str,
    cred_def_json: &str,
) -> VcxResult<(String, Option<String>)> {
    trace!(
        "Holder::_store_credential >>> issue_credential_message: {:?}, credential_request_metdata: {}, cred_def_json: {}",
        issue_credential_message,
        credential_request_metdata,
        cred_def_json
    );
    let ledger = Arc::clone(profile).inject_ledger();
    let anoncreds = Arc::clone(profile).inject_anoncreds();

    let credential_json = get_attach_as_string!(&issue_credential_message.content.credentials_attach);

    let rev_reg_id = parse_rev_reg_id_from_credential(&credential_json)?;
    let rev_reg_def_json = if let Some(rev_reg_id) = rev_reg_id {
        let json = ledger.get_rev_reg_def_json(&rev_reg_id).await?;
        Some(json)
    } else {
        None
    };
    let cred_id = anoncreds
        .prover_store_credential(
            None,
            credential_request_metdata,
            &credential_json,
            cred_def_json,
            rev_reg_def_json.as_deref(),
        )
        .await?;
    Ok((cred_id, rev_reg_def_json))
}

fn parse_rev_reg_id_from_credential(credential: &str) -> VcxResult<Option<String>> {
    trace!("Holder::_parse_rev_reg_id_from_credential >>>");

    let parsed_credential: serde_json::Value = serde_json::from_str(credential).map_err(|err| {
        AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidJson,
            format!("Invalid Credential Json: {}, err: {:?}", credential, err),
        )
    })?;

    let rev_reg_id = parsed_credential["rev_reg_id"].as_str().map(String::from);
    trace!("Holder::_parse_rev_reg_id_from_credential <<< {:?}", rev_reg_id);

    Ok(rev_reg_id)
}

fn build_credential_ack(thread_id: &str) -> AckCredential {
    let content = AckCredentialContent::new(AckStatus::Ok);
    let decorators = AckDecorators::new(Thread::new(thread_id.to_owned()));

    AckCredential::with_decorators(uuid(), content, decorators)
}
