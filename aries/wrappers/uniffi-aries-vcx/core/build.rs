fn main() {
    #[cfg(all(feature = "vdrtools_wallet", feature = "askar_wallet"))]
    compile_error!("features `vdrtools_wallet` and `askar_wallet` are mutually exclusive");

    #[cfg(feature = "vdrtools_wallet")]
    uniffi::generate_scaffolding("./src/vcx_indy.udl").unwrap();

    #[cfg(feature = "askar_wallet")]
    uniffi::generate_scaffolding("./src/vcx.udl").unwrap();
}
