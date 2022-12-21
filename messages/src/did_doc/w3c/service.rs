use url::Url;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DidDocService {
    pub id: String,
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(rename = "serviceEndpoint")]
    pub service_endpoint: String,
}
