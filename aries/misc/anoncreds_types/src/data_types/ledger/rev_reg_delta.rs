use anoncreds_clsignatures::Accumulator;

#[derive(Clone, Deserialize, Debug, Serialize, PartialEq)]
pub struct RevocationRegistryDelta {
    pub value: RevocationRegistryDeltaValue,
}

#[derive(Clone, Deserialize, Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RevocationRegistryDeltaValue {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev_accum: Option<Accumulator>,
    pub accum: Accumulator,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub issued: Vec<u32>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub revoked: Vec<u32>,
}
