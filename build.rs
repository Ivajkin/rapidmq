use std::io::Result;
use std::path::PathBuf;
use std::process::Command;
use std::env;

fn main() -> Result<()> {
    // Compile the CUDA kernel
    let status = Command::new("nvcc")
        .args(&["-ptx", "src/cuda/kernels.cu", "-o", "src/cuda/kernels.ptx"])
        .status()
        .expect("Failed to compile CUDA kernel");

    if !status.success() {
        panic!("CUDA kernel compilation failed");
    }

    // Link the compiled kernel
    println!("cargo:rustc-link-search=native=src/cuda");
    println!("cargo:rustc-link-lib=static=kernels");

    // Re-run build.rs if the CUDA kernel changes
    println!("cargo:rerun-if-changed=src/cuda/kernels.cu");

    prost_build::compile_protos(&["proto/message.proto"], &["proto/"])?;
    tonic_build::compile_protos("proto/service1.proto")?;
    tonic_build::compile_protos("proto/service2.proto")?;
    Ok(())
}