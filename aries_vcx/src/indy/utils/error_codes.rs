extern crate num_traits;

use crate::utils::error;

use self::num_traits::int::PrimInt;

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
