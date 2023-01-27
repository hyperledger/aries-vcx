use messages::protocols::connection::invite::Invitation;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InitialState {
    pub(crate) invitation: Invitation,
}

impl InitialState {
    pub fn new(invitation: Invitation) -> Self {
        Self { invitation }
    }
}
