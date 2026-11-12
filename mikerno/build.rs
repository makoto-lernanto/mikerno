//! Liveras la kernan ligilon sendepende de la kuranta dosierujo.
//!
//! La ligilaj argumentoj apartenas al la kerna crate; meti ilin en la radika
//! `.cargo/config.toml` aplikus la kernan `linker.ld` ankaŭ al `horizonto`.

use std::env;

fn main() {
    let radiko = env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR mankas")
        .replace('\\', "/");
    println!("cargo::rerun-if-changed=boot/linker.ld");
    println!("cargo::rustc-link-arg=-T{radiko}/boot/linker.ld");
    println!("cargo::rustc-link-arg=-no-pie");
}
