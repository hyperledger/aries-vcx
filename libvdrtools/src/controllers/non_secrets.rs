use std::{collections::HashMap, sync::Arc};

use futures::lock::Mutex;
use indy_api_types::{domain::wallet::Tags, errors::prelude::*, SearchHandle, WalletHandle};
use indy_utils::next_search_handle;
use indy_wallet::{RecordOptions, SearchOptions, WalletRecord, WalletSearch, WalletService};

pub struct NonSecretsController {
    wallet_service: Arc<WalletService>,
    searches: Mutex<HashMap<SearchHandle, Arc<Mutex<WalletSearch>>>>,
}

impl NonSecretsController {
    pub(crate) fn new(wallet_service: Arc<WalletService>) -> NonSecretsController {
        NonSecretsController {
            wallet_service,
            searches: Mutex::new(HashMap::new()),
        }
    }

    /// Create a new non-secret record in the wallet
    ///
    /// #Params
    /// wallet_handle: wallet handle (created by open_wallet)
    /// type_: allows to separate different record types collections
    /// id: the id of record
    /// value: the value of record
    /// tags_json: (optional) the record tags used for search and storing meta information as json:
    ///   {
    ///     "tagName1": <str>, // string tag (will be stored encrypted)
    ///     "tagName2": <str>, // string tag (will be stored encrypted)
    ///     "~tagName3": <str>, // string tag (will be stored un-encrypted)
    ///     "~tagName4": <str>, // string tag (will be stored un-encrypted)
    ///   }
    ///   Note that null means no tags
    ///   If tag name starts with "~" the tag will be stored un-encrypted that will allow
    ///   usage of this tag in complex search queries (comparison, predicates)
    ///   Encrypted tags can be searched only for exact matching
    // TODO: change to String -> &str
    pub async fn add_record(
        &self,
        wallet_handle: WalletHandle,
        type_: String,
        id: String,
        value: String,
        tags: Option<Tags>,
    ) -> IndyResult<()> {
        trace!(
            "add_record > wallet_handle {:?} type_ {:?} \
                id {:?} value {:?} tags {:?}",
            wallet_handle,
            type_,
            id,
            value,
            tags
        );

        self._check_type(&type_)?;

        self.wallet_service
            .add_record(
                wallet_handle,
                &type_,
                &id,
                &value,
                &tags.unwrap_or_else(|| Tags::new()),
            )
            .await?;

        let res = Ok(());
        trace!("add_record < {:?}", res);
        res
    }

    /// Update a non-secret wallet record value
    ///
    /// #Params

    /// wallet_handle: wallet handle (created by open_wallet)
    /// type_: allows to separate different record types collections
    /// id: the id of record
    /// value: the new value of record
    pub async fn update_record_value(
        &self,
        wallet_handle: WalletHandle,
        type_: String,
        id: String,
        value: String,
    ) -> IndyResult<()> {
        trace!(
            "update_record_value > wallet_handle {:?} type_ {:?} \
                id {:?} value {:?}",
            wallet_handle,
            type_,
            id,
            value
        );

        self._check_type(&type_)?;

        self.wallet_service
            .update_record_value(wallet_handle, &type_, &id, &value)
            .await?;

        let res = Ok(());
        trace!("update_record_value < {:?}", res);
        res
    }

    /// Update a non-secret wallet record tags
    ///
    /// #Params

    /// wallet_handle: wallet handle (created by open_wallet)
    /// type_: allows to separate different record types collections
    /// id: the id of record
    /// tags_json: the record tags used for search and storing meta information as json:
    ///   {
    ///     "tagName1": <str>, // string tag (will be stored encrypted)
    ///     "tagName2": <str>, // string tag (will be stored encrypted)
    ///     "~tagName3": <str>, // string tag (will be stored un-encrypted)
    ///     "~tagName4": <str>, // string tag (will be stored un-encrypted)
    ///   }
    ///   If tag name starts with "~" the tag will be stored un-encrypted that will allow
    ///   usage of this tag in complex search queries (comparison, predicates)
    ///   Encrypted tags can be searched only for exact matching
    pub async fn update_record_tags(
        &self,
        wallet_handle: WalletHandle,
        type_: String,
        id: String,
        tags: Tags,
    ) -> IndyResult<()> {
        trace!(
            "update_record_tags > wallet_handle {:?} type_ {:?} \
                id {:?} tags {:?}",
            wallet_handle,
            type_,
            id,
            tags
        );

        self._check_type(&type_)?;

        self.wallet_service
            .update_record_tags(wallet_handle, &type_, &id, &tags)
            .await?;

        let res = Ok(());
        trace!("update_record_tags < {:?}", res);
        res
    }

    /// Add new tags to the wallet record
    ///
    /// #Params

    /// wallet_handle: wallet handle (created by open_wallet)
    /// type_: allows to separate different record types collections
    /// id: the id of record
    /// tags_json: the record tags used for search and storing meta information as json:
    ///   {
    ///     "tagName1": <str>, // string tag (will be stored encrypted)
    ///     "tagName2": <str>, // string tag (will be stored encrypted)
    ///     "~tagName3": <str>, // string tag (will be stored un-encrypted)
    ///     "~tagName4": <str>, // string tag (will be stored un-encrypted)
    ///   }
    ///   If tag name starts with "~" the tag will be stored un-encrypted that will allow
    ///   usage of this tag in complex search queries (comparison, predicates)
    ///   Encrypted tags can be searched only for exact matching
    ///   Note if some from provided tags already assigned to the record than
    ///     corresponding tags values will be replaced
    pub async fn add_record_tags(
        &self,
        wallet_handle: WalletHandle,
        type_: String,
        id: String,
        tags: Tags,
    ) -> IndyResult<()> {
        trace!(
            "add_record_tags > wallet_handle {:?} type_ {:?} \
                id {:?} tags {:?}",
            wallet_handle,
            type_,
            id,
            tags
        );

        self._check_type(&type_)?;

        self.wallet_service
            .add_record_tags(wallet_handle, &type_, &id, &tags)
            .await?;

        let res = Ok(());
        trace!("add_record_tags < {:?}", tags);
        res
    }

    /// Delete tags from the wallet record
    ///
    /// #Params

    /// wallet_handle: wallet handle (created by open_wallet)
    /// type_: allows to separate different record types collections
    /// id: the id of record
    /// tag_names_json: the list of tag names to remove from the record as json array:
    ///   ["tagName1", "tagName2", ...]
    pub async fn delete_record_tags(
        &self,
        wallet_handle: WalletHandle,
        type_: String,
        id: String,
        tag_names_json: String,
    ) -> IndyResult<()> {
        trace!(
            "delete_record_tags > wallet_handle {:?} type_ {:?} \
                id {:?} tag_names_json {:?}",
            wallet_handle,
            type_,
            id,
            tag_names_json
        );

        self._check_type(&type_)?;

        let tag_names: Vec<&str> = serde_json::from_str(&tag_names_json).to_indy(
            IndyErrorKind::InvalidStructure,
            "Cannot deserialize tag names",
        )?;

        self.wallet_service
            .delete_record_tags(wallet_handle, &type_, &id, &tag_names)
            .await?;

        let res = Ok(());
        trace!("delete_record_tags < {:?}", res);
        res
    }

    /// Delete an existing wallet record in the wallet
    ///
    /// #Params

    /// wallet_handle: wallet handle (created by open_wallet)
    /// type_: record type
    /// id: the id of record
    pub async fn delete_record(
        &self,
        wallet_handle: WalletHandle,
        type_: String,
        id: String,
    ) -> IndyResult<()> {
        trace!(
            "delete_record > wallet_handle {:?} type_ {:?} id {:?}",
            wallet_handle,
            type_,
            id
        );

        self._check_type(&type_)?;

        self.wallet_service
            .delete_record(wallet_handle, &type_, &id)
            .await?;

        let res = Ok(());
        trace!("delete_record < {:?}", res);
        res
    }

    /// Get an wallet record by id
    ///
    /// #Params

    /// wallet_handle: wallet handle (created by open_wallet)
    /// type_: allows to separate different record types collections
    /// id: the id of record
    /// options_json: //TODO: FIXME: Think about replacing by bitmask
    ///  {
    ///    retrieveType: (optional, false by default) Retrieve record type,
    ///    retrieveValue: (optional, true by default) Retrieve record value,
    ///    retrieveTags: (optional, false by default) Retrieve record tags
    ///  }
    /// #Returns
    /// wallet record json:
    /// {
    ///   id: "Some id",
    ///   type: "Some type", // present only if retrieveType set to true
    ///   value: "Some value", // present only if retrieveValue set to true
    ///   tags: <tags json>, // present only if retrieveTags set to true
    /// }
    pub async fn get_record(
        &self,
        wallet_handle: WalletHandle,
        type_: String,
        id: String,
        options_json: String,
    ) -> IndyResult<String> {
        trace!(
            "get_record > wallet_handle {:?} type_ {:?} \
                id {:?} options_json {:?}",
            wallet_handle,
            type_,
            id,
            options_json
        );

        // self._check_type(&type_)?;

        serde_json::from_str::<RecordOptions>(&options_json).to_indy(
            IndyErrorKind::InvalidStructure,
            "Cannot deserialize options",
        )?;

        let record = self
            .wallet_service
            .get_record(wallet_handle, &type_, &id, &options_json)
            .await?;

        let record = serde_json::to_string(&record).to_indy(
            IndyErrorKind::InvalidStructure,
            "Cannot serialize WalletRecord",
        )?;

        let res = Ok(record);
        trace!("get_record < {:?}", res);
        res
    }

    /// Search for wallet records.
    ///
    /// Note instead of immediately returning of fetched records
    /// this call returns wallet_search_handle that can be used later
    /// to fetch records by small batches (with indy_fetch_wallet_search_next_records).
    ///
    /// #Params
    /// wallet_handle: wallet handle (created by open_wallet)
    /// type_: allows to separate different record types collections
    /// query_json: MongoDB style query to wallet record tags:
    ///  {
    ///    "tagName": "tagValue",
    ///    $or: {
    ///      "tagName2": { $regex: 'pattern' },
    ///      "tagName3": { $gte: '123' },
    ///    },
    ///  }
    /// options_json: //TODO: FIXME: Think about replacing by bitmask
    ///  {
    ///    retrieveRecords: (optional, true by default) If false only "counts" will be calculated,
    ///    retrieveTotalCount: (optional, false by default) Calculate total count,
    ///    retrieveType: (optional, false by default) Retrieve record type,
    ///    retrieveValue: (optional, true by default) Retrieve record value,
    ///    retrieveTags: (optional, false by default) Retrieve record tags,
    ///  }
    /// #Returns
    /// search_handle: Wallet search handle that can be used later
    ///   to fetch records by small batches (with indy_fetch_wallet_search_next_records)
    pub async fn open_search(
        &self,
        wallet_handle: WalletHandle,
        type_: String,
        query_json: String,
        options_json: String,
    ) -> IndyResult<SearchHandle> {
        trace!(
            "open_search > wallet_handle {:?} type_ {:?} \
                query_json {:?} options_json {:?}",
            wallet_handle,
            type_,
            query_json,
            options_json
        );

        self._check_type(&type_)?;

        serde_json::from_str::<SearchOptions>(&options_json).to_indy(
            IndyErrorKind::InvalidStructure,
            "Cannot deserialize options",
        )?;

        let search = self
            .wallet_service
            .search_records(wallet_handle, &type_, &query_json, &options_json)
            .await?;

        let search_handle = next_search_handle();

        self.searches
            .lock()
            .await
            .insert(search_handle, Arc::new(Mutex::new(search)));

        let res = Ok(search_handle);
        trace!("open_search < {:?}", search_handle);
        res
    }

    /// Fetch next records for wallet search.
    ///
    /// Not if there are no records this call returns WalletNoRecords error.
    ///
    /// #Params
    /// wallet_handle: wallet handle (created by open_wallet)
    /// wallet_search_handle: wallet search handle (created by indy_open_wallet_search)
    /// count: Count of records to fetch
    ///
    /// #Returns
    /// wallet records json:
    /// {
    ///   totalCount: <str>, // present only if retrieveTotalCount set to true
    ///   records: [{ // present only if retrieveRecords set to true
    ///       id: "Some id",
    ///       type: "Some type", // present only if retrieveType set to true
    ///       value: "Some value", // present only if retrieveValue set to true
    ///       tags: <tags json>, // present only if retrieveTags set to true
    ///   }],
    /// }
    pub async fn fetch_search_next_records(
        &self,
        wallet_handle: WalletHandle,
        wallet_search_handle: SearchHandle,
        count: usize,
    ) -> IndyResult<String> {
        trace!(
            "fetch_search_next_records > wallet_handle {:?} wallet_search_handle {:?} count {:?}",
            wallet_handle,
            wallet_search_handle,
            count
        );

        let search_mut = {
            self.searches
                .lock()
                .await
                .get(&wallet_search_handle)
                .ok_or_else(|| {
                    err_msg(IndyErrorKind::InvalidWalletHandle, "Unknown search handle")
                })?
                .clone()
        };

        let mut search = search_mut.lock().await;

        let mut records: Vec<WalletRecord> = Vec::new();

        for _ in 0..count {
            match search.fetch_next_record().await? {
                Some(record) => records.push(record),
                None => break,
            }
        }

        let search_result = SearchRecords {
            total_count: search.get_total_count()?,
            records: if records.is_empty() {
                None
            } else {
                Some(records)
            },
        };

        let search_result = serde_json::to_string(&search_result).to_indy(
            IndyErrorKind::InvalidState,
            "Cannot serialize SearchRecords",
        )?;

        let res = Ok(search_result);
        trace!("fetch_search_next_records < {:?}", res);
        res
    }

    /// Close wallet search (make search handle invalid)
    ///
    /// #Params
    /// wallet_search_handle: wallet search handle
    pub async fn close_search(&self, wallet_search_handle: SearchHandle) -> IndyResult<()> {
        trace!(
            "close_search > wallet_search_handle {:?}",
            wallet_search_handle
        );

        self.searches
            .lock()
            .await
            .remove(&wallet_search_handle)
            .ok_or_else(|| err_msg(IndyErrorKind::InvalidWalletHandle, "Unknown search handle"))?;

        let res = Ok(());
        trace!("close_search < {:?}", res);
        res
    }

    fn _check_type(&self, type_: &str) -> IndyResult<()> {
        if type_.starts_with(WalletService::PREFIX) {
            Err(err_msg(
                IndyErrorKind::WalletAccessFailed,
                format!("Record of type \"{}\" is not available for fetching", type_),
            ))?;
        }

        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchRecords {
    pub total_count: Option<usize>,
    pub records: Option<Vec<WalletRecord>>,
}
