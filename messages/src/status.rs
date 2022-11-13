use crate::problem_report::ProblemReport;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Status {
    Undefined,
    Success,
    Failed(ProblemReport),
    Declined(ProblemReport),
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
            Status::Declined(err) => {
                error!("Declined: {:?}", err);
                3
            }
        }
    }

    pub fn from_u32(state: u32) -> Self {
        match state {
            1 => Self::Success,
            2 => Self::Failed(ProblemReport::create()),
            3 => Self::Declined(ProblemReport::create()),
            _ => Self::Undefined,
        }
    }
}
