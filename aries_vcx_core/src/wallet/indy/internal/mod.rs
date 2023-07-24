use crate::errors::error::VcxCoreResult;
use crate::global::settings;
use crate::{SearchHandle, WalletHandle};
use vdrtools::Locator;

pub(crate) async fn add_wallet_record(
    wallet_handle: WalletHandle,
    xtype: &str,
    id: &str,
    value: &str,
    tags: Option<&str>,
) -> VcxCoreResult<()> {
    trace!(
        "add_record >>> xtype: {}, id: {}, value: {}, tags: {:?}",
        secret!(&xtype),
        secret!(&id),
        secret!(&value),
        secret!(&tags)
    );

    if settings::indy_mocks_enabled() {
        return Ok(());
    }

    Locator::instance()
        .non_secret_controller
        .add_record(
            wallet_handle,
            xtype.into(),
            id.into(),
            value.into(),
            tags.map(serde_json::from_str).transpose()?,
        )
        .await?;

    Ok(())
}

pub(crate) async fn get_wallet_record(
    wallet_handle: WalletHandle,
    xtype: &str,
    id: &str,
    options: &str,
) -> VcxCoreResult<String> {
    trace!(
        "get_record >>> xtype: {}, id: {}, options: {}",
        secret!(&xtype),
        secret!(&id),
        options
    );

    if settings::indy_mocks_enabled() {
        return Ok(r#"{"id":"123","type":"record type","value":"record value","tags":null}"#.to_string());
    }

    let res = Locator::instance()
        .non_secret_controller
        .get_record(wallet_handle, xtype.into(), id.into(), options.into())
        .await?;

    Ok(res)
}

pub async fn delete_wallet_record(wallet_handle: WalletHandle, xtype: &str, id: &str) -> VcxCoreResult<()> {
    trace!("delete_record >>> xtype: {}, id: {}", secret!(&xtype), secret!(&id));

    if settings::indy_mocks_enabled() {
        return Ok(());
    }

    Locator::instance()
        .non_secret_controller
        .delete_record(wallet_handle, xtype.into(), id.into())
        .await?;

    Ok(())
}

pub(crate) async fn update_wallet_record_value(
    wallet_handle: WalletHandle,
    xtype: &str,
    id: &str,
    value: &str,
) -> VcxCoreResult<()> {
    trace!(
        "update_record_value >>> xtype: {}, id: {}, value: {}",
        secret!(&xtype),
        secret!(&id),
        secret!(&value)
    );

    if settings::indy_mocks_enabled() {
        return Ok(());
    }

    Locator::instance()
        .non_secret_controller
        .update_record_value(wallet_handle, xtype.into(), id.into(), value.into())
        .await?;

    Ok(())
}

pub(crate) async fn add_wallet_record_tags(
    wallet_handle: WalletHandle,
    xtype: &str,
    id: &str,
    tags: &str,
) -> VcxCoreResult<()> {
    trace!(
        "add_record_tags >>> xtype: {}, id: {}, tags: {:?}",
        secret!(&xtype),
        secret!(&id),
        secret!(&tags)
    );

    if settings::indy_mocks_enabled() {
        return Ok(());
    }

    Locator::instance()
        .non_secret_controller
        .add_record_tags(wallet_handle, xtype.into(), id.into(), serde_json::from_str(tags)?)
        .await?;

    Ok(())
}

pub(crate) async fn update_wallet_record_tags(
    wallet_handle: WalletHandle,
    xtype: &str,
    id: &str,
    tags: &str,
) -> VcxCoreResult<()> {
    trace!(
        "update_record_tags >>> xtype: {}, id: {}, tags: {}",
        secret!(&xtype),
        secret!(&id),
        secret!(&tags)
    );

    if settings::indy_mocks_enabled() {
        return Ok(());
    }

    Locator::instance()
        .non_secret_controller
        .update_record_tags(wallet_handle, xtype.into(), id.into(), serde_json::from_str(tags)?)
        .await?;

    Ok(())
}

pub(crate) async fn delete_wallet_record_tags(
    wallet_handle: WalletHandle,
    xtype: &str,
    id: &str,
    tag_names: &str,
) -> VcxCoreResult<()> {
    trace!(
        "delete_record_tags >>> xtype: {}, id: {}, tag_names: {}",
        secret!(&xtype),
        secret!(&id),
        secret!(&tag_names)
    );

    if settings::indy_mocks_enabled() {
        return Ok(());
    }

    Locator::instance()
        .non_secret_controller
        .delete_record_tags(wallet_handle, xtype.into(), id.into(), tag_names.into())
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

    if settings::indy_mocks_enabled() {
        return Ok(SearchHandle(1));
    }

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

    if settings::indy_mocks_enabled() {
        return Ok(String::from("{}"));
    }

    let res = Locator::instance()
        .non_secret_controller
        .fetch_search_next_records(wallet_handle, search_handle, count)
        .await?;

    Ok(res)
}

// TODO - FUTURE - revert to pub(crate) after libvcx dependency is fixed
pub async fn close_search_wallet(search_handle: SearchHandle) -> VcxCoreResult<()> {
    trace!("close_search >>> search_handle: {:?}", search_handle);

    if settings::indy_mocks_enabled() {
        return Ok(());
    }

    Locator::instance()
        .non_secret_controller
        .close_search(search_handle)
        .await?;

    Ok(())
}
