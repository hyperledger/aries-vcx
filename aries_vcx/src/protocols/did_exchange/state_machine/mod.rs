mod helpers;

pub mod generic;
pub mod requester;
pub mod responder;

use std::marker::PhantomData;

use aries_vcx_core::wallet::base_wallet::BaseWallet;
use chrono::Utc;
use did_doc_sov::{service::ServiceSov, DidDocumentSov};
pub use helpers::generate_keypair;
use messages::{
    decorators::{thread::Thread, timing::Timing},
    msg_fields::protocols::did_exchange::problem_report::{
        ProblemCode, ProblemReport, ProblemReportContent, ProblemReportDecorators,
    },
    AriesMessage,
};
use url::Url;
use uuid::Uuid;

use super::{
    states::{
        abandoned::Abandoned,
        traits::{InvitationId, ThreadId},
    },
    transition::transition_result::TransitionResult,
};
use crate::{
    errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult},
    transport::Transport,
    utils::{encryption_envelope::EncryptionEnvelope, from_did_doc_sov_to_legacy},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DidExchange<I, S> {
    state: S,
    initiation_type: PhantomData<I>,
    our_did_document: DidDocumentSov,
    their_did_document: DidDocumentSov,
}

impl<I, S: ThreadId> DidExchange<I, S> {
    pub fn get_thread_id(&self) -> &str {
        self.state.thread_id()
    }

    pub fn fail(
        self,
        reason: String,
        problem_code: Option<ProblemCode>,
    ) -> TransitionResult<DidExchange<I, Abandoned>, ProblemReport> {
        let content = ProblemReportContent::builder()
            .problem_code(problem_code)
            .explain(Some(reason.clone()))
            .build();
        let decorators = ProblemReportDecorators::builder()
            .thread(
                Thread::builder()
                    .thid(self.state.thread_id().to_string())
                    .build(),
            )
            .timing(Timing::builder().out_time(Utc::now()).build())
            .build();
        let problem_report = ProblemReport::builder()
            .id(Uuid::new_v4().to_string())
            .content(content)
            .decorators(decorators)
            .build();

        TransitionResult {
            state: DidExchange {
                state: Abandoned {
                    reason,
                    request_id: self.state.thread_id().to_string(),
                },
                initiation_type: PhantomData,
                our_did_document: self.our_did_document,
                their_did_document: self.their_did_document,
            },
            output: problem_report,
        }
    }

    pub fn receive_problem_report(
        self,
        problem_report: ProblemReport,
    ) -> DidExchange<I, Abandoned> {
        DidExchange {
            state: Abandoned {
                reason: problem_report.content.explain.unwrap_or_default(),
                request_id: self.state.thread_id().to_string(),
            },
            initiation_type: PhantomData,
            our_did_document: self.our_did_document,
            their_did_document: self.their_did_document,
        }
    }
}

impl<I, S: InvitationId> DidExchange<I, S> {
    pub fn get_invitation_id(&self) -> &str {
        self.state.invitation_id()
    }
}

impl<I, S> DidExchange<I, S> {
    pub fn from_parts(
        state: S,
        their_did_document: DidDocumentSov,
        our_did_document: DidDocumentSov,
    ) -> Self {
        Self {
            state,
            initiation_type: PhantomData,
            our_did_document,
            their_did_document,
        }
    }
}

impl<I, S> DidExchange<I, S> {
    pub fn our_did_doc(&self) -> &DidDocumentSov {
        &self.our_did_document
    }

    pub fn their_did_doc(&self) -> &DidDocumentSov {
        &self.their_did_document
    }

    pub fn get_endpoint(&self) -> Option<Url> {
        let did_document = self.their_did_doc();
        // todo: no unwrap pls
        let service = did_document.service().first()?;
        // todo: will get cleaned up after service is de-generified
        Some(match service {
            ServiceSov::Legacy(d) => d.service_endpoint().into(),
            ServiceSov::AIP1(d) => d.service_endpoint().into(),
            ServiceSov::DIDCommV1(d) => d.service_endpoint().into(),
            ServiceSov::DIDCommV2(d) => d.service_endpoint().into(),
        })
    }
    pub async fn encrypt_message(
        &self,
        wallet: &impl BaseWallet,
        message: &AriesMessage,
    ) -> VcxResult<EncryptionEnvelope> {
        let sender_verkey = &self
            .our_did_doc()
            .resolved_key_agreement()
            .next()
            .ok_or_else(|| {
                AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidState,
                    "No key agreement method found",
                )
            })?
            .public_key()?
            .base58();
        EncryptionEnvelope::create(
            wallet,
            json!(message).to_string().as_bytes(),
            Some(sender_verkey),
            &from_did_doc_sov_to_legacy(self.their_did_doc().clone())?,
        )
        .await
    }

    pub async fn send_message<T>(
        &self,
        wallet: &impl BaseWallet,
        message: &AriesMessage,
        transport: &T,
    ) -> VcxResult<()>
    where
        T: Transport,
    {
        let msg = self.encrypt_message(wallet, message).await?.0;
        let service_endpoint = self.get_endpoint().ok_or_else(|| {
            AriesVcxError::from_msg(AriesVcxErrorKind::InvalidUrl, "No URL in DID Doc")
        })?;
        transport.send_message(msg, service_endpoint.clone()).await
    }
}
