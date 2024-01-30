fn main() {
    #[cfg(feature = "vdrtools_wallet")]
    uniffi::generate_scaffolding("./src/vcx.udl").unwrap();

    #[cfg(feature = "askar_wallet")]
    uniffi::generate_scaffolding("./src/vcx_askar.udl").unwrap();
}
