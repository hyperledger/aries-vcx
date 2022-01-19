#[derive(Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct DidTxnParams {
    pub did: String,
    pub verkey: String,
}