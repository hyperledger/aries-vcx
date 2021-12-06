use indyrs::{cheqd_keys, future::Future, IndyError, WalletHandle};

pub fn add_random(wallet_handle: WalletHandle, alias: &str) -> Result<String, IndyError> {
    cheqd_keys::add_random(wallet_handle, alias).wait()
}

pub fn add_from_mnemonic(wallet_handle: WalletHandle, alias: &str, mnemonic: &str, passphrase: &str) -> Result<String, IndyError> {
    cheqd_keys::add_from_mnemonic(wallet_handle, alias, mnemonic, passphrase).wait()
}

pub fn get_info(wallet_handle: WalletHandle, alias: &str) -> Result<String, IndyError> {
    cheqd_keys::get_info(wallet_handle, alias).wait()
}

pub fn get_list_keys(wallet_handle: WalletHandle) -> Result<String, IndyError> {
    cheqd_keys::list(wallet_handle).wait()
}

pub fn sign(wallet_handle: WalletHandle, alias: &str, tx: &[u8]) -> Result<Vec<u8>, IndyError> {
    cheqd_keys::sign(wallet_handle, alias, tx).wait()
}
