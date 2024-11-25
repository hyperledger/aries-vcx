//! Binary for re-generating the cheqd proto types
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let crate_dir = env!("CARGO_MANIFEST_DIR").to_string();

    tonic_build::configure()
        .build_server(false)
        .out_dir(crate_dir.clone() + "/../src/proto")
        .compile_protos(
            &[
                crate_dir.clone() + "/proto/cheqd/did/v2/query.proto",
                crate_dir.clone() + "/proto/cheqd/resource/v2/query.proto",
            ],
            &[crate_dir + "/proto"],
        )?;
    Ok(())
}
