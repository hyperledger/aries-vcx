macro_rules! prepare_result {
    ($result:ident) => {{
        trace!("prepare_result: >>> {:?}", $result);
        match $result {
            Ok(_) => ErrorCode::Success,
            Err(err) => {
                if err.kind() == indy_api_types::errors::IndyErrorKind::InvalidState {
                    error!("InvalidState: {}", err);
                }
                err.into()
            }
        }
    }};
    ($result:ident, $($dflt_val:expr),*) => {{
        trace!("prepare_result: >>> {:?}", $result);
        match $result {
            Ok(res) => (ErrorCode::Success, res),
            Err(err) => {
                if err.kind() == indy_api_types::errors::IndyErrorKind::InvalidState {
                    error!("InvalidState: {}", err);
                }
                (err.into(), ($($dflt_val),*))
            }
        }
    }}
}

macro_rules! unwrap_opt_or_return {
    ($opt:expr, $err:expr) => {
        match $opt {
            Some(val) => val,
            None => return $err
        };
    }
}

macro_rules! unwrap_or_return {
    ($result:expr, $err:expr) => {
        match $result {
            Ok(res) => res,
            Err(_) => return $err
        };
    }
}
