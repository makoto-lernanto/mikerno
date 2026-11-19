//! Ligas la bildon de `saluto` per propra skripto, same kiel `horizonto`.

use std::env;

fn main() {
    let radiko = env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR mankas")
        .replace('\\', "/");
    println!("cargo::rerun-if-changed=linker.ld");
    println!("cargo::rustc-link-arg=-T{radiko}/linker.ld");
    println!("cargo::rustc-link-arg=-no-pie");
    println!("cargo::rustc-link-arg=-z");
    println!("cargo::rustc-link-arg=max-page-size=4096");
}
