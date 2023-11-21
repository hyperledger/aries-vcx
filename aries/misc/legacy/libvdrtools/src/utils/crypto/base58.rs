use bs58::{decode, decode::Result, encode};

pub trait ToBase58 {
    fn to_base58(&self) -> String;
}
pub trait DecodeBase58 {
    fn decode_base58(self) -> Result<Vec<u8>>;
}

impl ToBase58 for [u8] {
    fn to_base58(&self) -> String {
        encode(self).into_string()
    }
}

impl DecodeBase58 for &[u8] {
    fn decode_base58(self) -> Result<Vec<u8>> {
        decode(self).into_vec()
    }
}

impl DecodeBase58 for &str {
    fn decode_base58(self) -> Result<Vec<u8>> {
        decode(self.as_bytes()).into_vec()
    }
}
