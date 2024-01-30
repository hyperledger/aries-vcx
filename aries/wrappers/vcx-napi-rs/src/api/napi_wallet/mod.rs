#[cfg(feature = "askar_wallet")]
pub mod askar;
#[cfg(feature = "askar_wallet")]
pub use askar as napi_wallet;

#[cfg(feature = "vdrtools_wallet")]
pub mod indy;
#[cfg(feature = "vdrtools_wallet")]
pub use indy as napi_wallet;
