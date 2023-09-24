use std::sync::Arc;

use aries_vcx_core::wallet::base_wallet::BaseWallet;
use chrono::Utc;
use diddoc_legacy::aries::diddoc::AriesDidDoc;
use messages::{
    decorators::{thread::Thread, timing::Timing},
    msg_fields::protocols::discover_features::{
        disclose::{Disclose, DiscloseContent, DiscloseDecorators},
        query::{Query, QueryContent, QueryDecorators},
        ProtocolDescriptor,
    },
};
use uuid::Uuid;

use crate::{errors::error::VcxResult, utils::send_message};

pub async fn send_discovery_query(
    wallet: &Arc<dyn BaseWallet>,
    query: Option<String>,
    comment: Option<String>,
    did_doc: &AriesDidDoc,
    pw_vk: &str,
) -> VcxResult<()> {
    let query = query.unwrap_or("*".to_owned());
    let content = QueryContent::builder().query(query);

    let content = if let Some(comment) = comment {
        content.comment(comment).build()
    } else {
        content.build()
    };

    let decorators = QueryDecorators::builder()
        .timing(Timing::builder().out_time(Utc::now()).build())
        .build();

    let query = Query::builder()
        .id(Uuid::new_v4().to_string())
        .content(content)
        .decorators(decorators)
        .build();

    send_message(
        Arc::clone(wallet),
        pw_vk.to_string(),
        did_doc.clone(),
        query,
    )
    .await
}

pub async fn respond_discovery_query(
    wallet: &Arc<dyn BaseWallet>,
    query: Query,
    did_doc: &AriesDidDoc,
    pw_vk: &str,
    _supported_protocols: Vec<ProtocolDescriptor>,
) -> VcxResult<()> {
    let content = DiscloseContent::default();

    let decorators = DiscloseDecorators::builder()
        .thread(Thread::builder().thid(query.id).build())
        .timing(Timing::builder().out_time(Utc::now()).build())
        .build();

    let disclose = Disclose::builder()
        .id(Uuid::new_v4().to_string())
        .content(content)
        .decorators(decorators)
        .build();

    send_message(
        Arc::clone(wallet),
        pw_vk.to_string(),
        did_doc.clone(),
        disclose,
    )
    .await
}
