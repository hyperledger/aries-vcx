use std::fmt;

#[derive(Debug)]
pub enum SearchFilter {
    #[cfg(feature = "vdrtools_wallet")]
    JsonFilter(String),
    #[cfg(feature = "askar_wallet")]
    TagFilter(aries_askar::entry::TagFilter),
}

impl fmt::Display for SearchFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            #[cfg(feature = "vdrtools_wallet")]
            SearchFilter::JsonFilter(_) => write!(f, "JsonFilter(_)"),
            #[cfg(feature = "askar_wallet")]
            SearchFilter::TagFilter(_) => write!(f, "TagFilter(_)"),
        }
    }
}
