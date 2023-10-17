use ursa::cl::MasterSecret as CryptoMasterSecret;

#[derive(Debug, Deserialize, Serialize)]
pub struct MasterSecret {
    pub value: CryptoMasterSecret,
}
