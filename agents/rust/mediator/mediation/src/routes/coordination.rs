// Copyright 2023 Naian G.
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use axum::{extract::State, Json};

use crate::{
    didcomm_types::mediator_coord_structs::{MediatorCoordMsgEnum::*, *},
    storage::MediatorPersistence,
};

pub async fn handle_coord_authenticated<T: MediatorPersistence>(
    State(storage): State<Arc<T>>,
    Json(message): Json<MediatorCoordMsgEnum>,
    auth_pubkey: &str,
) -> Json<MediatorCoordMsgEnum> {
    match message {
        MediateRequest => {
            panic!(
                "Use handle_mediate_request directly. This handler is for preregistered clients."
            );
            // handle_mediate_request(
            //     storage,
            //     auth_pubkey,
            //     "{}",
            //     "",
            //     MediateGrantData {
            //         endpoint: "".to_owned(),
            //         routing_keys: vec![],
            //     },
            // )
            // .await
        }
        KeylistUpdateRequest(keylist_update_data) => {
            handle_keylist_update(storage, keylist_update_data, auth_pubkey).await
        }
        KeylistQuery(keylist_query_data) => {
            handle_keylist_query(storage, keylist_query_data, auth_pubkey).await
        }
        _ => handle_unimplemented().await,
    }
}

pub async fn handle_unimplemented() -> Json<MediatorCoordMsgEnum> {
    Json(MediatorCoordMsgEnum::XumErrorMsg {
        error: "Unimplemented".to_owned(),
    })
}

pub async fn handle_mediate_request<T: MediatorPersistence>(
    storage: Arc<T>,
    auth_pubkey: &str,
    did_doc: &str,
    our_signing_key: &str,
    grant_data: MediateGrantData,
) -> Json<MediatorCoordMsgEnum> {
    match storage
        .create_account(auth_pubkey, our_signing_key, did_doc)
        .await
    {
        Ok(()) => Json(MediateGrant(grant_data)),
        Err(msg) => Json(MediateDeny(MediateDenyData { reason: msg })),
    }
}

pub async fn handle_keylist_query<T: MediatorPersistence>(
    storage: Arc<T>,
    //todo: use the limits mentioned in the KeylistQueryData to modify response
    _keylist_query_data: KeylistQueryData,
    auth_pubkey: &str,
) -> Json<MediatorCoordMsgEnum> {
    let keylist_items: Vec<KeylistItem> = match storage.list_recipient_keys(auth_pubkey).await {
        Ok(recipient_keys) => recipient_keys
            .into_iter()
            .map(|recipient_key| KeylistItem { recipient_key })
            .collect(),
        Err(err) => return Json(MediatorCoordMsgEnum::XumErrorMsg { error: err }),
    };
    Json(MediatorCoordMsgEnum::Keylist(KeylistData {
        keys: keylist_items,
    }))
}

pub async fn handle_keylist_update<T: MediatorPersistence>(
    storage: Arc<T>,
    keylist_update_data: KeylistUpdateRequestData,
    auth_pubkey: &str,
) -> Json<MediatorCoordMsgEnum> {
    let updates: Vec<KeylistUpdateItem> = keylist_update_data.updates;
    let mut updated: Vec<KeylistUpdateItem> = Vec::new();
    for update_item in updates.into_iter() {
        let result = match &update_item.action {
            KeylistUpdateItemAction::Add => {
                storage
                    .add_recipient(auth_pubkey, &update_item.recipient_key)
                    .await
            }
            KeylistUpdateItemAction::Remove => {
                storage
                    .remove_recipient(auth_pubkey, &update_item.recipient_key)
                    .await
            }
        };
        let update_item_result = match result {
            Ok(()) => KeylistUpdateItemResult::Success,
            Err(_msg) => KeylistUpdateItemResult::ServerError,
        };
        updated.push(KeylistUpdateItem {
            recipient_key: update_item.recipient_key,
            action: update_item.action,
            result: Some(update_item_result),
        });
    }
    Json(MediatorCoordMsgEnum::KeylistUpdateResponse(
        KeylistUpdateResponseData { updated },
    ))
}
