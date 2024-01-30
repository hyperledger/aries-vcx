use indy_api_types::SearchHandle;
use vdrtools::{Locator, WalletHandle};

use crate::errors::error::VcxCoreResult;

pub async fn delete_wallet_record(
    wallet_handle: WalletHandle,
    xtype: &str,
    id: &str,
) -> VcxCoreResult<()> {
    trace!(
        "delete_record >>> xtype: {}, id: {}",
        secret!(&xtype),
        secret!(&id)
    );

    Locator::instance()
        .non_secret_controller
        .delete_record(wallet_handle, xtype.into(), id.into())
        .await?;

    Ok(())
}

// TODO - FUTURE - revert to pub(crate) after libvcx dependency is fixed
pub async fn open_search_wallet(
    wallet_handle: WalletHandle,
    xtype: &str,
    query: &str,
    options: &str,
) -> VcxCoreResult<SearchHandle> {
    trace!(
        "open_search >>> xtype: {}, query: {}, options: {}",
        secret!(&xtype),
        query,
        options
    );

    let res = Locator::instance()
        .non_secret_controller
        .open_search(wallet_handle, xtype.into(), query.into(), options.into())
        .await?;

    Ok(res)
}

// TODO - FUTURE - revert to pub(crate) after libvcx dependency is fixed
pub async fn fetch_next_records_wallet(
    wallet_handle: WalletHandle,
    search_handle: SearchHandle,
    count: usize,
) -> VcxCoreResult<String> {
    trace!(
        "fetch_next_records >>> search_handle: {:?}, count: {}",
        search_handle,
        count
    );

    let res = Locator::instance()
        .non_secret_controller
        .fetch_search_next_records(wallet_handle, search_handle, count)
        .await?;

    Ok(res)
}

// TODO - FUTURE - revert to pub(crate) after libvcx dependency is fixed
pub async fn close_search_wallet(search_handle: SearchHandle) -> VcxCoreResult<()> {
    trace!("close_search >>> search_handle: {:?}", search_handle);

    Locator::instance()
        .non_secret_controller
        .close_search(search_handle)
        .await?;

    Ok(())
}
