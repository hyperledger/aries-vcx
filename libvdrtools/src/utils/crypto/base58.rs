use bs58::{encode, decode, decode::Result};

pub trait ToBase58 {
    fn to_base58(&self) -> String;
}
pub trait FromBase58 {
    fn from_base58(&self) -> Result<Vec<u8>>;
}

impl ToBase58 for [u8] {
    fn to_base58(&self) -> String {
        encode(self).into_string()
    }
}

impl FromBase58 for [u8] {
    fn from_base58(&self) -> Result<Vec<u8>> {
        decode(self).into_vec()
    }
}

impl FromBase58 for str {
    fn from_base58(&self) -> Result<Vec<u8>> {
        decode(self.as_bytes()).into_vec()
    }
}
