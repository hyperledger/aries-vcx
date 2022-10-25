#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct FinishedState {}

impl FinishedState {
    pub fn new() -> Self {
        Self {}
    }
}
