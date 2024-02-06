#[derive(Clone, Deserialize, Debug, Serialize)]
pub struct RevocationRegistryDelta {
    pub value: RevocationRegistryDeltaValue,
    // #[serde(rename = "ver")]
    // version: String,
}

#[derive(Clone, Deserialize, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RevocationRegistryDeltaValue {
    prev_accum: Option<String>,
    pub accum: String,
    #[serde(default)]
    pub issued: Vec<u32>,
    #[serde(default)]
    pub revoked: Vec<u32>,
}
