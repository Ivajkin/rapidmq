use std::io::Result;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("proto/service1.proto")?;
    tonic_build::compile_protos("proto/service2.proto")?;
    Ok(())
}