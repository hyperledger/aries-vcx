extern crate num_traits;

use vdrtoolsrs::IndyError;

use crate::error::prelude::{MessagesError, MesssagesErrorKind};
use crate::utils::error;

use self::num_traits::int::PrimInt;

impl From<IndyError> for MessagesError {
    fn from(error: IndyError) -> Self {
        match error.error_code as u32 {
            100..=111 => MessagesError::from_msg(MesssagesErrorKind::InvalidLibindyParam, error.message),
            113 => MessagesError::from_msg(MesssagesErrorKind::LibindyInvalidStructure, error.message),
            114 => MessagesError::from_msg(MesssagesErrorKind::IOError, error.message),
            200 => MessagesError::from_msg(MesssagesErrorKind::InvalidWalletHandle, error.message),
            203 => MessagesError::from_msg(MesssagesErrorKind::DuplicationWallet, error.message),
            204 => MessagesError::from_msg(MesssagesErrorKind::WalletNotFound, error.message),
            206 => MessagesError::from_msg(MesssagesErrorKind::WalletAlreadyOpen, error.message),
            212 => MessagesError::from_msg(MesssagesErrorKind::WalletRecordNotFound, error.message),
            213 => MessagesError::from_msg(MesssagesErrorKind::DuplicationWalletRecord, error.message),
            306 => MessagesError::from_msg(MesssagesErrorKind::CreatePoolConfig, error.message),
            404 => MessagesError::from_msg(MesssagesErrorKind::DuplicationMasterSecret, error.message),
            407 => MessagesError::from_msg(MesssagesErrorKind::CredDefAlreadyCreated, error.message),
            600 => MessagesError::from_msg(MesssagesErrorKind::DuplicationDid, error.message),
            702 => MessagesError::from_msg(MesssagesErrorKind::InsufficientTokenAmount, error.message),
            error_code => MessagesError::from_msg(MesssagesErrorKind::LibndyError(error_code), error.message),
        }
    }
}

pub fn map_indy_error<T, C: PrimInt>(rtn: T, error_code: C) -> Result<T, u32> {
    if error_code == C::zero() {
        return Ok(rtn);
    }

    Err(map_indy_error_code(error_code))
}

pub fn map_indy_error_code<C: PrimInt>(error_code: C) -> u32 {
    let error_code = match error_code.to_u32() {
        Some(n) => {
            error!("MAPPING ERROR: {:?} -- {}", n, error::error_message(&n));
            n
        }
        None => return error::UNKNOWN_LIBINDY_ERROR.code_num,
    };

    if error_code >= error::UNKNOWN_ERROR.code_num {
        return error_code;
    }

    match error_code {
        100..=111 => error::INVALID_LIBINDY_PARAM.code_num,
        113 => error::LIBINDY_INVALID_STRUCTURE.code_num,
        200 => error::INVALID_WALLET_HANDLE.code_num,
        203 => error::WALLET_ALREADY_EXISTS.code_num,
        206 => error::WALLET_ALREADY_OPEN.code_num,
        212 => error::WALLET_RECORD_NOT_FOUND.code_num,
        213 => error::DUPLICATE_WALLET_RECORD.code_num,
        306 => error::CREATE_POOL_CONFIG.code_num,
        404 => error::DUPLICATE_MASTER_SECRET.code_num,
        407 => error::CREDENTIAL_DEF_ALREADY_CREATED.code_num,
        600 => error::DID_ALREADY_EXISTS_IN_WALLET.code_num,
        702 => error::INSUFFICIENT_TOKEN_AMOUNT.code_num,
        error_cde => error_cde,
    }
}

#[cfg(test)]
#[cfg(feature = "general_test")]
pub mod unit_tests {
    use vdrtoolsrs::ErrorCode;

    use crate::utils::devsetup::SetupEmpty;

    use super::*;

    #[test]
    fn test_invalid_param_err() {
        let _setup = SetupEmpty::init();

        let err100: IndyError = IndyError {
            error_code: ErrorCode::CommonInvalidParam1,
            message: String::new(),
            indy_backtrace: None,
        };
        let err107: IndyError = IndyError {
            error_code: ErrorCode::CommonInvalidParam8,
            message: String::new(),
            indy_backtrace: None,
        };
        let err111: IndyError = IndyError {
            error_code: ErrorCode::CommonInvalidParam12,
            message: String::new(),
            indy_backtrace: None,
        };
        let err112: IndyError = IndyError {
            error_code: ErrorCode::CommonInvalidState,
            message: String::new(),
            indy_backtrace: None,
        };

        assert_eq!(
            MessagesError::from(err100).kind(),
            MesssagesErrorKind::InvalidLibindyParam
        );
        assert_eq!(
            MessagesError::from(err107).kind(),
            MesssagesErrorKind::InvalidLibindyParam
        );
        assert_eq!(
            MessagesError::from(err111).kind(),
            MesssagesErrorKind::InvalidLibindyParam
        );
        // Test that RC 112 falls out of the range 100...112
        assert_ne!(
            MessagesError::from(err112).kind(),
            MesssagesErrorKind::InvalidLibindyParam
        );
    }
}
