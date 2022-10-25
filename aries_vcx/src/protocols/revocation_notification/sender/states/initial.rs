#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct InitialState {}

impl InitialState {
    pub fn new() -> Self {
        Self {}
    }
}
