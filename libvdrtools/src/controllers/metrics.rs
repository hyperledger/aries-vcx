use crate::services::MetricsService;
use indy_api_types::errors::prelude::*;
use indy_wallet::WalletService;
use std::sync::Arc;
use serde_json::{Map, Value};

const OPENED_WALLETS_COUNT: &str = "opened";
const OPENED_WALLET_IDS_COUNT: &str = "opened_ids";
const PENDING_FOR_IMPORT_WALLETS_COUNT: &str = "pending_for_import";
const PENDING_FOR_OPEN_WALLETS_COUNT: &str = "pending_for_open";

pub struct MetricsController {
    wallet_service:Arc<WalletService>,
    metrics_service:Arc<MetricsService>,
}

impl MetricsController {
    pub fn new(
        wallet_service:Arc<WalletService>,
        metrics_service:Arc<MetricsService>,
    ) -> MetricsController {
        MetricsController {
            wallet_service,
            metrics_service,
        }
    }

    pub async fn collect(&self) -> IndyResult<String> {
        trace!("_collect >>>");
        let mut metrics_map = serde_json::Map::new();
        self.append_wallet_metrics(&mut metrics_map).await?;
        self.append_wallet_cache_metrics(&mut metrics_map).await?;
        self.metrics_service
            .append_command_metrics(&mut metrics_map).await?;
        let res = serde_json::to_string(&metrics_map)
            .to_indy(IndyErrorKind::InvalidState, "Can't serialize a metrics map")?;

        trace!("_collect <<< res: {:?}", res);
        debug!("collecting metrics from command thread");
        Ok(res)
    }

    async fn append_wallet_metrics(&self, metrics_map: &mut Map<String, Value>) -> IndyResult<()> {
        let mut wallet_count = Vec::new();

        wallet_count.push(self.get_labeled_metric_json(
            OPENED_WALLETS_COUNT,
            self.wallet_service.get_wallets_count().await
        )?);

        wallet_count.push(self.get_labeled_metric_json(
            OPENED_WALLET_IDS_COUNT,
            self.wallet_service.get_wallet_ids_count().await
        )?);

        wallet_count.push(self.get_labeled_metric_json(
            PENDING_FOR_IMPORT_WALLETS_COUNT,
            self.wallet_service.get_pending_for_import_count().await
        )?);

        wallet_count.push(self.get_labeled_metric_json(
        PENDING_FOR_OPEN_WALLETS_COUNT,
        self.wallet_service.get_pending_for_open_count().await
        )?);

        metrics_map.insert(
            String::from("wallet_count"),
            serde_json::to_value(wallet_count)
                .to_indy(IndyErrorKind::IOError, "Unable to convert json")?,
        );

        Ok(())
    }

    async fn append_wallet_cache_metrics(&self, metrics_map: &mut Map<String, Value>) -> IndyResult<()> {
        let mut cache_metrics = Vec::new();

        let metrics_data = self.wallet_service.get_wallet_cache_hit_metrics_data().await;

        for (type_, data) in metrics_data.into_iter() {
            cache_metrics.push(
                self.get_typed_metric_json(&type_, "hit", data.get_hit())?
            );
            cache_metrics.push(
                self.get_typed_metric_json(&type_, "miss", data.get_miss())?
            );
            cache_metrics.push(
                self.get_typed_metric_json(&type_, "uncached", data.get_not_cached())?
            );
        }

        metrics_map.insert(
            String::from("wallet_cache_requests_total"),
            serde_json::to_value(cache_metrics)
                .to_indy(IndyErrorKind::IOError, "Unable to convert json")?,
        );

        Ok(())
    }

    fn get_labeled_metric_json(&self, label: &str, value: usize) -> IndyResult<Value> {
        MetricsService::get_metric_json(value, map!("label".to_owned() => label.to_owned()))
    }

    fn get_typed_metric_json(&self, type_: &str, result: &str, value: usize) -> IndyResult<Value> {
        MetricsService::get_metric_json(
            value,
            map!("type".to_owned() => type_.to_owned(), "result".to_owned() => result.to_owned())
        )
    }
}