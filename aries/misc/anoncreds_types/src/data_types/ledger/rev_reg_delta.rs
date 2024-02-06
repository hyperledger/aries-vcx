use anoncreds_clsignatures::Accumulator;

#[derive(Clone, Deserialize, Debug, Serialize, PartialEq)]
pub struct RevocationRegistryDelta {
    pub value: RevocationRegistryDeltaValue,
}

#[derive(Clone, Deserialize, Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RevocationRegistryDeltaValue {
    pub prev_accum: Option<Accumulator>,
    pub accum: Accumulator,
    #[serde(default)]
    pub issued: Vec<u32>,
    #[serde(default)]
    pub revoked: Vec<u32>,
}
