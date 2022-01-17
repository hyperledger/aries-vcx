#[cfg(target_os = "ios")]
#[no_mangle]
pub extern "C" fn GFp_memcmp(a: *const u8, b: *const u8, len: usize) -> i32 {
    let result = unsafe { OPENSSL_memcmp(a, b, len) };
    return result
}

#[cfg(target_os = "ios")]
extern "C" {
    fn OPENSSL_memcmp(a: *const u8, b: *const u8, len: usize) -> i32;
}
