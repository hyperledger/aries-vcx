#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct MsgCreateSchema {
    pub name: String,
    pub version: String,
    pub attr_names: Vec<String>
}
