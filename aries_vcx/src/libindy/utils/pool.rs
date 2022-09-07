use indy::{pool, ErrorCode};
use indy_sys::PoolHandle;

use crate::error::prelude::*;
use crate::global;
use crate::global::settings;

pub async fn set_protocol_version() -> VcxResult<()> {
    pool::set_protocol_version(settings::get_protocol_version()).await?;

    Ok(())
}

pub async fn create_pool_ledger_config(pool_name: &str, path: &str) -> VcxResult<()> {
    let pool_config = json!({ "genesis_txn": path }).to_string();

    match pool::create_pool_ledger_config(pool_name, Some(&pool_config)).await {
        Ok(()) => Ok(()),
        Err(err) => match err.error_code {
            ErrorCode::PoolLedgerConfigAlreadyExistsError => Ok(()),
            ErrorCode::CommonIOError => Err(err.to_vcx(
                VcxErrorKind::InvalidGenesisTxnPath,
                "Pool genesis file is invalid or does not exist",
            )),
            _ => Err(err.to_vcx(VcxErrorKind::CreatePoolConfig, "Indy error occurred")),
        },
    }
}

pub async fn open_pool_ledger(pool_name: &str, config: Option<&str>) -> VcxResult<i32> {
    set_protocol_version().await?;

    let handle = pool::open_pool_ledger(pool_name, config)
        .await
        .map_err(|err| match err.error_code {
            ErrorCode::PoolLedgerNotCreatedError => err.to_vcx(
                VcxErrorKind::PoolLedgerConnect,
                format!("Pool \"{}\" does not exist.", pool_name),
            ),
            ErrorCode::PoolLedgerTimeout => err.to_vcx(
                VcxErrorKind::PoolLedgerConnect,
                format!("Can not connect to Pool \"{}\".", pool_name),
            ),
            ErrorCode::PoolIncompatibleProtocolVersion => {
                let protocol_version = settings::get_protocol_version();
                err.to_vcx(
                    VcxErrorKind::PoolLedgerConnect,
                    format!(
                        "Pool \"{}\" is not compatible with Protocol Version \"{}\".",
                        pool_name, protocol_version
                    ),
                )
            }
            ErrorCode::CommonInvalidState => err.to_vcx(
                VcxErrorKind::PoolLedgerConnect,
                "Geneses transactions are invalid.".to_string(),
            ),
            error_code => err.to_vcx(VcxErrorKind::LibndyError(error_code as u32), "Indy error occurred"),
        })?;
    Ok(handle)
}

pub async fn close() -> VcxResult<()> {
    let handle = global::pool::get_main_pool_handle()?;

    //TODO there was timeout here (before future-based Rust wrapper)
    pool::close_pool_ledger(handle).await?;

    global::pool::reset_main_pool_handle();

    Ok(())
}

pub async fn delete(pool_name: &str) -> VcxResult<()> {
    trace!("delete >>> pool_name: {}", pool_name);

    if settings::indy_mocks_enabled() {
        global::pool::set_main_pool_handle(None);
        return Ok(());
    }

    pool::delete_pool_ledger(pool_name).await?;

    Ok(())
}

#[cfg(feature = "test_utils")]
pub mod test_utils {
    use std::fs;
    use std::io::Write;

    use crate::global::pool::set_main_pool_handle;
    use crate::utils::{
        constants::{GENESIS_PATH, POOL},
        get_temp_dir_path,
    };

    use super::*;

    pub async fn create_test_ledger_config() {
        create_tmp_genesis_txn_file();
        create_pool_ledger_config(POOL, get_temp_dir_path(GENESIS_PATH).to_str().unwrap())
            .await
            .unwrap();
    }

    pub async fn delete_named_test_pool(pool_name: &str) {
        close().await.ok();
        delete(pool_name).await.unwrap();
    }

    pub async fn delete_test_pool() {
        close().await.ok();
        delete(POOL).await.unwrap();
    }

    pub async fn open_test_pool() -> PoolHandle {
        create_test_ledger_config().await;
        let handle = open_pool_ledger(POOL, None).await.unwrap();
        set_main_pool_handle(Some(handle));
        handle
    }

    pub fn get_txns(test_pool_ip: &str) -> Vec<String> {
        vec![
            format!(
                r#"{{"reqSignature":{{}},"txn":{{"data":{{"data":{{"alias":"Node1","blskey":"4N8aUNHSgjQVgkpm8nhNEfDf6txHznoYREg9kirmJrkivgL4oSEimFF6nsQ6M41QvhM2Z33nves5vfSn9n1UwNFJBYtWVnHYMATn76vLuL3zU88KyeAYcHfsih3He6UHcXDxcaecHVz6jhCYz1P2UZn2bDVruL5wXpehgBfBaLKm3Ba","blskey_pop":"RahHYiCvoNCtPTrVtP7nMC5eTYrsUA8WjXbdhNc8debh1agE9bGiJxWBXYNFbnJXoXhWFMvyqhqhRoq737YQemH5ik9oL7R4NTTCz2LEZhkgLJzB3QRQqJyBNyv7acbdHrAT8nQ9UkLbaVL9NBpnWXBTw4LEMePaSHEw66RzPNdAX1","client_ip":"{}","client_port":9702,"node_ip":"{}","node_port":9701,"services":["VALIDATOR"]}},"dest":"Gw6pDLhcBcoQesN72qfotTgFa7cbuqZpkX3Xo6pLhPhv"}},"metadata":{{"from":"Th7MpTaRZVRYnPiabds81Y"}},"type":"0"}},"txnMetadata":{{"seqNo":1,"txnId":"fea82e10e894419fe2bea7d96296a6d46f50f93f9eeda954ec461b2ed2950b62"}},"ver":"1"}}"#,
                test_pool_ip, test_pool_ip
            ),
            format!(
                r#"{{"reqSignature":{{}},"txn":{{"data":{{"data":{{"alias":"Node2","blskey":"37rAPpXVoxzKhz7d9gkUe52XuXryuLXoM6P6LbWDB7LSbG62Lsb33sfG7zqS8TK1MXwuCHj1FKNzVpsnafmqLG1vXN88rt38mNFs9TENzm4QHdBzsvCuoBnPH7rpYYDo9DZNJePaDvRvqJKByCabubJz3XXKbEeshzpz4Ma5QYpJqjk","blskey_pop":"Qr658mWZ2YC8JXGXwMDQTzuZCWF7NK9EwxphGmcBvCh6ybUuLxbG65nsX4JvD4SPNtkJ2w9ug1yLTj6fgmuDg41TgECXjLCij3RMsV8CwewBVgVN67wsA45DFWvqvLtu4rjNnE9JbdFTc1Z4WCPA3Xan44K1HoHAq9EVeaRYs8zoF5","client_ip":"{}","client_port":9704,"node_ip":"{}","node_port":9703,"services":["VALIDATOR"]}},"dest":"8ECVSk179mjsjKRLWiQtssMLgp6EPhWXtaYyStWPSGAb"}},"metadata":{{"from":"EbP4aYNeTHL6q385GuVpRV"}},"type":"0"}},"txnMetadata":{{"seqNo":2,"txnId":"1ac8aece2a18ced660fef8694b61aac3af08ba875ce3026a160acbc3a3af35fc"}},"ver":"1"}}"#,
                test_pool_ip, test_pool_ip
            ),
            format!(
                r#"{{"reqSignature":{{}},"txn":{{"data":{{"data":{{"alias":"Node3","blskey":"3WFpdbg7C5cnLYZwFZevJqhubkFALBfCBBok15GdrKMUhUjGsk3jV6QKj6MZgEubF7oqCafxNdkm7eswgA4sdKTRc82tLGzZBd6vNqU8dupzup6uYUf32KTHTPQbuUM8Yk4QFXjEf2Usu2TJcNkdgpyeUSX42u5LqdDDpNSWUK5deC5","blskey_pop":"QwDeb2CkNSx6r8QC8vGQK3GRv7Yndn84TGNijX8YXHPiagXajyfTjoR87rXUu4G4QLk2cF8NNyqWiYMus1623dELWwx57rLCFqGh7N4ZRbGDRP4fnVcaKg1BcUxQ866Ven4gw8y4N56S5HzxXNBZtLYmhGHvDtk6PFkFwCvxYrNYjh","client_ip":"{}","client_port":9706,"node_ip":"{}","node_port":9705,"services":["VALIDATOR"]}},"dest":"DKVxG2fXXTU8yT5N7hGEbXB3dfdAnYv1JczDUHpmDxya"}},"metadata":{{"from":"4cU41vWW82ArfxJxHkzXPG"}},"type":"0"}},"txnMetadata":{{"seqNo":3,"txnId":"7e9f355dffa78ed24668f0e0e369fd8c224076571c51e2ea8be5f26479edebe4"}},"ver":"1"}}"#,
                test_pool_ip, test_pool_ip
            ),
            format!(
                r#"{{"reqSignature":{{}},"txn":{{"data":{{"data":{{"alias":"Node4","blskey":"2zN3bHM1m4rLz54MJHYSwvqzPchYp8jkHswveCLAEJVcX6Mm1wHQD1SkPYMzUDTZvWvhuE6VNAkK3KxVeEmsanSmvjVkReDeBEMxeDaayjcZjFGPydyey1qxBHmTvAnBKoPydvuTAqx5f7YNNRAdeLmUi99gERUU7TD8KfAa6MpQ9bw","blskey_pop":"RPLagxaR5xdimFzwmzYnz4ZhWtYQEj8iR5ZU53T2gitPCyCHQneUn2Huc4oeLd2B2HzkGnjAff4hWTJT6C7qHYB1Mv2wU5iHHGFWkhnTX9WsEAbunJCV2qcaXScKj4tTfvdDKfLiVuU2av6hbsMztirRze7LvYBkRHV3tGwyCptsrP","client_ip":"{}","client_port":9708,"node_ip":"{}","node_port":9707,"services":["VALIDATOR"]}},"dest":"4PS3EDQ3dW1tci1Bp6543CfuuebjFrg36kLAUcskGfaA"}},"metadata":{{"from":"TWwCRQRZ2ZHMJFn9TzLp7W"}},"type":"0"}},"txnMetadata":{{"seqNo":4,"txnId":"aa5e817d7cc626170eca175822029339a444eb0ee8f0bd20d3b0b76e566fb008"}},"ver":"1"}}"#,
                test_pool_ip, test_pool_ip
            ),
        ]
    }

    pub fn create_tmp_genesis_txn_file() -> String {
        let test_pool_ip = ::std::env::var("TEST_POOL_IP").unwrap_or("127.0.0.1".to_string());

        let node_txns = get_txns(&test_pool_ip);
        let txn_file_data = node_txns[0..4].join("\n");
        let file_path = String::from(get_temp_dir_path(GENESIS_PATH).to_str().unwrap());
        let mut f = fs::File::create(&file_path).unwrap();
        f.write_all(txn_file_data.as_bytes()).unwrap();
        f.flush().unwrap();
        f.sync_all().unwrap();
        file_path
    }
}

#[derive(Clone, Debug, Default, Builder, Serialize, Deserialize)]
#[builder(setter(into, strip_option), default)]
pub struct PoolConfig {
    pub genesis_path: String,
    pub pool_name: Option<String>,
    pub pool_config: Option<String>,
}
