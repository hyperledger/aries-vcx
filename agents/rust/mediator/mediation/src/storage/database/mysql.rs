// Copyright 2023 Naian G.
// SPDX-License-Identifier: Apache-2.0

use log::info;

use sqlx::{mysql::MySqlPoolOptions, MySqlPool};
use sqlx::Row;
use futures::TryStreamExt;

use async_trait::async_trait;
use super::super::MediatorPersistence;


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
    async fn create_account(&self, auth_pubkey: &str, our_signing_key: &str, did_doc: &str) -> Result<(), String> {
        info!("Adding new account to database with auth_pubkey {:#?}", &auth_pubkey);
        let insert_result = sqlx::query("INSERT INTO accounts (auth_pubkey, our_signing_key, did_doc) VALUES (?, ?, ?);")
            .bind(auth_pubkey)
            .bind(our_signing_key)
            .bind(did_doc)
            .execute(self)
            .await;
        if let Err(err) = insert_result {
            info!("Error during creating new account, {:#?}", err);
            return Err(format!("{:#}", err))
        };
        let account_id = self.get_account_id(auth_pubkey).await?;
        info!("Created account {:x?} for auth_pubkey {:#?}", &account_id, &auth_pubkey);
        Ok(())
    }
    /// Get account id associated with auth_pubkey
    async fn get_account_id(&self, auth_pubkey: &str) -> Result<Vec<u8>, String> {
        let account_id: Vec<u8> = match 
        sqlx::query("SELECT (account_id) FROM accounts WHERE auth_pubkey = ?;")
        .bind(auth_pubkey)
        .fetch_one(self)
        .await
        {
            Ok(account_row) => {account_row.get("account_id") }
            Err(err) => {
                info!("Error while finding account, {:#?}", err);
                return Err(format!("{:#}", err))
            }
        };
        Ok(account_id)
    }
    #[cfg(feature = "mediator_persistence_extras")]
    async fn list_accounts(&self) -> Result<Vec<(String, String)>, String> {
        let list: Vec<(String, String)> = sqlx::query("SELECT account_name, auth_pubkey FROM accounts;")
            .fetch_all(self).await.map_err(|e| e.to_string())?
            .iter().map(
                |row| (
                    row.get("account_name"), 
                    row.get("auth_pubkey")
                )
            ).collect();
        Ok(list)
    }
    #[cfg(feature = "mediator_persistence_extras")]
    async fn get_account_details(&self, auth_pubkey: &str) -> Result<(u64, String, String, serde_json::Value), String> {
        let row = sqlx::query("SELECT * FROM accounts WHERE auth_pubkey = ?;")
            .bind(auth_pubkey)
            .fetch_one(self).await.map_err(|e| e.to_string())?;
        Ok((
            row.get("seq_num"), 
            row.get("account_name"), 
            row.get("our_signing_key"), 
            row.get::<serde_json::Value, &str>("did_doc")
        ))
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
    async fn persist_forward_message(&self, recipient_key: &str, message_data: &str) -> Result<(), String> {
        // Fetch recipient with given recipient_key
        info!("Fetching recipient with recipient_key {:#?}", recipient_key);
        let recipient_row = sqlx::query(
            "SELECT * FROM recipients WHERE recipient_key = ?"
        )
            .bind(recipient_key)
            .fetch_one(self)
            .await;
        if let Err(err) = recipient_row {
            info!("Error while finding target recipient, {:#}", err);
            return Err(format!("{:#}", err))
        }
        let account_id: Vec<u8> = recipient_row.unwrap().get("account_id");
        // Save message for recipient
        info!("Persisting message for account {:x?}", account_id);
        let insert_result = sqlx::query("INSERT INTO messages (account_id, recipient_key, message_data) VALUES (?, ?, ?)")
            .bind(&account_id)
            .bind(recipient_key)
            .bind(message_data)
            .execute(self)
            .await;
        if let Err(err) = insert_result {
            info!("Error while saving message for recipient {:x?}, {:#}", recipient_key, err);
            return Err(format!("{:#}", err))
        }
        Ok(())
    }
    async fn retrieve_pending_message_count(&self, auth_pubkey: &str, recipient_key: Option<&String>) -> Result<u32, String> {
        let account_id: Vec<u8> = self.get_account_id(auth_pubkey).await?;
        let message_count_result = if let Some(recipient_key) = recipient_key {
            sqlx::query(
                "SELECT COUNT(*) FROM messages
                WHERE (account_id = ?) AND (recipient_key = ?)"
            )
            .bind(&account_id)
            .bind(recipient_key)
            .fetch_one(self)
            .await
        }
        else {
            sqlx::query(
                "SELECT COUNT(*) FROM messages
                WHERE (account_id = ?)"
            )
            .bind(&account_id)
            .fetch_one(self)
            .await
        };
        // MySQL BIGINT can be converted to i32 only, not u32
        let message_count: i32 = message_count_result.unwrap().get::<i32, &str>("COUNT(*)"); 
        let message_count: u32 = message_count.try_into().unwrap();
        info!("Total message count of all requested recipients: {:#?}", &message_count);
        Ok(message_count)
    }
    async fn retrieve_pending_messages(
        &self,
        auth_pubkey: &str,
        limit: u32,
        recipient_key: Option<&String>,
    ) -> Result<Vec<(String, Vec<u8>)>, String> {
        info!("Processing retrieve for messages to recipient_key {:#?} of auth_pubkey {:#?}", recipient_key, auth_pubkey);
        let account_id: Vec<u8> = self.get_account_id(auth_pubkey).await?;
        let mut messages: Vec<(String, Vec<u8>)> = Vec::new();
        let mut message_rows = if let Some(recipient_key) = recipient_key {
            sqlx::query(
                "SELECT * FROM messages WHERE (account_id = ?) AND (recipient_key = ?)"
            )
                .bind(&account_id)
                .bind(recipient_key)
                .fetch(self)
        } else {
            sqlx::query(
                "SELECT * FROM messages WHERE (account_id = ?)"
            )
                .bind(&account_id)
                .fetch(self)
        };
        while let Some(message_row) = message_rows.try_next().await.unwrap() {
            let id: String = message_row.get("message_id");
            let msg : Vec<u8> = message_row.get("message_data");
            // debug!("id {:#?}", id);
            // debug!("recipient {:x?}", recipient);
            // debug!("message {:x?}", msg); 
            messages.push((id, msg));
            if u32::try_from(messages.len()).unwrap() >= limit {
                info!("Found enough messages {:#?}", limit);
                break;
            }
        }
        info!("Found total of {:#?} messages, returning them", messages.len());
        Ok(messages)
    }
    async fn add_recipient(&self, auth_pubkey: &str, recipient_key: &str) ->  Result<(), String> {
        info!("Adding recipient_key to account with auth_pubkey {:#?}", auth_pubkey);
        let account_id: Vec<u8> = self.get_account_id(auth_pubkey).await?;
        info!(
            "Found matching account {:x?}. Proceeding with attempt to add recipient recipient_key {:#?} ",
            account_id,
            recipient_key
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
    async fn remove_recipient(&self, auth_pubkey: &str, recipient_key: &str) ->  Result<(), String> {
        info!("Removing recipient_key from account with auth_pubkey {:#?}", auth_pubkey);
        let account_id: Vec<u8> = self.get_account_id(auth_pubkey).await?;
        info!(
            "Found matching account {:x?}. Proceeding with attempt to remove recipient recipient_key {:#?} ",
            account_id,
            recipient_key
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
    async fn list_recipient_keys(&self, auth_pubkey: &str) -> Result<Vec<String>, String> {
        info!("Retrieving recipient_keys for account with auth_pubkey {:#?}", auth_pubkey);
        let account_id: Vec<u8> = self.get_account_id(auth_pubkey).await?;
        let recipient_keys: Vec<String> = match
            sqlx::query("SELECT (recipient_key) FROM recipients WHERE account_id = ?;")
            .bind(&account_id)
            .fetch_all(self)
            .await
        {
            Ok(recipient_key_rows) => {
                recipient_key_rows
                    .into_iter()
                    .map(|row| row.get("recipient_key"))
                    .collect()
            }
            Err(err) => {
                info!("Error while getting recipient_keys, {:#}", err);
                return Err(format!("{:#}", err))
            }
        };
        Ok(recipient_keys)
    }
}
