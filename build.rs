fn main() {
    // Ensure the vendored `protoc` is available
    protoc_bin_vendored::protoc_bin_path().expect("Protoc not found");

    // Compile the protobuf files
    tonic_build::compile_protos("proto/rapidmq.proto").unwrap();
}