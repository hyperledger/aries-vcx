#![cfg_attr(feature = "fatal_warnings", deny(warnings))]

#[macro_use]
extern crate derivative;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate serde_json;

#[macro_use]
extern crate log;

#[macro_use]
mod utils;

use utils::{constants::*, metrics, wallet, Setup};

mod collect {
    use super::*;

    use std::collections::HashMap;

    use serde_json::Value;

    #[test]
    fn test_metrics_schema() {
        let setup = Setup::empty();
        let config = config(&setup.name);
        wallet::create_wallet(&config, WALLET_CREDENTIALS).unwrap();

        let result_metrics = metrics::collect_metrics().unwrap();

        let metrics_map = serde_json::from_str::<HashMap<String, Value>>(&result_metrics)
            .expect("Top level object should be a dictionary");

        for metrics_set in metrics_map.values() {
            let metrics_set = metrics_set
                .as_array()
                .expect("Metrics set should be an array");

            for metric in metrics_set.iter() {
                let metrics = metric.as_object().expect("Metrics should be an object");
                metrics.contains_key("value");
                metrics.contains_key("tags");
            }
        }
    }

    #[test]
    fn collect_metrics_contains_wallet_service_statistics() {
        let result_metrics = metrics::collect_metrics().unwrap();
        let metrics_map = serde_json::from_str::<HashMap<String, Value>>(&result_metrics).unwrap();

        assert!(metrics_map.contains_key("wallet_count"));

        let wallet_count = metrics_map.get("wallet_count").unwrap().as_array().unwrap();

        assert!(wallet_count.contains(&json!({"tags":{"label":"opened"},"value":0})));
        assert!(wallet_count.contains(&json!({"tags":{"label":"opened_ids"},"value":0})));
        assert!(wallet_count.contains(&json!({"tags":{"label":"pending_for_import"},"value":0})));
        assert!(wallet_count.contains(&json!({"tags":{"label":"pending_for_open"},"value":0})));
    }

    #[test]
    fn collect_metrics_includes_commands_count() {
        let setup = Setup::empty();
        let config = config(&setup.name);
        wallet::create_wallet(&config, WALLET_CREDENTIALS).unwrap();

        let result_metrics = metrics::collect_metrics().unwrap();
        let metrics_map = serde_json::from_str::<HashMap<String, Value>>(&result_metrics).unwrap();

        let coummand_count_json = metrics_map.get("command_duration_ms_count").unwrap();

        let commands_count = coummand_count_json.as_array().unwrap().to_owned();

        assert!(commands_count.contains(&json!({"tags":{"command": "pairwise_command_pairwise_exists", "stage": "executed"} ,"value": 0})));
        assert!(commands_count.contains(&json!({"tags":{"command": "payments_command_build_set_txn_fees_req_ack", "stage": "executed"} ,"value": 0})));

        let mut queued = commands_count
            .into_iter()
            .filter(|val| val["tags"]["stage"].as_str() == Some("queued"))
            .collect::<Vec<Value>>();

        assert_eq!(queued.len(), 1);

        let queued = queued.remove(0);

        assert_eq!(queued["tags"]["stage"].as_str().unwrap(), "queued");
        assert!(queued["tags"].get("command").is_none());
        assert_eq!(queued["tags"].as_object().unwrap().keys().len(), 1);
        assert!(queued["value"].as_u64().unwrap() > 0);
    }

    #[test]
    fn collect_metrics_includes_commands_duration_ms() {
        let setup = Setup::empty();
        let config = config(&setup.name);
        wallet::create_wallet(&config, WALLET_CREDENTIALS).unwrap();

        let result_metrics = metrics::collect_metrics().unwrap();
        let metrics_map = serde_json::from_str::<HashMap<String, Value>>(&result_metrics).unwrap();

        let coummand_duration_ms_json = metrics_map.get("command_duration_ms_sum").unwrap();

        let commands_duration_ms = coummand_duration_ms_json.as_array().unwrap().to_owned();

        assert!(commands_duration_ms.contains(&json!({"tags":{"command": "pairwise_command_pairwise_exists", "stage": "executed"} ,"value": 0})));
        assert!(commands_duration_ms.contains(&json!({"tags":{"command": "payments_command_build_set_txn_fees_req_ack", "stage": "executed"} ,"value": 0})));

        let mut queued = commands_duration_ms
            .into_iter()
            .filter(|val| val["tags"]["stage"].as_str() == Some("queued"))
            .collect::<Vec<Value>>();

        assert_eq!(queued.len(), 1);

        let queued = queued.remove(0);

        assert_eq!(queued["tags"]["stage"].as_str().unwrap(), "queued");
        assert!(queued["tags"].get("command").is_none());
        assert_eq!(queued["tags"].as_object().unwrap().keys().len(), 1);
        assert!(queued["value"].as_u64().is_some());
    }

    #[test]
    fn collect_metrics_includes_commands_duration_ms_bucket() {
        let setup = Setup::empty();
        let config = config(&setup.name);
        wallet::create_wallet(&config, WALLET_CREDENTIALS).unwrap();

        let result_metrics = metrics::collect_metrics().unwrap();
        let metrics_map = serde_json::from_str::<HashMap<String, Value>>(&result_metrics).unwrap();

        let command_duration_ms_bucket_json =
            metrics_map.get("command_duration_ms_bucket").unwrap();

        let commands_duration_ms_bucket = command_duration_ms_bucket_json
            .as_array()
            .unwrap()
            .to_owned();

        assert!(commands_duration_ms_bucket.contains(&json!({"tags":{"command": "pairwise_command_pairwise_exists", "stage": "executed", "le": "0.5"} ,"value": 0})));
        assert!(commands_duration_ms_bucket.contains(&json!({"tags":{"command": "payments_command_build_set_txn_fees_req_ack", "stage": "executed", "le": "1"} ,"value": 0})));

        let mut queued = commands_duration_ms_bucket
            .into_iter()
            .filter(|val| val["tags"]["stage"].as_str() == Some("queued"))
            .collect::<Vec<Value>>();
        assert_eq!(queued.len(), 16);

        let queued = queued.remove(0);
        assert_eq!(queued["tags"]["stage"].as_str().unwrap(), "queued");
        assert_eq!(queued["tags"]["le"].as_str().unwrap(), "+Inf");
        assert!(queued["tags"].get("command").is_none());
        assert_eq!(queued["tags"].as_object().unwrap().keys().len(), 2);
        assert!(queued["value"].as_u64().unwrap() > 0);
    }

    fn config(name: &str) -> String {
        json!({ "id": name }).to_string()
    }
}
