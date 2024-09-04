use std::io::Result;
use std::path::PathBuf;

fn main() -> Result<()> {
    let proto_file = "proto/rapidmq.proto";
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    tonic_build::configure()
        .build_server(true)
        .out_dir(&out_dir)
        .compile(&[proto_file], &["proto"])?;

    println!("cargo:rerun-if-changed={}", proto_file);

    Ok(())
}