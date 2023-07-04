use serde::{Deserialize, Serialize};
use vdrtools::domain::pool::VdrtoolsPoolOpenConfig;
use vdrtools::Locator;

use vdrtools::types::errors::IndyErrorKind;

use crate::errors::error::prelude::*;
use crate::global::settings;
use crate::PoolHandle;

pub fn create_pool_ledger_config(pool_name: &str, path: &str) -> VcxCoreResult<()> {
    let res = Locator::instance().pool_controller.create(
        pool_name.into(),
        Some(vdrtools::PoolConfig {
            genesis_txn: path.into(),
        }),
    );

    match res {
        Ok(_) => Ok(()),
        Err(indy) if indy.kind() == IndyErrorKind::PoolConfigAlreadyExists => Ok(()),
        Err(indy) if indy.kind() == IndyErrorKind::IOError => Err(AriesVcxCoreError::from_msg(
            AriesVcxCoreErrorKind::InvalidGenesisTxnPath,
            indy.to_string(),
        )),
        Err(indy) => Err(AriesVcxCoreError::from_msg(
            AriesVcxCoreErrorKind::CreatePoolConfig,
            indy.to_string(),
        )),
    }
}

pub async fn indy_open_pool(pool_name: &str, config: Option<VdrtoolsPoolOpenConfig>) -> VcxCoreResult<i32> {
    Locator::instance()
        .pool_controller
        .set_protocol_version(settings::get_protocol_version())?;

    let handle = Locator::instance().pool_controller.open(pool_name.into(), config).await;

    match handle {
        Ok(handle) => Ok(handle),
        Err(err) => Err(AriesVcxCoreError::from_msg(
            AriesVcxCoreErrorKind::PoolLedgerConnect,
            err.to_string(),
        )),
    }
}

pub async fn indy_close_pool(handle: PoolHandle) -> VcxCoreResult<()> {
    // TODO there was timeout here (before future-based Rust wrapper)

    Locator::instance().pool_controller.close(handle).await?;

    Ok(())
}

pub async fn indy_delete_pool(pool_name: &str) -> VcxCoreResult<()> {
    trace!("delete >>> pool_name: {}", pool_name);

    Locator::instance().pool_controller.delete(pool_name.into()).await?;

    Ok(())
}

pub mod test_utils {
    use std::fs;
    use std::io::Write;
    use std::{env, path::PathBuf};

    use super::*;

    pub fn get_temp_dir_path(filename: &str) -> PathBuf {
        let mut path = env::temp_dir();
        path.push(filename);
        path
    }

    pub fn get_txns(test_pool_ip: &str) -> Vec<String> {
        vec![
            format!(
                r#"{{"reqSignature":{{}},"txn":{{"data":{{"data":{{"alias":"Node1","blskey":"4N8aUNHSgjQVgkpm8nhNEfDf6txHznoYREg9kirmJrkivgL4oSEimFF6nsQ6M41QvhM2Z33nves5vfSn9n1UwNFJBYtWVnHYMATn76vLuL3zU88KyeAYcHfsih3He6UHcXDxcaecHVz6jhCYz1P2UZn2bDVruL5wXpehgBfBaLKm3Ba","blskey_pop":"RahHYiCvoNCtPTrVtP7nMC5eTYrsUA8WjXbdhNc8debh1agE9bGiJxWBXYNFbnJXoXhWFMvyqhqhRoq737YQemH5ik9oL7R4NTTCz2LEZhkgLJzB3QRQqJyBNyv7acbdHrAT8nQ9UkLbaVL9NBpnWXBTw4LEMePaSHEw66RzPNdAX1","client_ip":"{test_pool_ip}","client_port":9702,"node_ip":"{test_pool_ip}","node_port":9701,"services":["VALIDATOR"]}},"dest":"Gw6pDLhcBcoQesN72qfotTgFa7cbuqZpkX3Xo6pLhPhv"}},"metadata":{{"from":"Th7MpTaRZVRYnPiabds81Y"}},"type":"0"}},"txnMetadata":{{"seqNo":1,"txnId":"fea82e10e894419fe2bea7d96296a6d46f50f93f9eeda954ec461b2ed2950b62"}},"ver":"1"}}"#
            ),
            format!(
                r#"{{"reqSignature":{{}},"txn":{{"data":{{"data":{{"alias":"Node2","blskey":"37rAPpXVoxzKhz7d9gkUe52XuXryuLXoM6P6LbWDB7LSbG62Lsb33sfG7zqS8TK1MXwuCHj1FKNzVpsnafmqLG1vXN88rt38mNFs9TENzm4QHdBzsvCuoBnPH7rpYYDo9DZNJePaDvRvqJKByCabubJz3XXKbEeshzpz4Ma5QYpJqjk","blskey_pop":"Qr658mWZ2YC8JXGXwMDQTzuZCWF7NK9EwxphGmcBvCh6ybUuLxbG65nsX4JvD4SPNtkJ2w9ug1yLTj6fgmuDg41TgECXjLCij3RMsV8CwewBVgVN67wsA45DFWvqvLtu4rjNnE9JbdFTc1Z4WCPA3Xan44K1HoHAq9EVeaRYs8zoF5","client_ip":"{test_pool_ip}","client_port":9704,"node_ip":"{test_pool_ip}","node_port":9703,"services":["VALIDATOR"]}},"dest":"8ECVSk179mjsjKRLWiQtssMLgp6EPhWXtaYyStWPSGAb"}},"metadata":{{"from":"EbP4aYNeTHL6q385GuVpRV"}},"type":"0"}},"txnMetadata":{{"seqNo":2,"txnId":"1ac8aece2a18ced660fef8694b61aac3af08ba875ce3026a160acbc3a3af35fc"}},"ver":"1"}}"#
            ),
            format!(
                r#"{{"reqSignature":{{}},"txn":{{"data":{{"data":{{"alias":"Node3","blskey":"3WFpdbg7C5cnLYZwFZevJqhubkFALBfCBBok15GdrKMUhUjGsk3jV6QKj6MZgEubF7oqCafxNdkm7eswgA4sdKTRc82tLGzZBd6vNqU8dupzup6uYUf32KTHTPQbuUM8Yk4QFXjEf2Usu2TJcNkdgpyeUSX42u5LqdDDpNSWUK5deC5","blskey_pop":"QwDeb2CkNSx6r8QC8vGQK3GRv7Yndn84TGNijX8YXHPiagXajyfTjoR87rXUu4G4QLk2cF8NNyqWiYMus1623dELWwx57rLCFqGh7N4ZRbGDRP4fnVcaKg1BcUxQ866Ven4gw8y4N56S5HzxXNBZtLYmhGHvDtk6PFkFwCvxYrNYjh","client_ip":"{test_pool_ip}","client_port":9706,"node_ip":"{test_pool_ip}","node_port":9705,"services":["VALIDATOR"]}},"dest":"DKVxG2fXXTU8yT5N7hGEbXB3dfdAnYv1JczDUHpmDxya"}},"metadata":{{"from":"4cU41vWW82ArfxJxHkzXPG"}},"type":"0"}},"txnMetadata":{{"seqNo":3,"txnId":"7e9f355dffa78ed24668f0e0e369fd8c224076571c51e2ea8be5f26479edebe4"}},"ver":"1"}}"#
            ),
            format!(
                r#"{{"reqSignature":{{}},"txn":{{"data":{{"data":{{"alias":"Node4","blskey":"2zN3bHM1m4rLz54MJHYSwvqzPchYp8jkHswveCLAEJVcX6Mm1wHQD1SkPYMzUDTZvWvhuE6VNAkK3KxVeEmsanSmvjVkReDeBEMxeDaayjcZjFGPydyey1qxBHmTvAnBKoPydvuTAqx5f7YNNRAdeLmUi99gERUU7TD8KfAa6MpQ9bw","blskey_pop":"RPLagxaR5xdimFzwmzYnz4ZhWtYQEj8iR5ZU53T2gitPCyCHQneUn2Huc4oeLd2B2HzkGnjAff4hWTJT6C7qHYB1Mv2wU5iHHGFWkhnTX9WsEAbunJCV2qcaXScKj4tTfvdDKfLiVuU2av6hbsMztirRze7LvYBkRHV3tGwyCptsrP","client_ip":"{test_pool_ip}","client_port":9708,"node_ip":"{test_pool_ip}","node_port":9707,"services":["VALIDATOR"]}},"dest":"4PS3EDQ3dW1tci1Bp6543CfuuebjFrg36kLAUcskGfaA"}},"metadata":{{"from":"TWwCRQRZ2ZHMJFn9TzLp7W"}},"type":"0"}},"txnMetadata":{{"seqNo":4,"txnId":"aa5e817d7cc626170eca175822029339a444eb0ee8f0bd20d3b0b76e566fb008"}},"ver":"1"}}"#
            ),
        ]
    }

    pub fn create_testpool_genesis_txn_file(genesis_file_path: &str) {
        let test_pool_ip = env::var("TEST_POOL_IP").unwrap_or("127.0.0.1".to_string());

        let node_txns = get_txns(&test_pool_ip);
        let txn_file_data = node_txns[0..4].join("\n");
        let mut f = fs::File::create(&genesis_file_path).unwrap();
        f.write_all(txn_file_data.as_bytes()).unwrap();
        f.flush().unwrap();
        f.sync_all().unwrap();
    }
}

#[derive(Clone, Debug, Default, Builder, Serialize, Deserialize)]
#[builder(setter(into, strip_option), default)]
pub struct PoolConfig {
    pub genesis_path: String,
    pub pool_name: Option<String>,
    pub pool_config: Option<VdrtoolsPoolOpenConfig>,
}
