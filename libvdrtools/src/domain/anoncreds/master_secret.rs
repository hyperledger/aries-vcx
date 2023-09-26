use indy_api_types::validation::Validatable;
use ursa::cl::MasterSecret as CryptoMasterSecret;

#[derive(Debug, Deserialize, Serialize)]
pub struct MasterSecret {
    pub value: CryptoMasterSecret,
}

impl Validatable for MasterSecret {}
