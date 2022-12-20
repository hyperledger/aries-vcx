extern crate num_traits;

use vcx::api_lib::utils::libvcx_error;

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
            error!("MAPPING ERROR: {:?} -- {}", n, libvcx_error::error_message(&n));
            n
        }
        None => return libvcx_error::UNKNOWN_LIBINDY_ERROR.code_num,
    };

    if error_code >= libvcx_error::UNKNOWN_ERROR.code_num {
        return error_code;
    }

    match error_code {
        100..=111 => libvcx_error::INVALID_LIBINDY_PARAM.code_num,
        113 => libvcx_error::LIBINDY_INVALID_STRUCTURE.code_num,
        200 => libvcx_error::INVALID_WALLET_HANDLE.code_num,
        203 => libvcx_error::WALLET_ALREADY_EXISTS.code_num,
        206 => libvcx_error::WALLET_ALREADY_OPEN.code_num,
        212 => libvcx_error::WALLET_RECORD_NOT_FOUND.code_num,
        213 => libvcx_error::DUPLICATE_WALLET_RECORD.code_num,
        306 => libvcx_error::CREATE_POOL_CONFIG.code_num,
        404 => libvcx_error::DUPLICATE_MASTER_SECRET.code_num,
        407 => libvcx_error::CREDENTIAL_DEF_ALREADY_CREATED.code_num,
        600 => libvcx_error::DID_ALREADY_EXISTS_IN_WALLET.code_num,
        702 => libvcx_error::INSUFFICIENT_TOKEN_AMOUNT.code_num,
        error_cde => error_cde,
    }
}
