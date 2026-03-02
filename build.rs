// build.rs - Build script for the Steam Emulator
use std::env;

fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();

    if target_os == "windows" {
        println!("cargo:rustc-link-lib=user32");
        println!("cargo:rustc-link-lib=kernel32");
        println!("cargo:rustc-link-lib=d3d11");
        println!("cargo:rustc-link-lib=dxgi");
        println!("cargo:rustc-link-arg=/SUBSYSTEM:WINDOWS");
    }

    let version = env::var("CARGO_PKG_VERSION").unwrap();
    println!("cargo:rustc-env=STEAM_EMU_VERSION={}", version);
}
