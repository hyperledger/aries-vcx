pub enum SearchFilter {
    #[cfg(feature = "vdrtools_wallet")]
    JsonFilter(String),
    #[cfg(feature = "askar_wallet")]
    TagFilter(aries_askar::entry::TagFilter),
}
