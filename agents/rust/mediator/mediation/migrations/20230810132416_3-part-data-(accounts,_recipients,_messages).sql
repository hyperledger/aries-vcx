-- Add migration script here

CREATE TABLE IF NOT EXISTS accounts (
    -- metadata (total accounts registered until this account's creation)
    seq_num SERIAL,
    -- sequential time ordered uuid
    -- https://dev.mysql.com/blog-archive/mysql-8-0-uuid-support/
    account_id BINARY(16) DEFAULT (UUID_TO_BIN(UUID(),true)) NOT NULL UNIQUE,
    -- for display purpose
    account_name CHAR(36) GENERATED ALWAYS AS (BIN_TO_UUID(account_id)) VIRTUAL  NOT NULL,
    -- mediator facing pubkey
    -- UNIQUE constraint automatically creates index for fast unique checking; is reused for scanning
    auth_pubkey VARCHAR(64) NOT NULL UNIQUE,
    -- key used to sign messages to aries peer
    our_signing_key VARCHAR(64) NOT NULL,
    -- their complete did_doc, including routing keys etc (useful for wrapping messages in encryption envelope) 
    did_doc JSON NOT NULL,
    PRIMARY KEY(account_id)
);


CREATE TABLE IF NOT EXISTS recipients (
    seq_num SERIAL,
    account_id BINARY(16) NOT NULL,
    recipient_key VARCHAR(64) NOT NULL UNIQUE,
    -- UNIQUE constraint creates index to implement fast unique checking
    -- so explicitly creating index is not needed (and will lead to duplicate index)
    -- If removing UNIQUE above then activate the commented index below
    -- INDEX(recipient_key)
    PRIMARY KEY (account_id, recipient_key),
    FOREIGN KEY (account_id) REFERENCES accounts(account_id)
        ON DELETE CASCADE
);


CREATE TABLE IF NOT EXISTS messages (
    seq_num SERIAL,
    account_id BINARY(16) NOT NULL,
    -- Allow nullable so that recipient_key remove key-updates are possible
    -- without also removing associated messages
    recipient_key VARCHAR(64) NULL, 
    INDEX(account_id, recipient_key),
    message_idb BINARY(16) DEFAULT (UUID_TO_BIN(UUID(),true)) NOT NULL UNIQUE,
    -- Readable message id (for identifying message over wire)
    message_id CHAR(36) GENERATED ALWAYS AS (BIN_TO_UUID(message_idb)) VIRTUAL,
    -- 16MiB limit (medium blob)
    message_data MEDIUMBLOB NOT NULL,
    PRIMARY KEY (account_id, message_idb),
    FOREIGN KEY (account_id) REFERENCES accounts(account_id)
        ON DELETE CASCADE,
    FOREIGN KEY (recipient_key) REFERENCES recipients(recipient_key)
        ON DELETE SET NULL
);

