pub mod error;
#[cfg(feature = "askar_wallet")]
mod mapping_askar;
#[cfg(feature = "vdrtools_wallet")]
mod mapping_indy_error;
mod mapping_others;
