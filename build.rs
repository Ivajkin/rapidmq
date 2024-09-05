use std::io::Result;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Compile protos using tonic-build
    tonic_build::compile_protos("proto/your_proto_file.proto")?;

    // Compile protos using protobuf-build
    protobuf_build::Builder::new()
        .files(&["proto/your_proto_file.proto"])
        .out_dir("src/generated")
        .generate()?;

    Ok(())
}