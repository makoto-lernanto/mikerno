//! Liveras la kernan ligilon sendepende de la kuranta dosierujo.
//!
//! La ligilaj argumentoj apartenas al la kerna crate; meti ilin en la radika
//! `.cargo/config.toml` aplikus la kernan `linker.ld` ankaŭ al `horizonto`.

use std::env;

fn main() {
    let radiko = env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR mankas")
        .replace('\\', "/");
    let arkitekturo = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    let arm64 = arkitekturo == "aarch64";
    let skripto = if arm64 {
        "boot/linker-arm64.ld"
    } else {
        "boot/linker.ld"
    };
    println!("cargo::rerun-if-changed={skripto}");
    println!("cargo::rustc-link-arg=-T{radiko}/{skripto}");
    // La x86_64-bildo estas PIE (ET_DYN): ĉiuj difinoj estas lokaj, do la ŝelko
    // kaj la kerno alvokas ilin sen PLT, kiun neniu dinamika ligilo plenumus.
    if !arm64 {
        println!("cargo::rustc-link-arg=-Bsymbolic");
    }
}
