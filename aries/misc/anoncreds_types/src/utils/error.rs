use std::error::Error as StdError;
use thiserror::Error;

type DynError = Box<dyn StdError + Send + Sync + 'static>;

macro_rules! define_error {
    ($name:tt, $short:expr, $doc:tt) => {
        #[derive(Debug, Error)]
        #[doc=$doc]
        pub struct $name {
            pub context: Option<String>,
            pub source: Option<DynError>,
        }

        impl $name {
            pub fn from_msg<T: Into<String>>(msg: T) -> Self {
                Self::from(msg.into())
            }

            pub fn from_err<E>(err: E) -> Self
            where
                E: StdError + Send + Sync + 'static,
            {
                Self {
                    context: None,
                    source: Some(Box::new(err) as DynError),
                }
            }

            pub fn from_msg_err<M, E>(msg: M, err: E) -> Self
            where
                M: Into<String>,
                E: StdError + Send + Sync + 'static,
            {
                Self {
                    context: Some(msg.into()),
                    source: Some(Box::new(err) as DynError),
                }
            }
        }

        impl From<&str> for $name {
            fn from(context: &str) -> Self {
                Self {
                    context: Some(context.to_owned()),
                    source: None,
                }
            }
        }

        impl From<String> for $name {
            fn from(context: String) -> Self {
                Self {
                    context: Some(context),
                    source: None,
                }
            }
        }

        impl From<Option<String>> for $name {
            fn from(context: Option<String>) -> Self {
                Self {
                    context,
                    source: None,
                }
            }
        }

        impl<M, E> From<(M, E)> for $name
        where
            M: Into<String>,
            E: StdError + Send + Sync + 'static,
        {
            fn from((context, err): (M, E)) -> Self {
                Self::from_msg_err(context, err)
            }
        }

        impl From<$name> for String {
            fn from(s: $name) -> Self {
                s.to_string()
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, $short)?;
                match self.context {
                    Some(ref context) => write!(f, ": {}", context),
                    None => Ok(()),
                }
            }
        }
    };
}

define_error!(
    ConversionError,
    "Conversion error",
    "Error type for general data conversion errors"
);

define_error!(
    EncryptionError,
    "Encryption error",
    "Error type for failure of encryption and decryption operations"
);

define_error!(
    UnexpectedError,
    "Unexpected error",
    "Error type for eventualities that shouldn't normally occur"
);

define_error!(
    ValidationError,
    "Validation error",
    "Error type for failures of `Validatable::validate`"
);

impl From<serde_json::error::Error> for ConversionError {
    fn from(err: serde_json::error::Error) -> Self {
        Self::from_msg(err.to_string())
    }
}

impl From<std::str::Utf8Error> for ConversionError {
    fn from(_err: std::str::Utf8Error) -> Self {
        Self::from("UTF-8 decoding error")
    }
}

impl From<std::string::FromUtf8Error> for ConversionError {
    fn from(_err: std::string::FromUtf8Error) -> Self {
        Self::from("UTF-8 decoding error")
    }
}

impl From<ValidationError> for ConversionError {
    fn from(err: ValidationError) -> Self {
        Self {
            context: err.context,
            source: err.source,
        }
    }
}

impl From<ConversionError> for ValidationError {
    fn from(err: ConversionError) -> Self {
        Self {
            context: err.context,
            source: err.source,
        }
    }
}

impl From<UnexpectedError> for ConversionError {
    fn from(err: UnexpectedError) -> Self {
        Self {
            context: err.context,
            source: err.source,
        }
    }
}

impl From<UnexpectedError> for EncryptionError {
    fn from(err: UnexpectedError) -> Self {
        Self {
            context: err.context,
            source: err.source,
        }
    }
}
