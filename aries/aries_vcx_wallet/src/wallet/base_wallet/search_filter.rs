use std::fmt;

use crate::errors::error::VcxWalletResult;

#[derive(Debug)]
pub enum SearchFilter {
    #[cfg(feature = "vdrtools_wallet")]
    JsonFilter(String),
    #[cfg(feature = "askar_wallet")]
    TagFilter(aries_askar::entry::TagFilter),
}

#[cfg(feature = "askar_wallet")]
fn tag_filter_from_string(wql: &str) -> VcxWalletResult<SearchFilter> {
    use std::str::FromStr;

    use aries_askar::entry::TagFilter;

    use crate::errors::error::VcxWalletError;

    let filter =
        TagFilter::from_str(wql).map_err(|err| VcxWalletError::InvalidWql(err.to_string()))?;

    Ok(SearchFilter::TagFilter(filter))
}

#[cfg(feature = "vdrtools_wallet")]
fn json_filter_from_string(wql: &str) -> VcxWalletResult<SearchFilter> {
    Ok(SearchFilter::JsonFilter(wql.into()))
}

impl SearchFilter {
    #[allow(unused_variables)]
    pub fn from_string(wql: &str) -> VcxWalletResult<Self> {
        #[cfg(feature = "vdrtools_wallet")]
        let filter = json_filter_from_string(wql);

        #[cfg(feature = "askar_wallet")]
        let filter = tag_filter_from_string(wql);

        filter
    }
}

impl fmt::Display for SearchFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            #[cfg(feature = "vdrtools_wallet")]
            SearchFilter::JsonFilter(_) => write!(f, "JsonFilter(_)"),
            #[cfg(feature = "askar_wallet")]
            SearchFilter::TagFilter(_) => write!(f, "TagFilter(_)"),
            #[cfg(all(not(feature = "askar_wallet"), not(feature = "vdrtools_wallet")))]
            _ => {
                let _ = f;
                unreachable!()
            }
        }
    }
}
