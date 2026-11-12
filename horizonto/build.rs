//! Ligas la uzantan bildon per sia propra skripto, ne per la kerna.

use std::env;

fn main() {
    let radiko = env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR mankas")
        .replace('\\', "/");
    println!("cargo::rerun-if-changed=linker.ld");
    println!("cargo::rustc-link-arg=-T{radiko}/linker.ld");
    println!("cargo::rustc-link-arg=-no-pie");
}
