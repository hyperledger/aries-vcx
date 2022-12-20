// extern crate num_traits;

use crate::api_lib::errors::error;
use crate::api_lib::errors::error::ErrorKindLibvcx;
use num_traits::PrimInt;

pub fn map_indy_error<T, C: PrimInt>(rtn: T, error_code: C) -> Result<T, u32> {
    if error_code == C::zero() {
        return Ok(rtn);
    }

    Err(map_indy_error_code(error_code))
}

pub fn map_indy_error_code<C: PrimInt>(error_code: C) -> u32 {
    let error_code = match error_code.to_u32() {
        Some(n) => n,
        None => return ErrorKindLibvcx::UnknownError.into()
    };

    if error_code >= ErrorKindLibvcx::UnknownError.into() {
        return error_code;
    }

    match error_code {
        code @ 100..=111 => code,
        code @ 113 => code,
        200 => ErrorKindLibvcx::InvalidWalletHandle.into(),
        203 => ErrorKindLibvcx::DuplicationWallet.into(),
        206 => ErrorKindLibvcx::WalletAlreadyOpen.into(),
        212 => ErrorKindLibvcx::WalletRecordNotFound.into(),
        213 => ErrorKindLibvcx::DuplicationWalletRecord.into(),
        306 => ErrorKindLibvcx::CreatePoolConfig.into(),
        404 => ErrorKindLibvcx::DuplicationMasterSecret.into(),
        407 => ErrorKindLibvcx::CredDefAlreadyCreated.into(),
        600 => ErrorKindLibvcx::DuplicationDid.into(),
        code => code,
    }
}
