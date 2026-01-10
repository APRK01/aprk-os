// =============================================================================
// APRK OS - ARM64 Build Script
// =============================================================================
// This build script compiles the assembly boot code and links it with Rust.
// =============================================================================

use std::env;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let arch_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    
    // Tell Cargo to rerun this if assembly files change
    println!("cargo:rerun-if-changed=src/boot.S");
    println!("cargo:rerun-if-changed=src/exception.S");
    
    // Compile boot.S and exception.S
    cc::Build::new()
        .file(arch_dir.join("src/boot.S"))
        .file(arch_dir.join("src/exception.S"))
        .file(arch_dir.join("src/context.S"))
        .flag("-c")
        .flag("-target")
        .flag("aarch64-unknown-none")
        .compile("boot");
    
    // Link the compiled object file
    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-link-lib=static=boot");
}
