#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PublicKeyEncoding {
    Multibase,
    Base58,
}

pub struct ExtraFieldsOptions {
    public_key_encoding: PublicKeyEncoding,
}

impl Default for ExtraFieldsOptions {
    fn default() -> Self {
        Self {
            public_key_encoding: PublicKeyEncoding::Base58,
        }
    }
}

impl ExtraFieldsOptions {
    pub fn new() -> Self {
        Self { ..Default::default() }
    }

    pub fn set_public_key_encoding(mut self, public_key_encoding: PublicKeyEncoding) -> Self {
        self.public_key_encoding = public_key_encoding;
        self
    }

    pub fn public_key_encoding(&self) -> PublicKeyEncoding {
        self.public_key_encoding
    }
}
