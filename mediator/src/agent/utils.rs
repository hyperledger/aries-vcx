use std::sync::Arc;

use aries_vcx::{common::signing::sign_connection_response, errors::error::VcxResult, transport::Transport};
use aries_vcx_core::wallet::base_wallet::BaseWallet;
use axum::async_trait;
use diddoc_legacy::aries::diddoc::AriesDidDoc;
use messages::{
    decorators::{thread::Thread, timing::Timing},
    msg_fields::protocols::{
        connection::{
            response::{Response, ResponseContent, ResponseDecorators},
            ConnectionData,
        },
        out_of_band::invitation::{Invitation as OOBInvitation, OobService},
    },
};
use uuid::Uuid;

use crate::utils::structs::VeriKey;

pub async fn build_response_content(
    wallet: &Arc<dyn BaseWallet>,
    thread_id: String,
    old_recipient_vk: VeriKey,
    new_recipient_did: String,
    new_recipient_vk: VeriKey,
    new_service_endpoint: url::Url,
    new_routing_keys: Vec<String>,
) -> VcxResult<Response> {
    let mut did_doc = AriesDidDoc::default();
    let did = new_recipient_did.clone();

    did_doc.set_id(new_recipient_did);
    did_doc.set_service_endpoint(new_service_endpoint);
    did_doc.set_routing_keys(new_routing_keys);
    did_doc.set_recipient_keys(vec![new_recipient_vk]);

    let con_data = ConnectionData::new(did, did_doc);

    let id = Uuid::new_v4().to_string();

    let con_sig = sign_connection_response(wallet, &old_recipient_vk, &con_data).await?;

    let content = ResponseContent::builder().connection_sig(con_sig).build();

    let decorators = ResponseDecorators::builder()
        .thread(Thread::builder().thid(thread_id).build())
        .build();

    Ok(Response::builder()
        .id(id)
        .content(content)
        .decorators(decorators)
        .build())
}

pub fn oob2did(oob: OOBInvitation) -> AriesDidDoc {
    let mut did_doc: AriesDidDoc = AriesDidDoc::default();
    did_doc.set_id(oob.id.clone());
    let oob_service = oob.content.services.first().expect("OOB needs a service").clone();

    match oob_service {
        OobService::AriesService(service) => {
            did_doc.set_service_endpoint(service.service_endpoint);
            did_doc.set_recipient_keys(service.recipient_keys);
            did_doc.set_routing_keys(service.routing_keys);
        }
        _ => panic!("Assuming fully clean AriesService variant only"),
    }
    did_doc
}

pub struct MockTransport;

#[async_trait]
impl Transport for MockTransport {
    async fn send_message(&self, _msg: Vec<u8>, _service_endpoint: url::Url) -> VcxResult<()> {
        Ok(())
    }
}
