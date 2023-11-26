#[cfg(debug_assertions)]
#[macro_export]
macro_rules! secret {
    ($val:expr) => {{
        $val
    }};
}

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! secret {
    ($val:expr) => {{
        "_"
    }};
}

pub mod base64;
pub mod openssl;
pub mod qualifier;

#[macro_use]
pub mod encryption_envelope;
pub mod serialization;
pub mod validation;
