use crate::messages::error::ProblemReport;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Status {
    Undefined,
    Success,
    Failed(ProblemReport),
    Declined,
}

impl Status {
    pub fn code(&self) -> u32 {
        match self {
            Status::Undefined => 0,
            Status::Success => 1,
            Status::Failed(err) => {
                error!("Process Failed: {:?}", err);
                2
            }
            Status::Declined => 3
        }
    }

    pub fn from_u32(state: u32) -> Self {
        match state {
            1 => Self::Success,
            2 => Self::Failed(ProblemReport::create()),
            3 => Self::Declined,
            _ => Self::Undefined,
        }
    }
}
