use std::sync::Arc;

use aries_vcx_core::wallet::base_wallet::BaseWallet;
use chrono::Utc;
use diddoc_legacy::aries::diddoc::AriesDidDoc;
use messages::{
    decorators::{thread::Thread, timing::Timing},
    msg_fields::protocols::connection::{
        request::Request,
        response::{Response, ResponseContent, ResponseDecorators},
        ConnectionData,
    },
};
use url::Url;
use uuid::Uuid;

use crate::{
    common::signing::sign_connection_response,
    errors::error::VcxResult,
    new_protocols::{
        connection::{inviter::InviterConnection, ConnectionSM},
        AriesSM, StateMachineStorage,
    },
};

async fn build_response_content(
    verkey: &str,
    wallet: &Arc<dyn BaseWallet>,
    did: String,
    recipient_keys: Vec<String>,
    new_service_endpoint: Url,
    new_routing_keys: Vec<String>,
) -> VcxResult<ResponseContent> {
    let mut did_doc = AriesDidDoc::default();

    did_doc.set_id(did.clone());
    did_doc.set_service_endpoint(new_service_endpoint);
    did_doc.set_routing_keys(new_routing_keys);
    did_doc.set_recipient_keys(recipient_keys);

    let con_data = ConnectionData::new(did, did_doc);
    let con_sig = sign_connection_response(wallet.as_ref(), verkey, &con_data).await?;
    let content = ResponseContent::new(con_sig);

    Ok(content)
}

pub async fn handle_request<S>(
    sm_storage: S,
    sm_id: S::SmInfo,
    wallet: &Arc<dyn BaseWallet>,
    request: Request,
    invitation_verkey: &str,
    service_endpoint: Url,
    routing_keys: Vec<String>,
) -> VcxResult<Response>
where
    S: StateMachineStorage,
{
    // If the request's DidDoc validation fails, we generate and send a ProblemReport.
    // We then return early with the provided error.
    if let Err(err) = request.content.connection.did_doc.validate() {
        error!("Request DidDoc validation failed! Sending ProblemReport...");
        // TODO: There is a problem report generated here
        Err(err)?;
    }

    // Generate new pairwise info that will be used from this point on
    // and incorporate that into the response.
    let (did, verkey) = wallet.create_and_store_my_did(None, None).await?;
    let thread_id = request.decorators.thread.map(|t| t.thid).unwrap_or(request.id);
    let did_doc = request.content.connection.did_doc;

    let content = build_response_content(
        invitation_verkey,
        wallet,
        did.clone(),
        vec![verkey.clone()],
        service_endpoint,
        routing_keys,
    )
    .await?;

    let id = Uuid::new_v4().to_string();

    let timing = Timing {
        out_time: Some(Utc::now()),
        ..Default::default()
    };

    let decorators = ResponseDecorators {
        thread: Thread::new(thread_id),
        please_ack: None,
        timing: Some(timing),
    };

    let response = Response::with_decorators(id, content, decorators);

    let sm = InviterConnection::new_inviter(did, verkey, did_doc);
    let sm = AriesSM::Connection(ConnectionSM::InviterComplete(sm));
    sm_storage.put_new_state(sm_id, sm).await?;

    Ok(response)
}
