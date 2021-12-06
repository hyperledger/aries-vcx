pub use indy_utils::environment;

#[macro_use]
pub mod ccallback;

pub mod crypto;
#[cfg(feature = "cheqd")]
pub mod cheqd_crypto;
#[macro_use]
pub mod logger;

#[allow(unused_macros)]
#[macro_use]
pub mod result;

#[cfg(test)]
pub use indy_utils::test;

#[macro_use]
pub mod try_utils;

pub use indy_api_types::validation;

pub use indy_utils::wql;

#[macro_use]
pub mod qualifier;

pub mod extensions;
#[cfg(feature = "cheqd")]
pub mod cheqd_ledger;

macro_rules! map (
    { $($key:expr => $value:expr),+ } => {
        {
            let mut m = ::std::collections::HashMap::new();
            $(
                m.insert($key, $value);
            )+
            m
        }
     };
);

macro_rules! json_string {
    ($value:ident) => ({
        serde_json::to_string(&$value)
            .map_err(|err| IndyError::from_msg(
                IndyErrorKind::InvalidStructure,
                format!("Cannot serialize Object into JSON String. Err: {:?}", err),
            ))?
    })
}

macro_rules! json_string_result {
    ($value:ident) => ({
        Ok(json_string!($value))
    })
}
