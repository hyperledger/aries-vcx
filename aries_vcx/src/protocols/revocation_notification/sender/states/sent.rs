#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct NotificationSentState {}

impl NotificationSentState {
    pub fn new() -> Self {
        Self {}
    }
}
