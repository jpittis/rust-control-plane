use glob::glob;
use std::io;
use std::path::PathBuf;

fn main() -> io::Result<()> {
    let protos: Vec<PathBuf> = glob("data-plane-api/envoy/**/v3/*.proto")
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    let mut config = prost_build::Config::new();
    config.disable_comments(["."]);
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_well_known_types(true)
        .include_file("mod.rs")
        .compile_with_config(
            config,
            &protos,
            &[
                "data-plane-api",
                "googleapis",
                "protoc-gen-validate",
                "xds",
                "opencensus-proto/src",
                "opentelemetry-proto",
                "client_model",
            ],
        )?;
    Ok(())
}
