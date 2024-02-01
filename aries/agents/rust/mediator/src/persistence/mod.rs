// Copyright 2023 Naian G.
// SPDX-License-Identifier: Apache-2.0

pub mod database;
pub mod errors;
use async_trait::async_trait;
/// Database backend is used for default implementation of MediatorPersistence trait
pub use database::get_db_pool as get_persistence;
use diddoc_legacy::aries::diddoc::AriesDidDoc;

use self::errors::{
    AddRecipientError, CreateAccountError, GetAccountDetailsError, GetAccountIdError,
    ListAccountsError, ListRecipientKeysError, PersistForwardMessageError, RemoveRecipientError,
    RetrievePendingMessageCountError, RetrievePendingMessagesError,
};
use crate::utils::structs::VerKey;

#[async_trait]
pub trait MediatorPersistence: Send + Sync + 'static {
    async fn create_account(
        &self,
        auth_pubkey: &str,
        our_signing_key: &str,
        did_doc: &str,
    ) -> Result<(), CreateAccountError>;
    async fn get_account_id(&self, auth_pubkey: &str) -> Result<Vec<u8>, GetAccountIdError>;
    // async fn vaporize_account(&self, auth_pubkey: String);
    async fn add_recipient(
        &self,
        auth_pubkey: &str,
        recipient_key: &str,
    ) -> Result<(), AddRecipientError>;
    async fn remove_recipient(
        &self,
        auth_pubkey: &str,
        recipient_key: &str,
    ) -> Result<(), RemoveRecipientError>;
    async fn list_recipient_keys(
        &self,
        auth_pubkey: &str,
    ) -> Result<Vec<String>, ListRecipientKeysError>;
    async fn persist_forward_message(
        &self,
        recipient_key: &str,
        message_data: &str,
    ) -> Result<(), PersistForwardMessageError>;
    async fn retrieve_pending_message_count(
        &self,
        auth_pubkey: &str,
        recipient_key: Option<&String>,
    ) -> Result<u32, RetrievePendingMessageCountError>;
    async fn retrieve_pending_messages(
        &self,
        auth_pubkey: &str,
        limit: u32,
        recipient_key: Option<&String>,
    ) -> Result<Vec<(String, Vec<u8>)>, RetrievePendingMessagesError>;
    // async fn mark_messages_received(&self, message_id: Vec<u32>);
    /// Returns vector of (account_name, auth_pubkey)
    async fn list_accounts(&self) -> Result<Vec<(String, String)>, ListAccountsError>;
    /// Returns account details (sr.no, account_name, our_signing_key, did_doc)
    async fn get_account_details(
        &self,
        auth_pubkey: &str,
    ) -> Result<AccountDetails, GetAccountDetailsError>;
}

#[derive(Debug)]
pub struct AccountDetails {
    // Unique ID for account
    pub account_id: Vec<u8>,
    // A human readable string name for the account
    // (not to be used for any other purpose)
    pub account_name: String,
    pub auth_pubkey: VerKey,
    pub our_signing_key: VerKey,
    pub their_did_doc: AriesDidDoc,
}
