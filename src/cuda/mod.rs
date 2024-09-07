use rust_cuda::prelude::*;
use std::ffi::CString;
use std::ptr;

pub fn add_vectors(a: &[f32], b: &[f32], c: &mut [f32]) {
    let n = a.len();
    assert_eq!(n, b.len());
    assert_eq!(n, c.len());

    // Load the CUDA module
    let module_data = include_bytes!("kernels.ptx");
    let module = Module::load_from_bytes(module_data).expect("Failed to load CUDA module");

    // Get the kernel function
    let kernel = module.get_function("add_vectors").expect("Failed to get kernel function");

    // Allocate device memory
    let mut d_a = DeviceBuffer::from_slice(a).expect("Failed to allocate device memory for a");
    let mut d_b = DeviceBuffer::from_slice(b).expect("Failed to allocate device memory for b");
    let mut d_c = DeviceBuffer::from_slice(c).expect("Failed to allocate device memory for c");

    // Launch the kernel
    let grid_size = (n as u32 + 255) / 256;
    let block_size = 256;
    unsafe {
        kernel.launch(&[grid_size, 1, 1], &[block_size, 1, 1], &[
            &d_a.as_device_ptr(),
            &d_b.as_device_ptr(),
            &d_c.as_device_ptr(),
            &n,
        ]).expect("Failed to launch kernel");
    }

    // Copy the result back to the host
    d_c.copy_to(c).expect("Failed to copy result to host");
}