use std::sync::Arc;

use aries_vcx::{common::signing::sign_connection_response, errors::error::VcxResult};
use aries_vcx_core::wallet::base_wallet::BaseWallet;
use diddoc_legacy::aries::diddoc::AriesDidDoc;
use messages::{
    decorators::{thread::Thread, timing::Timing},
    msg_fields::protocols::connection::{
        response::{Response, ResponseContent, ResponseDecorators},
        ConnectionData,
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
