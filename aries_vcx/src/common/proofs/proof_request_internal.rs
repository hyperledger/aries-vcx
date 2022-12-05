#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Filter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema_issuer_did: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issuer_did: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cred_def_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(untagged)]
pub enum Restrictions {
    V1(Vec<Filter>),
    V2(serde_json::Value),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct PredicateInfo {
    pub name: String,
    //Todo: Update p_type to use Enum
    pub p_type: String,
    pub p_value: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restrictions: Option<Restrictions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub non_revoked: Option<NonRevokedInterval>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct AttrInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub names: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restrictions: Option<Restrictions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub non_revoked: Option<NonRevokedInterval>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub self_attest_allowed: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct NonRevokedInterval {
    pub from: Option<u64>,
    pub to: Option<u64>,
}
