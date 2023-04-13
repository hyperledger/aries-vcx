// TODO - FUTURE - visibility of all indy should be 'crate' to confirm indy dependency is stripped
pub(crate) mod anoncreds;
pub(crate) mod credentials;
pub(crate) mod keys;
pub mod ledger; // temporarily left public due to pool set up utils
pub(crate) mod primitives;
pub(crate) mod proofs;
pub(crate) mod signing;
pub mod utils;
pub mod wallet; // temporarily left public due to wallet set up utils
pub(crate) mod wallet_non_secrets;

// Vdrtools handle wrappers

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct WalletHandle(pub vdrtools::WalletHandle);
pub const INVALID_WALLET_HANDLE: WalletHandle = WalletHandle(vdrtools::INVALID_WALLET_HANDLE);

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct SearchHandle(pub vdrtools::SearchHandle);
pub const INVALID_SEARCH_HANDLE: SearchHandle = SearchHandle(vdrtools::INVALID_SEARCH_HANDLE);

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct PoolHandle(pub vdrtools::PoolHandle);
pub const INVALID_POOL_HANDLE: PoolHandle = PoolHandle(vdrtools::INVALID_POOL_HANDLE);
