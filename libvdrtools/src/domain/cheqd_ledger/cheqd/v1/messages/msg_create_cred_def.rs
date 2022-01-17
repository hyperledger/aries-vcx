#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct MsgCreateCredDef {
    pub schema_id: String,
    pub tag: String,
    pub signature_type: String,
    pub value: String,
}
