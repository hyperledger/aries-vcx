fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(false)
        .compile_protos(
            &[
                "proto/cheqd/did/v2/query.proto",
                "proto/cheqd/resource/v2/query.proto",
            ],
            &["proto"],
        )?;
    Ok(())
}
