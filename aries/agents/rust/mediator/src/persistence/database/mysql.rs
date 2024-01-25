// Copyright 2023 Naian G.
// SPDX-License-Identifier: Apache-2.0

use anyhow::anyhow;
use async_trait::async_trait;
use diddoc_legacy::aries::diddoc::AriesDidDoc;
use futures::TryStreamExt;
use log::info;
use sqlx::{
    mysql::{MySqlPoolOptions, MySqlRow},
    MySqlPool, Row,
};

use super::super::MediatorPersistence;
use crate::{
    persistence::{
        errors::{CreateAccountError, GetAccountDetailsError, ListAccountsError},
        AccountDetails,
    },
    utils::structs::VerKey,
};

pub async fn get_db_pool() -> MySqlPool {
    let _ = dotenvy::dotenv();
    let database_url = std::env::var("MYSQL_URL")
        .expect("Required environment variable MYSQL_URL on command line or in .env!");

    MySqlPoolOptions::new()
        .connect(&database_url)
        .await
        .expect("Failed to connect to database!")
}

/// MediatorPersistence trait implementation for MySql Database
#[async_trait]
impl MediatorPersistence for sqlx::MySqlPool {
    async fn create_account(
        &self,
        auth_pubkey: &str,
        our_signing_key: &str,
        did_doc: &str,
    ) -> Result<(), CreateAccountError> {
        info!(
            "Adding new account to database with auth_pubkey {:#?}",
            &auth_pubkey
        );
        let insert_result = sqlx::query(
            "INSERT INTO accounts (auth_pubkey, our_signing_key, did_doc) VALUES (?, ?, ?);",
        )
        .bind(auth_pubkey)
        .bind(our_signing_key)
        .bind(did_doc)
        .execute(self)
        .await;
        if let Err(err) = insert_result {
            info!("Error during creating new account, {:#?}", err);
            return Err(CreateAccountError::StorageBackendError {
                source: anyhow!(err),
            });
        };
        let account_details = self.get_account_details(auth_pubkey).await?;
        info!(
            "Created account {:?} for auth_pubkey {:#?}",
            &account_details, &auth_pubkey
        );
        Ok(())
    }
    /// Get account id associated with auth_pubkey
    async fn get_account_id(&self, auth_pubkey: &str) -> Result<Vec<u8>, String> {
        let account_id: Vec<u8> =
            match sqlx::query("SELECT (account_id) FROM accounts WHERE auth_pubkey = ?;")
                .bind(auth_pubkey)
                .fetch_one(self)
                .await
            {
                Ok(account_row) => account_row.get("account_id"),
                Err(err) => {
                    info!("Error while finding account, {:#?}", err);
                    return Err(format!("{:#}", err));
                }
            };
        Ok(account_id)
    }
    /// Returns list of accounts in form of tuples containing
    /// account_name and associated auth_pubkey
    async fn list_accounts(&self) -> Result<Vec<(String, VerKey)>, ListAccountsError> {
        let accounts_rows: Vec<MySqlRow> =
            sqlx::query("SELECT account_name, auth_pubkey FROM accounts;")
                .fetch_all(self)
                .await
                .map_err(|e| anyhow!(e))?;
        let mut vec_tup = vec![];
        for row in accounts_rows {
            vec_tup.push((
                row.try_get("account_name").map_err(|e| anyhow!(e))?,
                row.try_get("auth_pubkey").map_err(|e| anyhow!(e))?,
            ))
        }
        Ok(vec_tup)
    }
    async fn get_account_details(
        &self,
        auth_pubkey: &str,
    ) -> Result<AccountDetails, GetAccountDetailsError> {
        let row = sqlx::query("SELECT * FROM accounts WHERE auth_pubkey = ?;")
            .bind(auth_pubkey)
            .fetch_one(self)
            .await
            .map_err(|e| match e {
                sqlx::error::Error::RowNotFound => GetAccountDetailsError::NotFound,
                _ => GetAccountDetailsError::hkpLXwHUQError(anyhow!(e)),
            })?;
        let account_id = row
            .try_get("account_id")
            .map_err(|e| GetAccountDetailsError::DecodeError(e.to_string()))?;
        let account_name = row
            .try_get("account_name")
            .map_err(|e| GetAccountDetailsError::DecodeError(e.to_string()))?;
        let auth_pubkey = row
            .try_get("auth_pubkey")
            .map_err(|e| GetAccountDetailsError::DecodeError(e.to_string()))?;
        let our_signing_key = row
            .try_get("our_signing_key")
            .map_err(|e| GetAccountDetailsError::DecodeError(e.to_string()))?;
        let did_doc_json = row
            .try_get::<serde_json::Value, &str>("did_doc")
            .map_err(|e| GetAccountDetailsError::DecodeError(e.to_string()))?;
        let account_details = AccountDetails {
            account_id,
            account_name,
            auth_pubkey,
            our_signing_key,
            their_did_doc: serde_json::from_value::<AriesDidDoc>(did_doc_json)
                .map_err(|e| GetAccountDetailsError::DecodeError(e.to_string()))?,
        };
        Ok(account_details)
    }

    // async fn vaporize_account(&self, auth_pubkey: String) {
    //     let account: Vec<u8> = self.get_account(auth_pubkey).await?;
    //     let mut recipient_rows = sqlx::query(
    //         "SELECT * FROM recipients WHERE account = ?;"
    //     )
    //         .bind(&account)
    //         .fetch(self);

    //     while let Some(recipient_row) = recipient_rows.try_next().await.unwrap() {
    //         // map the row into a user-defined domain type
    //         let recipient: Vec<u8> = recipient_row.get("recipient");  // binary decode
    //         info!("Recipient {:x?}", recipient);
    //         sqlx::query("DROP (*) FROM messages WHERE recipient = ?;")
    //         .bind(&recipient)
    //         .execute(self)
    //         .await
    //         .unwrap();
    //         sqlx::query("DROP (*) FROM recipients WHERE recipient = ?;")
    //         .bind(&recipient)
    //         .execute(self)
    //         .await
    //         .unwrap();

    //     }

    // }
    async fn persist_forward_message(
        &self,
        recipient_key: &str,
        message_data: &str,
    ) -> Result<(), String> {
        // Fetch recipient with given recipient_key
        info!("Fetching recipient with recipient_key {:#?}", recipient_key);
        let recipient_row = sqlx::query("SELECT * FROM recipients WHERE recipient_key = ?")
            .bind(recipient_key)
            .fetch_one(self)
            .await;
        if let Err(err) = recipient_row {
            info!("Error while finding target recipient, {:#}", err);
            return Err(format!("{:#}", err));
        }
        let account_id: Vec<u8> = recipient_row.unwrap().get("account_id");
        // Save message for recipient
        info!("Persisting message for account {:x?}", account_id);
        let insert_result = sqlx::query(
            "INSERT INTO messages (account_id, recipient_key, message_data) VALUES (?, ?, ?)",
        )
        .bind(&account_id)
        .bind(recipient_key)
        .bind(message_data)
        .execute(self)
        .await;
        if let Err(err) = insert_result {
            info!(
                "Error while saving message for recipient {:x?}, {:#}",
                recipient_key, err
            );
            return Err(format!("{:#}", err));
        }
        Ok(())
    }
    async fn retrieve_pending_message_count(
        &self,
        auth_pubkey: &str,
        recipient_key: Option<&String>,
    ) -> Result<u32, String> {
        let account_id: Vec<u8> = self.get_account_id(auth_pubkey).await?;
        let message_count_result = if let Some(recipient_key) = recipient_key {
            sqlx::query(
                "SELECT COUNT(*) FROM messages
                WHERE (account_id = ?) AND (recipient_key = ?)",
            )
            .bind(&account_id)
            .bind(recipient_key)
            .fetch_one(self)
            .await
        } else {
            sqlx::query(
                "SELECT COUNT(*) FROM messages
                WHERE (account_id = ?)",
            )
            .bind(&account_id)
            .fetch_one(self)
            .await
        };
        // MySQL BIGINT can be converted to i32 only, not u32
        let message_count: i32 = message_count_result.unwrap().get::<i32, &str>("COUNT(*)");
        let message_count: u32 = message_count.try_into().unwrap();
        info!(
            "Total message count of all requested recipients: {:#?}",
            &message_count
        );
        Ok(message_count)
    }
    async fn retrieve_pending_messages(
        &self,
        auth_pubkey: &str,
        limit: u32,
        recipient_key: Option<&VerKey>,
    ) -> Result<Vec<(String, Vec<u8>)>, String> {
        info!(
            "Processing retrieve for messages to recipient_key {:#?} of auth_pubkey {:#?}",
            recipient_key, auth_pubkey
        );
        let account_id: Vec<u8> = self.get_account_id(auth_pubkey).await?;
        let mut messages: Vec<(String, Vec<u8>)> = Vec::new();
        let mut message_rows = if let Some(recipient_key) = recipient_key {
            sqlx::query("SELECT * FROM messages WHERE (account_id = ?) AND (recipient_key = ?)")
                .bind(&account_id)
                .bind(recipient_key)
                .fetch(self)
        } else {
            sqlx::query("SELECT * FROM messages WHERE (account_id = ?)")
                .bind(&account_id)
                .fetch(self)
        };
        while let Some(message_row) = message_rows.try_next().await.unwrap() {
            let id: String = message_row.get("message_id");
            let msg: Vec<u8> = message_row.get("message_data");
            // debug!("id {:#?}", id);
            // debug!("recipient {:x?}", recipient);
            // debug!("message {:x?}", msg);
            messages.push((id, msg));
            if u32::try_from(messages.len()).unwrap() >= limit {
                info!("Found enough messages {:#?}", limit);
                break;
            }
        }
        info!(
            "Found total of {:#?} messages, returning them",
            messages.len()
        );
        Ok(messages)
    }
    async fn add_recipient(&self, auth_pubkey: &str, recipient_key: &str) -> Result<(), String> {
        info!(
            "Adding recipient_key to account with auth_pubkey {:#?}",
            auth_pubkey
        );
        let account_id: Vec<u8> = self.get_account_id(auth_pubkey).await?;
        info!(
            "Found matching account {:x?}. Proceeding with attempt to add recipient recipient_key \
             {:#?} ",
            account_id, recipient_key
        );
        match sqlx::query("INSERT INTO recipients (account_id, recipient_key) VALUES (?, ?);")
            .bind(&account_id)
            .bind(recipient_key)
            .execute(self)
            .await
        {
            Ok(_result) => Ok(()),
            Err(err) => {
                info!("Error while adding recipient, {:#}", err);
                Err(format!("{:#}", err))
            }
        }
    }
    async fn remove_recipient(&self, auth_pubkey: &str, recipient_key: &str) -> Result<(), String> {
        info!(
            "Removing recipient_key from account with auth_pubkey {:#?}",
            auth_pubkey
        );
        let account_id: Vec<u8> = self.get_account_id(auth_pubkey).await?;
        info!(
            "Found matching account {:x?}. Proceeding with attempt to remove recipient \
             recipient_key {:#?} ",
            account_id, recipient_key
        );
        match sqlx::query("DELETE FROM recipients WHERE (account_id = ?) AND (recipient_key = ?);")
            .bind(&account_id)
            .bind(recipient_key)
            .execute(self)
            .await
        {
            Ok(_result) => Ok(()),
            Err(err) => {
                info!("Error while removing recipient, {:#}", err);
                Err(format!("{:#}", err))
            }
        }
    }
    async fn list_recipient_keys(&self, auth_pubkey: &str) -> Result<Vec<VerKey>, String> {
        info!(
            "Retrieving recipient_keys for account with auth_pubkey {:#?}",
            auth_pubkey
        );
        let account_id: Vec<u8> = self.get_account_id(auth_pubkey).await?;
        let recipient_keys: Vec<VerKey> =
            match sqlx::query("SELECT (recipient_key) FROM recipients WHERE account_id = ?;")
                .bind(&account_id)
                .fetch_all(self)
                .await
            {
                Ok(recipient_key_rows) => recipient_key_rows
                    .into_iter()
                    .map(|row| row.get("recipient_key"))
                    .collect(),
                Err(err) => {
                    info!("Error while getting recipient_keys, {:#}", err);
                    return Err(format!("{:#}", err));
                }
            };
        Ok(recipient_keys)
    }
}
