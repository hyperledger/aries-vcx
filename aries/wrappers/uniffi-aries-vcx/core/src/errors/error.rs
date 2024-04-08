pub type VcxUniFFIResult<T> = Result<T, VcxUniFFIError>;

// I've been super lazy here and only defined two types. But there
// can/should be effectively 1-to-1 mapping with Aries_VCX errors
#[derive(Debug, thiserror::Error)]
pub enum VcxUniFFIError {
    #[error("An AriesVCX error occured. More Info: {}", error_msg)]
    AriesVcxError { error_msg: String },
    #[error("An AriesVCXWallet error occured. More Info: {}", error_msg)]
    AriesVcxWalletError { error_msg: String },
    #[error("An AriesVCXLedger error occured. More Info: {}", error_msg)]
    AriesVcxLedgerError { error_msg: String },
    #[error("An AriesVCXAnoncreds error occured. More Info: {}", error_msg)]
    AriesVcxAnoncredsError { error_msg: String },
    #[error(
        "A serialization error occurred. Check your inputs. More Info: {}",
        error_msg
    )]
    SerializationError { error_msg: String },
    #[error("A string could not be parsed. More Info: {}", error_msg)]
    StringParseError { error_msg: String },
    #[error("An unexpected internal error occured. More Info: {}", error_msg)]
    InternalError { error_msg: String },
}
