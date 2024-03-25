pub mod error;
#[cfg(feature = "anoncreds")]
mod mapping_anoncreds;
#[cfg(feature = "credx")]
mod mapping_credx;
mod mapping_indyvdr;
#[cfg(feature = "vdr_proxy_ledger")]
mod mapping_indyvdr_proxy;
mod mapping_ledger_response_parser;
mod mapping_others;
mod mapping_wallet;
