// Copyright 2023 Naian G.
// SPDX-License-Identifier: Apache-2.0

use sqlx::{postgres::PgPoolOptions, PgPool};

pub async fn get_db_pool() -> PgPool {
    let _ = dotenvy::dotenv();
    let database_url: String = std::env::var("POSTGRES_URL")
        .expect("Required environment variable POSTGRES_URL on command line or in .env!");

    PgPoolOptions::new()
        .connect(&database_url)
        .await
        .expect("Failed to connect to database!")
}

use async_trait::async_trait;
use super::super::MediatorPersistence;

#[cfg(feature = "postgres_db")]
#[async_trait]
impl MediatorPersistence for sqlx::AnyPool {
    async fn create_account(&self, auth_pubkey: &str) -> Result<(), String>{unimplemented!()}
    async fn get_account_id(&self, auth_pubkey: &str) -> Result<Vec<u8>, String>{unimplemented!()}
    // async fn vaporize_account(&self, auth_pubkey: String);
    async fn add_recipient(&self, auth_pubkey: &str, recipient_key: &str) ->  Result<(), String>{unimplemented!()}
    async fn remove_recipient(&self, auth_pubkey: &str, recipient_key: &str) ->  Result<(), String>{unimplemented!()}
    async fn list_recipient_keys(&self, auth_pubkey: &str) -> Result<Vec<String>, String>{unimplemented!()}
    async fn persist_forward_message(&self, recipient_key: &str, message_data: &str) -> Result<(), String>{unimplemented!()}
    async fn retrieve_pending_message_count(&self, auth_pubkey: &str, recipient_key: Option<&String>) -> Result<u32, String>{unimplemented!()}
    async fn retrieve_pending_messages(
        &self,
        auth_pubkey: &str,
        limit: u32,
        recipient_key: Option<&String>,
    ) -> Result<Vec<(String, Vec<u8>)>, String>{unimplemented!()}
}

#[cfg(test)]
mod tests {
    use super::get_db_pool;

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }

    #[tokio::test]
    #[ignore]
    pub async fn test_query() {
        let first_todo_title = "Learn SQLx";
        let pool = get_db_pool().await;

        sqlx::query("INSERT INTO todos (title) VALUES ($1)")
            .bind(first_todo_title)
            .execute(&pool)
            .await
            .unwrap();
    }
}
