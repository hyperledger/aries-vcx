use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum ThinState {
    RequestSent,
    ResponseSent,
    Completed,
    Abandoned,
}

impl Display for ThinState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThinState::RequestSent => write!(f, "RequestSent"),
            ThinState::ResponseSent => write!(f, "ResponseSent"),
            ThinState::Completed => write!(f, "Completed"),
            ThinState::Abandoned => write!(f, "Abandoned"),
        }
    }
}
