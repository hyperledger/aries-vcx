pub mod error;
mod mapping_agency_client;
#[cfg(any(feature = "modular_libs", feature = "vdr_proxy_ledger"))]
mod mapping_credx;
#[cfg(any(feature = "modular_libs", feature = "vdr_proxy_ledger"))]
mod mapping_indyvdr;
#[cfg(feature = "vdr_proxy_ledger")]
mod mapping_indyvdr_proxy;
mod mapping_others;
#[cfg(any(feature = "vdrtools_wallet", feature = "vdrtools_ledger", feature = "vdrtools_anoncreds", feature = "vdr_proxy_ledger"))]
mod mapping_vdrtools;
