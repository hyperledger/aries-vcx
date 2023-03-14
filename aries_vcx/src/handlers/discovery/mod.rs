use std::sync::Arc;

use messages::{
    diddoc::aries::diddoc::AriesDidDoc,
    protocols::discovery::{
        disclose::{Disclose, ProtocolDescriptor},
        query::Query,
    },
};

use crate::{errors::error::VcxResult, plugins::wallet::base_wallet::BaseWallet, utils::send_message};

pub async fn send_discovery_query(
    wallet: &Arc<dyn BaseWallet>,
    query: Option<String>,
    comment: Option<String>,
    did_doc: &AriesDidDoc,
    pw_vk: &str,
) -> VcxResult<()> {
    let query_ = Query::create().set_query(query).set_comment(comment).set_out_time();
    send_message(
        Arc::clone(wallet),
        pw_vk.to_string(),
        did_doc.clone(),
        query_.to_a2a_message(),
    )
    .await
}

pub async fn respond_discovery_query(
    wallet: &Arc<dyn BaseWallet>,
    query: Query,
    did_doc: &AriesDidDoc,
    pw_vk: &str,
    supported_protocols: Vec<ProtocolDescriptor>,
) -> VcxResult<()> {
    let disclose = Disclose::create()
        .set_protocols(supported_protocols)
        .set_thread_id(&query.id.0.clone())
        .set_out_time();

    send_message(
        Arc::clone(wallet),
        pw_vk.to_string(),
        did_doc.clone(),
        disclose.to_a2a_message(),
    )
    .await
}
