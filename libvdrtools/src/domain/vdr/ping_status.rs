use indy_api_types::errors::*;

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct PingStatus {
    pub code: PingStatusCodes,
    pub message: String,
}

impl PingStatus {
    pub fn success(message: String) -> PingStatus {
        PingStatus {
            code: PingStatusCodes::SUCCESS,
            message,
        }
    }

    pub fn fail(error: IndyError) -> PingStatus {
        PingStatus {
            code: PingStatusCodes::FAIL,
            message: format!("{:?}", error),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub enum PingStatusCodes {
    SUCCESS,
    FAIL,
}
