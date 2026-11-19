//! Ligas la uzantan bildon per sia propra skripto, ne per la kerna.

use std::env;

fn main() {
    let radiko = env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR mankas")
        .replace('\\', "/");
    println!("cargo::rerun-if-changed=linker.ld");
    println!("cargo::rustc-link-arg=-T{radiko}/linker.ld");
    println!("cargo::rustc-link-arg=-no-pie");
    // La defaŭlta 64 KiB-pagegigo de `lld` lasas 64 KiB-an truon antaŭ
    // `.text`; por la arm64-bildo tio superus la 64 KiB-an dosierlimon de
    // SNDS. Kun 4 KiB la segmentaj adresoj restas la samaj kaj la dosiero
    // fidas en la formato.
    println!("cargo::rustc-link-arg=-z");
    println!("cargo::rustc-link-arg=max-page-size=4096");
}
