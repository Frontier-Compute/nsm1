fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(false) // client only
        .compile_protos(
            &["proto/service.proto", "proto/compact_formats.proto"],
            &["proto"],
        )?;
    Ok(())
}
