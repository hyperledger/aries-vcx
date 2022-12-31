#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct InitialState {}

impl InitialState {
    pub fn new() -> Self {
        Self {}
    }
}
