pub mod error;
#[cfg(feature = "cheqd")]
mod mapping_cheqd;
mod mapping_indyvdr;
#[cfg(feature = "vdr_proxy_ledger")]
mod mapping_indyvdr_proxy;
mod mapping_ledger_response_parser;
mod mapping_others;
