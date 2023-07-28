pub mod error;
mod mapping_agency_client;
#[cfg(any(feature = "modular_libs", feature = "vdr_proxy_ledger"))]
mod mapping_credx;
#[cfg(any(feature = "modular_libs", feature = "vdr_proxy_ledger"))]
mod mapping_indyvdr;
#[cfg(feature = "vdr_proxy_ledger")]
mod mapping_indyvdr_proxy;
mod mapping_others;
mod mapping_vdrtools;
