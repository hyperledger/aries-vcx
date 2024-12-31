use actix_web::{error, http::StatusCode, HttpResponse, HttpResponseBuilder};
use aries_vcx_agent::{
    aries_vcx,
    aries_vcx::{did_parser_nom::ParseError, messages::error::MsgTypeError},
    AgentError,
};
use derive_more::{Display, Error};

pub type HarnessResult<T> = Result<T, HarnessError>;

#[derive(Debug, Display, Error, Clone)]
pub enum HarnessErrorType {
    #[display("Internal server error")]
    InternalServerError,
    #[display("Request not accepted")]
    RequestNotAcceptedError,
    #[display("Request not received")]
    RequestNotReceived,
    #[display("Not found")]
    NotFoundError,
    #[display("Invalid JSON")]
    InvalidJson,
    #[display("Protocol error")]
    ProtocolError,
    #[display("Invalid state for requested operation")]
    InvalidState,
    #[display("Encryption error")]
    EncryptionError,
    #[display("Multiple credential definitions found")]
    MultipleCredDefinitions,
}

#[derive(Debug, Display, Error, Clone)]
#[display("Error: {}", message)]
pub struct HarnessError {
    pub message: String,
    pub kind: HarnessErrorType,
}

impl error::ResponseError for HarnessError {
    fn error_response(&self) -> HttpResponse {
        error!("{}", self.to_string());
        HttpResponseBuilder::new(self.status_code()).body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match self.kind {
            HarnessErrorType::RequestNotAcceptedError
            | HarnessErrorType::RequestNotReceived
            | HarnessErrorType::InvalidJson => StatusCode::NOT_ACCEPTABLE,
            HarnessErrorType::NotFoundError => StatusCode::NOT_FOUND,
            HarnessErrorType::InvalidState => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl HarnessError {
    pub fn from_msg(kind: HarnessErrorType, msg: &str) -> Self {
        HarnessError {
            kind,
            message: msg.to_string(),
        }
    }

    pub fn from_kind(kind: HarnessErrorType) -> Self {
        let message = kind.to_string();
        HarnessError { kind, message }
    }
}

impl std::convert::From<aries_vcx::errors::error::AriesVcxError> for HarnessError {
    fn from(vcx_err: aries_vcx::errors::error::AriesVcxError) -> HarnessError {
        let kind = HarnessErrorType::InternalServerError;
        HarnessError {
            message: vcx_err.to_string(),
            kind,
        }
    }
}

impl std::convert::From<aries_vcx::aries_vcx_anoncreds::errors::error::VcxAnoncredsError>
    for HarnessError
{
    fn from(
        vcx_err: aries_vcx::aries_vcx_anoncreds::errors::error::VcxAnoncredsError,
    ) -> HarnessError {
        let kind = HarnessErrorType::InternalServerError;
        HarnessError {
            message: vcx_err.to_string(),
            kind,
        }
    }
}

impl std::convert::From<serde_json::Error> for HarnessError {
    fn from(serde_err: serde_json::Error) -> HarnessError {
        let kind = HarnessErrorType::InternalServerError;
        let message = format!("(De)serialization failed; err: {}", serde_err);
        HarnessError { message, kind }
    }
}

impl std::convert::From<std::io::Error> for HarnessError {
    fn from(io_err: std::io::Error) -> HarnessError {
        let kind = HarnessErrorType::InternalServerError;
        let message = format!("I/O error: {}", io_err);
        HarnessError { message, kind }
    }
}

impl std::convert::From<reqwest::Error> for HarnessError {
    fn from(rw_err: reqwest::Error) -> HarnessError {
        let kind = HarnessErrorType::InternalServerError;
        let message = format!("Reqwest error: {}", rw_err);
        HarnessError { message, kind }
    }
}

impl std::convert::From<AgentError> for HarnessError {
    fn from(err: AgentError) -> HarnessError {
        let kind = HarnessErrorType::InternalServerError;
        let message = format!("AgentError: {}", err);
        HarnessError { message, kind }
    }
}

impl std::convert::From<MsgTypeError> for HarnessError {
    fn from(err: MsgTypeError) -> HarnessError {
        let kind = HarnessErrorType::InternalServerError;
        let message = format!("MsgTypeError: {}", err);
        HarnessError { message, kind }
    }
}

impl std::convert::From<ParseError> for HarnessError {
    fn from(err: ParseError) -> HarnessError {
        let kind = HarnessErrorType::InternalServerError;
        let message = format!("MsgTypeError: {}", err);
        HarnessError { message, kind }
    }
}

impl std::convert::From<anoncreds_types::Error> for HarnessError {
    fn from(err: anoncreds_types::Error) -> HarnessError {
        let kind = HarnessErrorType::InternalServerError;
        let message = format!("MsgTypeError: {}", err);
        HarnessError { message, kind }
    }
}
