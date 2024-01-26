use std::fmt;

use crate::cl::{bn::BigNumber, Prover as CryptoProver};
use crate::error::ConversionError;

pub struct LinkSecret(pub(crate) BigNumber);

impl LinkSecret {
    pub fn new() -> Result<Self, ConversionError> {
        let value = CryptoProver::new_link_secret()
            .map_err(|err| ConversionError::from_msg(format!("Error creating link secret: {err}")))?
            .into();

        Ok(Self(value))
    }

    pub fn try_clone(&self) -> Result<Self, ConversionError> {
        let cloned = self.0.try_clone().map_err(|err| {
            ConversionError::from_msg(format!("Error cloning link secret: {err}"))
        })?;

        Ok(Self(cloned))
    }
}

impl fmt::Debug for LinkSecret {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("LinkSecret")
            .field(if cfg!(test) { &self.0 } else { &"<hidden>" })
            .finish()
    }
}

impl TryInto<String> for LinkSecret {
    type Error = ConversionError;

    fn try_into(self) -> Result<String, Self::Error> {
        self.0.to_dec().map_err(|err| {
            ConversionError::from_msg(format!("Error converting link secret: {err}"))
        })
    }
}

impl TryFrom<&str> for LinkSecret {
    type Error = ConversionError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(Self(BigNumber::from_dec(value).map_err(|err| {
            ConversionError::from_msg(format!("Error converting link secret: {err}"))
        })?))
    }
}

#[cfg(test)]
mod link_secret_tests {
    use super::*;

    #[test]
    fn should_create_new_link_secret() {
        let link_secret = LinkSecret::new();
        assert!(link_secret.is_ok());
    }

    #[test]
    fn should_convert_between_string_and_link_secret_roundtrip() {
        let ls = "123";
        let link_secret = LinkSecret::try_from(ls).expect("Error creating link secret");
        let link_secret_str: String = link_secret.try_into().expect("Error creating link secret");
        assert_eq!(link_secret_str, ls);
    }

    #[test]
    fn should_clone_link_secret() {
        let link_secret = LinkSecret::new().expect("Unable to create link secret");
        let cloned_link_secret = link_secret
            .try_clone()
            .expect("Unable to clone link secret");

        assert_eq!(link_secret.0, cloned_link_secret.0);
    }
}
