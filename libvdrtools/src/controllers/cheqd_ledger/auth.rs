use crate::controllers::CheqdLedgerController;
use indy_api_types::WalletHandle;
use indy_api_types::errors::{IndyResult, IndyErrorKind, IndyResultExt};
use crate::domain::cheqd_ledger::cosmos_ext::{
    CosmosSignDocExt,
};

use cosmrs::tx::SignDoc;
use indy_wallet::RecordOptions;
use crate::domain::cheqd_keys::Key;

impl CheqdLedgerController {
    pub(crate) async fn auth_build_tx(
        &self,
        pool_alias: &str,
        sender_public_key: &str,
        msg: &[u8],
        account_number: u64,
        sequence_number: u64,
        max_gas: u64,
        max_coin_amount: u64,
        max_coin_denom: &str,
        timeout_height: u64,
        memo: &str,
    ) -> IndyResult<Vec<u8>> {
        trace!("auth_build_tx > pool_alias {:?}, sender_public_key {:?}, msg {:?}, account_number {:?}, sequence_number {:?}, max_gas {:?}, max_coin_amount {:?}, \
               max_coin_denom {:?}, timeout_height {:?}, memo {:?}",
               pool_alias, sender_public_key, msg, account_number, sequence_number, max_gas, max_coin_amount, max_coin_denom, timeout_height, memo);

        let pool = self.cheqd_pool_service.get_config(pool_alias).await?;

        let account_id = self.cheqd_keys_service.get_account_id_from_public_key(sender_public_key)?;

        let (_, sign_doc_bytes) = self
            .cheqd_ledger_service
            .auth_build_tx(
                &pool.chain_id,
                sender_public_key,
                msg,
                account_number,
                sequence_number,
                max_gas,
                max_coin_amount,
                max_coin_denom,
                &account_id,
                timeout_height,
                memo,
            )
            .await?;

        trace!("auth_build_tx <");

        Ok(sign_doc_bytes)
    }

    pub(crate) fn auth_build_query_account(&self, address: &str) -> IndyResult<String> {
        trace!("auth_build_query_account >");
        let query = self
            .cheqd_ledger_service
            .auth_build_query_account(address)?;
        trace!("auth_build_query_account < {:?}", query);
        Ok(query)
    }

    pub(crate) fn auth_parse_query_account_resp(
        &self,
        resp_json: &str,
    ) -> IndyResult<String> {
        trace!(
            "auth_parse_query_account_resp > resp {:?}",
            resp_json
        );
        let result = self
            .cheqd_ledger_service
            .auth_parse_query_account_resp(resp_json)?;
        trace!("auth_parse_query_account_resp < {:?}", result);
        Ok(result)
    }

    pub(crate) async fn sign_tx(&self, wallet_handle: WalletHandle, key_alias: &str, tx: &[u8]) -> IndyResult<Vec<u8>> {
        trace!("sign > wallet_handle {:?}, alias {:?}, tx {:?}", wallet_handle, key_alias, tx);

        let key: Key = self.wallet_service
            .get_indy_object(wallet_handle, &key_alias, &RecordOptions::id_value())
            .await
            .to_indy(IndyErrorKind::WalletItemNotFound, "Can't read cheqd key")?;

        let sign_doc: SignDoc = SignDoc::from_bytes(tx)?;
        let sign_doc_bytes = sign_doc.clone().into_bytes()?;
        let signature = self.cheqd_keys_service.sign(&key, &sign_doc_bytes).await?;
        let signed_tx_bytes = self.cheqd_ledger_service.build_signed_txn(sign_doc, signature)?;

        trace!("sign_txn < signed_tx_bytes {:?}", signed_tx_bytes);
        Ok(signed_tx_bytes)
    }
}
