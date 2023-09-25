// extern crate num_traits;

use num_traits::PrimInt;

use crate::errors::error::LibvcxErrorKind;

pub fn map_indy_error<T, C: PrimInt>(rtn: T, error_code: C) -> Result<T, u32> {
    if error_code == C::zero() {
        return Ok(rtn);
    }

    Err(map_indy_error_code(error_code))
}

pub fn map_indy_error_code<C: PrimInt>(error_code: C) -> u32 {
    let error_code = match error_code.to_u32() {
        Some(n) => n,
        None => return LibvcxErrorKind::UnknownError.into(),
    };

    if error_code >= LibvcxErrorKind::UnknownError.into() {
        return error_code;
    }

    match error_code {
        code @ 100..=111 => code,
        code @ 113 => code,
        200 => LibvcxErrorKind::InvalidWalletHandle.into(),
        203 => LibvcxErrorKind::DuplicationWallet.into(),
        206 => LibvcxErrorKind::WalletAlreadyOpen.into(),
        212 => LibvcxErrorKind::WalletRecordNotFound.into(),
        213 => LibvcxErrorKind::DuplicationWalletRecord.into(),
        306 => LibvcxErrorKind::CreatePoolConfig.into(),
        404 => LibvcxErrorKind::DuplicationMasterSecret.into(),
        407 => LibvcxErrorKind::CredDefAlreadyCreated.into(),
        600 => LibvcxErrorKind::DuplicationDid.into(),
        code => code,
    }
}
