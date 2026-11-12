//! Komuna API de MiKerno: la sama protokolo por la kerno kaj por la uzantspaco.
//!
//! La crate estas `no_std` kaj sen dependaĵoj. Ĝi fiksas la datumaran aranĝon
//! (`#[repr(C)]`) kaj la sistemvokajn numerojn; la efektivigo restas en la
//! kerno kaj la alvokoj en la uzanta rultempo.

#![no_std]

pub mod kapablo;
pub mod mesago;
pub mod starto;
pub mod syscall;

pub use kapablo::{FinaPunkto, Kapablo, Permesoj};
pub use mesago::{MAKSA_MESAGA_LONGO, Mesago};
pub use starto::{REĜIMO_FINI, REĜIMO_KRUI};
