macro_rules! check_useful_c_callback {
    ($x:ident, $e:expr) => {
        let $x = match $x {
            Some($x) => $x,
            None => {
                let err = VcxError::from_msg($e, "Invalid callback has been passed");
                set_current_error_vcx(&err);
                return err.into();
            }
        };
    };
}
