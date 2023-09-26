use std::{
    collections::{HashMap, VecDeque},
    fs,
};

use async_trait::async_trait;
use indy_api_types::errors::prelude::*;
use indy_utils::environment;
use log::LevelFilter;
use serde::Deserialize;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
    ConnectOptions, SqlitePool,
};

use crate::{
    language,
    storage::{StorageIterator, StorageRecord, Tag, TagName, WalletStorage, WalletStorageType},
    wallet::EncryptedValue,
    RecordOptions, SearchOptions,
};

mod query;

const _SQLITE_DB: &str = "sqlite.db";

struct SQLiteStorageIterator {
    records: Option<VecDeque<StorageRecord>>,
    total_count: Option<usize>,
}

impl SQLiteStorageIterator {
    fn new(
        records: Option<VecDeque<StorageRecord>>,
        total_count: Option<usize>,
    ) -> IndyResult<SQLiteStorageIterator> {
        Ok(SQLiteStorageIterator {
            records,
            total_count,
        })
    }
}

#[async_trait]
impl StorageIterator for SQLiteStorageIterator {
    async fn next(&mut self) -> IndyResult<Option<StorageRecord>> {
        if let Some(ref mut records) = self.records {
            Ok(records.pop_front())
        } else {
            Ok(None)
        }
    }

    fn get_total_count(&self) -> IndyResult<Option<usize>> {
        Ok(self.total_count.to_owned())
    }
}

#[derive(Deserialize, Debug)]
struct Config {
    path: Option<String>,
}

#[derive(Debug)]
struct SQLiteStorage {
    pool: SqlitePool,
}

pub struct SQLiteStorageType {}

impl SQLiteStorageType {
    pub fn new() -> SQLiteStorageType {
        SQLiteStorageType {}
    }

    fn _db_path(id: &str, config: Option<&Config>) -> std::path::PathBuf {
        let mut path = match config {
            Some(Config {
                path: Some(ref path),
            }) => std::path::PathBuf::from(path),
            _ => environment::wallet_home_path(),
        };

        path.push(id);
        path.push(_SQLITE_DB);
        path
    }
}

#[async_trait]
impl WalletStorage for SQLiteStorage {
    ///
    /// Tries to fetch values and/or tags from the storage.
    /// Returns Result with StorageEntity object which holds requested data in case of success or
    /// Result with IndyError in case of failure.
    ///
    ///
    /// # Arguments
    ///
    ///  * `type_` - type_ of the item in storage
    ///  * `id` - id of the item in storage
    ///  * `options` - JSon containing what needs to be fetched.
    ///  Example: {"retrieveValue": true, "retrieveTags": true}
    ///
    /// # Returns
    ///
    /// Result that can be either:
    ///
    ///  * `StorageEntity` - Contains name, optional value and optional tags
    ///  * `IndyError`
    ///
    /// # Errors
    ///
    /// Any of the following `IndyError` type_ of errors can be throw by this method:
    ///
    ///  * `IndyError::Closed` - Storage is closed
    ///  * `IndyError::ItemNotFound` - Item is not found in database
    ///  * `IOError("IO error during storage operation:...")` - Failed connection or SQL query
    async fn get(&self, type_: &[u8], id: &[u8], options: &str) -> IndyResult<StorageRecord> {
        let options: RecordOptions = serde_json::from_str(options).to_indy(
            IndyErrorKind::InvalidStructure,
            "RecordOptions is malformed json",
        )?;

        let mut conn = self.pool.acquire().await?;

        let (item_id, value, key): (i64, Vec<u8>, Vec<u8>) =
            sqlx::query_as("SELECT id, value, key FROM items where type = ?1 AND name = ?2")
                .bind(type_)
                .bind(id)
                .fetch_one(&mut conn)
                .await?;

        let value = if options.retrieve_value {
            Some(EncryptedValue::new(value, key))
        } else {
            None
        };

        let type_ = if options.retrieve_type {
            Some(type_.to_vec())
        } else {
            None
        };

        let tags = if options.retrieve_tags {
            let mut tags = Vec::new();

            tags.extend(
                sqlx::query_as::<_, (Vec<u8>, String)>(
                    "SELECT name, value from tags_plaintext where item_id = ?",
                )
                .bind(item_id)
                .fetch_all(&mut conn)
                .await?
                .drain(..)
                .map(|r| Tag::PlainText(r.0, r.1)),
            );

            tags.extend(
                sqlx::query_as::<_, (Vec<u8>, Vec<u8>)>(
                    "SELECT name, value from tags_encrypted where item_id = ?",
                )
                .bind(item_id)
                .fetch_all(&mut conn)
                .await?
                .drain(..)
                .map(|r| Tag::Encrypted(r.0, r.1)),
            );

            Some(tags)
        } else {
            None
        };

        Ok(StorageRecord::new(id.to_vec(), value, type_, tags))
    }

    ///
    /// inserts value and tags into storage.
    /// Returns Result with () on success or
    /// Result with IndyError in case of failure.
    ///
    ///
    /// # Arguments
    ///
    ///  * `type_` - type of the item in storage
    ///  * `id` - id of the item in storage
    ///  * `value` - value of the item in storage
    ///  * `value_key` - key used to encrypt the value
    ///  * `tags` - tags assigned to the value
    ///
    /// # Returns
    ///
    /// Result that can be either:
    ///
    ///  * `()`
    ///  * `IndyError`
    ///
    /// # Errors
    ///
    /// Any of the following `IndyError` class of errors can be throw by this method:
    ///
    ///  * `IndyError::Closed` - Storage is closed
    ///  * `IndyError::ItemAlreadyExists` - Item is already present in database
    ///  * `IOError("IO error during storage operation:...")` - Failed connection or SQL query
    async fn add(
        &self,
        type_: &[u8],
        id: &[u8],
        value: &EncryptedValue,
        tags: &[Tag],
    ) -> IndyResult<()> {
        let mut tx = self.pool.begin().await?;

        let id = sqlx::query("INSERT INTO items (type, name, value, key) VALUES (?1, ?2, ?3, ?4)")
            .bind(type_)
            .bind(id)
            .bind(&value.data)
            .bind(&value.key)
            .execute(&mut tx)
            .await?
            .last_insert_rowid();

        for tag in tags {
            match *tag {
                Tag::Encrypted(ref tag_name, ref tag_data) => {
                    sqlx::query(
                        "INSERT INTO tags_encrypted (item_id, name, value) VALUES (?1, ?2, ?3)",
                    )
                    .bind(id)
                    .bind(tag_name)
                    .bind(tag_data)
                    .execute(&mut tx)
                    .await?
                }
                Tag::PlainText(ref tag_name, ref tag_data) => {
                    sqlx::query(
                        "INSERT INTO tags_plaintext (item_id, name, value) VALUES (?1, ?2, ?3)",
                    )
                    .bind(id)
                    .bind(tag_name)
                    .bind(tag_data)
                    .execute(&mut tx)
                    .await?
                }
            };
        }

        tx.commit().await?;
        Ok(())
    }

    async fn update(&self, type_: &[u8], id: &[u8], value: &EncryptedValue) -> IndyResult<()> {
        let mut tx = self.pool.begin().await?;

        let row_updated =
            sqlx::query("UPDATE items SET value = ?1, key = ?2 WHERE type = ?3 AND name = ?4")
                .bind(&value.data)
                .bind(&value.key)
                .bind(&type_)
                .bind(&id)
                .execute(&mut tx)
                .await?
                .rows_affected();

        match row_updated {
            1 => {
                tx.commit().await?;
                Ok(())
            }
            0 => Err(err_msg(
                IndyErrorKind::WalletItemNotFound,
                "Item to update not found",
            )),
            _ => Err(err_msg(
                IndyErrorKind::InvalidState,
                "More than one row update. Seems wallet structure is inconsistent",
            )),
        }
    }

    async fn add_tags(&self, type_: &[u8], id: &[u8], tags: &[Tag]) -> IndyResult<()> {
        let mut tx = self.pool.begin().await?;

        let (item_id,): (i64,) =
            sqlx::query_as("SELECT id FROM items WHERE type = ?1 AND name = ?2")
                .bind(type_)
                .bind(id)
                .fetch_one(&mut tx)
                .await?;

        for tag in tags {
            match *tag {
                Tag::Encrypted(ref tag_name, ref tag_data) => {
                    sqlx::query(
                        "INSERT OR REPLACE INTO tags_encrypted (item_id, name, value) VALUES (?1, \
                         ?2, ?3)",
                    )
                    .bind(item_id)
                    .bind(tag_name)
                    .bind(tag_data)
                    .execute(&mut tx)
                    .await?
                }
                Tag::PlainText(ref tag_name, ref tag_data) => {
                    sqlx::query(
                        "INSERT OR REPLACE INTO tags_plaintext (item_id, name, value) VALUES (?1, \
                         ?2, ?3)",
                    )
                    .bind(item_id)
                    .bind(tag_name)
                    .bind(tag_data)
                    .execute(&mut tx)
                    .await?
                }
            };
        }

        tx.commit().await?;
        Ok(())
    }

    async fn update_tags(&self, type_: &[u8], id: &[u8], tags: &[Tag]) -> IndyResult<()> {
        let mut tx = self.pool.begin().await?;

        let (item_id,): (i64,) =
            sqlx::query_as("SELECT id FROM items WHERE type = ?1 AND name = ?2")
                .bind(type_)
                .bind(&id)
                .fetch_one(&mut tx)
                .await?;

        sqlx::query("DELETE FROM tags_encrypted WHERE item_id = ?1")
            .bind(item_id)
            .execute(&mut tx)
            .await?;

        sqlx::query("DELETE FROM tags_plaintext WHERE item_id = ?1")
            .bind(item_id)
            .execute(&mut tx)
            .await?;

        for tag in tags {
            match *tag {
                Tag::Encrypted(ref tag_name, ref tag_data) => {
                    sqlx::query(
                        "INSERT INTO tags_encrypted (item_id, name, value) VALUES (?1, ?2, ?3)",
                    )
                    .bind(item_id)
                    .bind(tag_name)
                    .bind(tag_data)
                    .execute(&mut tx)
                    .await?
                }
                Tag::PlainText(ref tag_name, ref tag_data) => {
                    sqlx::query(
                        "INSERT INTO tags_plaintext (item_id, name, value) VALUES (?1, ?2, ?3)",
                    )
                    .bind(item_id)
                    .bind(tag_name)
                    .bind(tag_data)
                    .execute(&mut tx)
                    .await?
                }
            };
        }

        tx.commit().await?;

        Ok(())
    }

    async fn delete_tags(&self, type_: &[u8], id: &[u8], tag_names: &[TagName]) -> IndyResult<()> {
        let mut tx = self.pool.begin().await?;

        let (item_id,): (i64,) =
            sqlx::query_as("SELECT id FROM items WHERE type = ?1 AND name = ?2")
                .bind(type_)
                .bind(id)
                .fetch_one(&mut tx)
                .await?;

        for tag_name in tag_names {
            match *tag_name {
                TagName::OfEncrypted(ref tag_name) => {
                    sqlx::query("DELETE FROM tags_encrypted WHERE item_id = ?1 AND name = ?2")
                        .bind(item_id)
                        .bind(tag_name)
                        .execute(&mut tx)
                        .await?
                }
                TagName::OfPlain(ref tag_name) => {
                    sqlx::query("DELETE FROM tags_plaintext WHERE item_id = ?1 AND name = ?2")
                        .bind(item_id)
                        .bind(tag_name)
                        .execute(&mut tx)
                        .await?
                }
            };
        }

        tx.commit().await?;
        Ok(())
    }

    ///
    /// deletes value and tags into storage.
    /// Returns Result with () on success or
    /// Result with IndyError in case of failure.
    ///
    ///
    /// # Arguments
    ///
    ///  * `type_` - type of the item in storage
    ///  * `id` - id of the item in storage
    ///
    /// # Returns
    ///
    /// Result that can be either:
    ///
    ///  * `()`
    ///  * `IndyError`
    ///
    /// # Errors
    ///
    /// Any of the following `IndyError` type_ of errors can be throw by this method:
    ///
    ///  * `IndyError::Closed` - Storage is closed
    ///  * `IndyError::ItemNotFound` - Item is not found in database
    ///  * `IOError("IO error during storage operation:...")` - Failed connection or SQL query
    async fn delete(&self, type_: &[u8], id: &[u8]) -> IndyResult<()> {
        let mut tx = self.pool.begin().await?;

        let rows_affected = sqlx::query("DELETE FROM items where type = ?1 AND name = ?2")
            .bind(type_)
            .bind(id)
            .execute(&mut tx)
            .await?
            .rows_affected();

        match rows_affected {
            1 => {
                tx.commit().await?;
                Ok(())
            }
            0 => Err(err_msg(
                IndyErrorKind::WalletItemNotFound,
                "Item to delete not found",
            )),
            _ => Err(err_msg(
                IndyErrorKind::InvalidState,
                "More than one row deleted. Seems wallet structure is inconsistent",
            )),
        }
    }

    async fn get_storage_metadata(&self) -> IndyResult<Vec<u8>> {
        let mut conn = self.pool.acquire().await?;

        let (metadata,): (Vec<u8>,) = sqlx::query_as::<_, (Vec<u8>,)>("SELECT value FROM metadata")
            .fetch_one(&mut conn)
            .await?;

        Ok(metadata)
    }

    async fn set_storage_metadata(&self, metadata: &[u8]) -> IndyResult<()> {
        let mut tx = self.pool.begin().await?;

        sqlx::query("UPDATE metadata SET value = ?1")
            .bind(metadata)
            .execute(&mut tx)
            .await?;

        tx.commit().await?;
        Ok(())
    }

    async fn get_all(&self) -> IndyResult<Box<dyn StorageIterator>> {
        let mut conn = self.pool.acquire().await?;
        let mut tags: Vec<(i64, Tag)> = Vec::new();

        tags.extend(
            sqlx::query_as::<_, (i64, Vec<u8>, String)>(
                "SELECT item_id, name, value from tags_plaintext",
            )
            .fetch_all(&mut conn)
            .await?
            .drain(..)
            .map(|r| (r.0, Tag::PlainText(r.1, r.2))),
        );

        tags.extend(
            sqlx::query_as::<_, (i64, Vec<u8>, Vec<u8>)>(
                "SELECT item_id, name, value from tags_encrypted",
            )
            .fetch_all(&mut conn)
            .await?
            .drain(..)
            .map(|r| (r.0, Tag::Encrypted(r.1, r.2))),
        );

        let mut mtags = HashMap::new();

        for (k, v) in tags {
            mtags.entry(k).or_insert_with(Vec::new).push(v)
        }

        let records: VecDeque<_> = sqlx::query_as::<_, (i64, Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>)>(
            "SELECT id, name, value, key, type FROM items",
        )
        .fetch_all(&mut conn)
        .await?
        .drain(..)
        .map(|r| {
            StorageRecord::new(
                r.1,
                Some(EncryptedValue::new(r.2, r.3)),
                Some(r.4),
                mtags.remove(&r.0).or_else(|| Some(Vec::new())),
            )
        })
        .collect();

        let total_count = records.len();

        Ok(Box::new(SQLiteStorageIterator::new(
            Some(records),
            Some(total_count),
        )?))
    }

    async fn search(
        &self,
        type_: &[u8],
        query: &language::Operator,
        options: Option<&str>,
    ) -> IndyResult<Box<dyn StorageIterator>> {
        let options = if let Some(options) = options {
            serde_json::from_str(options).to_indy(
                IndyErrorKind::InvalidStructure,
                "Search options is malformed json",
            )?
        } else {
            SearchOptions::default()
        };

        let mut conn = self.pool.acquire().await?;

        let records = if options.retrieve_records {
            let (query, args) = query::wql_to_sql(type_, query, None)?;

            // "SELECT i.id, i.name, i.value, i.key, i.type FROM items as i WHERE i.type = ?"

            let mut query =
                sqlx::query_as::<sqlx::Sqlite, (i64, Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>)>(&query);

            for arg in args.iter() {
                query = match arg {
                    query::ToSQL::ByteSlice(a) => query.bind(a),
                    query::ToSQL::CharSlice(a) => query.bind(a),
                }
            }

            let mut records = query.fetch_all(&mut conn).await?;

            let mut mtags = if options.retrieve_tags && !records.is_empty() {
                let mut tags: Vec<(i64, Tag)> = Vec::new();

                let in_binings = std::iter::repeat("?")
                    .take(records.len())
                    .collect::<Vec<_>>()
                    .join(",");

                let query = format!(
                    r#"
                    SELECT item_id, name, value
                    FROM tags_plaintext
                    WHERE item_id IN ({})
                    "#,
                    in_binings
                );

                let mut query = sqlx::query_as::<sqlx::Sqlite, (i64, Vec<u8>, String)>(&query);

                for record in records.iter() {
                    query = query.bind(record.0);
                }

                tags.extend(
                    query
                        .fetch_all(&mut conn)
                        .await?
                        .drain(..)
                        .map(|r| (r.0, Tag::PlainText(r.1, r.2))),
                );

                let query = format!(
                    r#"
                    SELECT item_id, name, value
                    FROM tags_encrypted
                    WHERE item_id IN ({})
                    "#,
                    in_binings
                );

                let mut query = sqlx::query_as::<sqlx::Sqlite, (i64, Vec<u8>, Vec<u8>)>(&query);

                for record in records.iter() {
                    query = query.bind(record.0);
                }

                tags.extend(
                    query
                        .fetch_all(&mut conn)
                        .await?
                        .drain(..)
                        .map(|r| (r.0, Tag::Encrypted(r.1, r.2))),
                );

                let mut mtags = HashMap::new();

                for (k, v) in tags {
                    mtags.entry(k).or_insert_with(Vec::new).push(v)
                }

                mtags
            } else {
                HashMap::new()
            };

            let records = records
                .drain(..)
                .map(|r| {
                    StorageRecord::new(
                        r.1,
                        if options.retrieve_value {
                            Some(EncryptedValue::new(r.2, r.3))
                        } else {
                            None
                        },
                        if options.retrieve_type {
                            Some(r.4)
                        } else {
                            None
                        },
                        if options.retrieve_tags {
                            mtags.remove(&r.0).or_else(|| Some(Vec::new()))
                        } else {
                            None
                        },
                    )
                })
                .collect();

            Some(records)
        } else {
            None
        };

        let total_count = if options.retrieve_total_count {
            let (query, args) = query::wql_to_sql_count(type_, query)?;

            let mut query = sqlx::query_as::<sqlx::Sqlite, (i64,)>(&query);

            for arg in args.iter() {
                query = match arg {
                    query::ToSQL::ByteSlice(a) => query.bind(a),
                    query::ToSQL::CharSlice(a) => query.bind(a),
                }
            }

            let (total_count,) = query.fetch_one(&mut conn).await?;
            Some(total_count as usize)
        } else {
            None
        };

        Ok(Box::new(SQLiteStorageIterator::new(records, total_count)?))
    }

    fn close(&mut self) -> IndyResult<()> {
        Ok(())
    }
}

#[async_trait]
impl WalletStorageType for SQLiteStorageType {
    ///
    /// Deletes the SQLite database file with the provided id from the path specified in the
    /// config file.
    ///
    /// # Arguments
    ///
    ///  * `id` - id of the SQLite DB file
    ///  * `storage_config` - config containing the location of SQLite DB files
    ///  * `storage_credentials` - DB credentials
    ///
    /// # Returns
    ///
    /// Result that can be either:
    ///
    ///  * `()`
    ///  * `IndyError`
    ///
    /// # Errors
    ///
    /// Any of the following `IndyError` type_ of errors can be throw by this method:
    ///
    ///  * `IndyError::NotFound` - File with the provided id not found
    ///  * `IOError(..)` - Deletion of the file form the file-system failed
    async fn delete_storage(
        &self,
        id: &str,
        config: Option<&str>,
        _credentials: Option<&str>,
    ) -> IndyResult<()> {
        let config = config
            .map(serde_json::from_str::<Config>)
            .map_or(Ok(None), |v| v.map(Some))
            .to_indy(IndyErrorKind::InvalidStructure, "Malformed config json")?;

        let db_file_path = SQLiteStorageType::_db_path(id, config.as_ref());

        if !db_file_path.exists() {
            return Err(err_msg(
                IndyErrorKind::WalletNotFound,
                format!("Wallet storage file isn't found: {:?}", db_file_path),
            ));
        }

        std::fs::remove_dir_all(db_file_path.parent().unwrap())?;
        Ok(())
    }

    ///
    /// Creates the SQLite DB file with the provided name in the path specified in the config file,
    /// and initializes the encryption keys needed for encryption and decryption of data.
    ///
    /// # Arguments
    ///
    ///  * `id` - name of the SQLite DB file
    ///  * `config` - config containing the location of SQLite DB files
    ///  * `credentials` - DB credentials
    ///  * `metadata` - encryption keys that need to be stored in the newly created DB
    ///
    /// # Returns
    ///
    /// Result that can be either:
    ///
    ///  * `()`
    ///  * `IndyError`
    ///
    /// # Errors
    ///
    /// Any of the following `IndyError` type_ of errors can be throw by this method:
    ///
    ///  * `AlreadyExists` - File with a given name already exists on the path
    ///  * `IOError("IO error during storage operation:...")` - Connection to the DB failed
    ///  * `IOError("Error occurred while creating wallet file:..)"` - Creation of schema failed
    ///  * `IOError("Error occurred while inserting the keys...")` - Insertion of keys failed
    ///  * `IOError(..)` - Deletion of the file form the file-system failed
    async fn create_storage(
        &self,
        id: &str,
        config: Option<&str>,
        _credentials: Option<&str>,
        metadata: &[u8],
    ) -> IndyResult<()> {
        let config = config
            .map(serde_json::from_str::<Config>)
            .map_or(Ok(None), |v| v.map(Some))
            .to_indy(IndyErrorKind::InvalidStructure, "Malformed config json")?;

        let db_path = SQLiteStorageType::_db_path(id, config.as_ref());

        if db_path.exists() {
            return Err(err_msg(
                IndyErrorKind::WalletAlreadyExists,
                format!("Wallet database file already exists: {:?}", db_path),
            ));
        }

        fs::DirBuilder::new()
            .recursive(true)
            .create(db_path.parent().unwrap())?;

        let mut conn = SqliteConnectOptions::default()
            .filename(db_path.as_path())
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .log_statements(LevelFilter::Debug)
            .connect()
            .await?;

        let res = sqlx::query(
            r#"
            PRAGMA locking_mode=EXCLUSIVE;
            PRAGMA foreign_keys=ON;

            BEGIN EXCLUSIVE TRANSACTION;

            /*** Keys Table ***/

            CREATE TABLE metadata (
                id INTEGER NOT NULL,
                value NOT NULL,
                PRIMARY KEY(id)
            );

            /*** Items Table ***/

            CREATE TABLE items(
                id INTEGER NOT NULL,
                type NOT NULL,
                name NOT NULL,
                value NOT NULL,
                key NOT NULL,
                PRIMARY KEY(id)
            );

            CREATE UNIQUE INDEX ux_items_type_name ON items(type, name);

            /*** Encrypted Tags Table ***/

            CREATE TABLE tags_encrypted(
                name NOT NULL,
                value NOT NULL,
                item_id INTEGER NOT NULL,
                PRIMARY KEY(name, item_id),
                FOREIGN KEY(item_id)
                    REFERENCES items(id)
                    ON DELETE CASCADE
                    ON UPDATE CASCADE
            );

            CREATE INDEX ix_tags_encrypted_name ON tags_encrypted(name);
            CREATE INDEX ix_tags_encrypted_value ON tags_encrypted(value);
            CREATE INDEX ix_tags_encrypted_item_id ON tags_encrypted(item_id);

            /*** PlainText Tags Table ***/

            CREATE TABLE tags_plaintext(
                name NOT NULL,
                value NOT NULL,
                item_id INTEGER NOT NULL,
                PRIMARY KEY(name, item_id),
                FOREIGN KEY(item_id)
                    REFERENCES items(id)
                    ON DELETE CASCADE
                    ON UPDATE CASCADE
            );

            CREATE INDEX ix_tags_plaintext_name ON tags_plaintext(name);
            CREATE INDEX ix_tags_plaintext_value ON tags_plaintext(value);
            CREATE INDEX ix_tags_plaintext_item_id ON tags_plaintext(item_id);

            /*** Insert metadata ***/
            INSERT INTO metadata(value) VALUES (?1);

            COMMIT;
        "#,
        )
        .persistent(false)
        .bind(metadata)
        .execute(&mut conn)
        .await;

        // TODO: I am not sure force cleanup here is a good idea.
        if let Err(err) = res {
            std::fs::remove_file(db_path)?;
            Err(err)?;
        }

        Ok(())
    }

    ///
    /// Establishes a connection to the SQLite DB with the provided id located in the path
    /// specified in the config. In case of a successful onection returns a Storage object
    /// embedding the connection and the encryption keys that will be used for encryption and
    /// decryption operations.
    ///
    ///
    /// # Arguments
    ///
    ///  * `id` - id of the SQLite DB file
    ///  * `config` - config containing the location of SQLite DB files
    ///  * `credentials` - DB credentials
    ///
    /// # Returns
    ///
    /// Result that can be either:
    ///
    ///  * `(Box<Storage>, Vec<u8>)` - Tuple of `SQLiteStorage` and `encryption keys`
    ///  * `IndyError`
    ///
    /// # Errors
    ///
    /// Any of the following `IndyError` type_ of errors can be throw by this method:
    ///
    ///  * `IndyError::NotFound` - File with the provided id not found
    ///  * `IOError("IO error during storage operation:...")` - Failed connection or SQL query
    async fn open_storage(
        &self,
        id: &str,
        config: Option<&str>,
        _credentials: Option<&str>,
    ) -> IndyResult<Box<dyn WalletStorage>> {
        let config: Option<Config> = config
            .map(serde_json::from_str)
            .map_or(Ok(None), |v| v.map(Some))
            .to_indy(IndyErrorKind::InvalidStructure, "Malformed config json")?;

        let db_path = SQLiteStorageType::_db_path(id, config.as_ref());

        if !db_path.exists() {
            return Err(err_msg(
                IndyErrorKind::WalletNotFound,
                "No wallet database exists",
            ));
        }

        let mut connect_options = SqliteConnectOptions::new()
            .filename(db_path.as_path())
            .journal_mode(SqliteJournalMode::Wal);
        connect_options.disable_statement_logging();

        Ok(Box::new(SQLiteStorage {
            pool: SqlitePoolOptions::default()
                .min_connections(1)
                .max_connections(1)
                .max_lifetime(None)
                .connect_with(connect_options)
                .await?,
        }))
    }
}
