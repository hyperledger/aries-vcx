fn main() {
    #[cfg(feature = "main_vdrtools_wallet")]
    uniffi::generate_scaffolding("./src/main_vcx_indy.udl").unwrap();

    #[cfg(feature = "main_askar_wallet")]
    uniffi::generate_scaffolding("./src/main_vcx_askar.udl").unwrap();
}
