use std::path::Path;
use std::process::Command;

/// Build script for the hardware attestation service.
/// This script:
/// 1. Builds the Zig NVML wrapper project
/// 2. Links the resulting static library
/// 3. Sets up cargo to rebuild when Zig files change
fn main() {
    let zig_dir = Path::new("src/zig");

    // Build Zig project
    let status = Command::new("zig")
        .current_dir(zig_dir)
        .arg("build")
        .arg("-Doptimize=ReleaseFast")
        .status()
        .expect("Failed to execute Zig build command");

    if !status.success() {
        panic!("Failed to build Zig project");
    }

    // Link the static library
    println!("cargo:rustc-link-search=native={}/zig-out/lib", zig_dir.display());
    println!("cargo:rustc-link-lib=static=nvml_wrapper");

    // Rebuild if Zig files change
    println!("cargo:rerun-if-changed=src/zig/src/main.zig");
    println!("cargo:rerun-if-changed=src/zig/build.zig");
}
