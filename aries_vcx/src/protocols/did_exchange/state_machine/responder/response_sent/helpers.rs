use std::sync::Arc;

use did_doc_sov::DidDocumentSov;
use did_resolver_registry::ResolverRegistry;
use messages::{
    decorators::{attachment::Attachment, thread::Thread},
    msg_fields::protocols::did_exchange::{
        request::Request,
        response::{Response, ResponseContent, ResponseDecorators},
    },
};

use crate::{
    errors::error::AriesVcxError,
    protocols::did_exchange::state_machine::helpers::{attach_to_ddo_sov, ddo_sov_to_attach},
};

pub async fn resolve_their_ddo(
    resolver_registry: &Arc<ResolverRegistry>,
    request: &Request,
) -> Result<DidDocumentSov, AriesVcxError> {
    Ok(request
        .content
        .did_doc
        .clone()
        .map(attach_to_ddo_sov)
        .transpose()?
        .unwrap_or(
            resolver_registry
                .resolve(&request.content.did.parse()?, &Default::default())
                .await?
                .did_document()
                .to_owned()
                .into(),
        ))
}

// TODO: Replace by a builder
pub fn construct_response(
    our_did_document: DidDocumentSov,
    invitation_id: String,
    request_id: String,
    attachment: Option<Attachment>,
) -> Result<Response, AriesVcxError> {
    let content = ResponseContent {
        did: our_did_document.id().to_string(),
        did_doc: attachment,
    };
    let thread = {
        let mut thread = Thread::new(request_id.clone());
        thread.pthid = Some(invitation_id.clone());
        thread
    };
    let decorators = ResponseDecorators { thread, timing: None };
    Ok(Response::with_decorators(request_id, content, decorators))
}
