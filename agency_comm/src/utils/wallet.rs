use indy::{INVALID_WALLET_HANDLE, WalletHandle};

pub static mut WALLET_HANDLE: WalletHandle = INVALID_WALLET_HANDLE;

pub fn set_wallet_handle(handle: WalletHandle) -> WalletHandle {
    unsafe { WALLET_HANDLE = handle; }
    unsafe { WALLET_HANDLE }
}

pub fn get_wallet_handle() -> WalletHandle { unsafe { WALLET_HANDLE } }
