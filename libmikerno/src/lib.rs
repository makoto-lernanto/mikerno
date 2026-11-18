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

/// Versio de la komuna ABI. `2` aldonas la tenilojn, la novajn permesojn, la
/// erar-kodojn `-9/-11/-28/-32/-90/-110` kaj la sistemvokojn 6–11.
pub const ABI_VERSIO: u32 = 2;

pub use kapablo::{FinaPunkto, KOMENCA_TENILO, Kapablo, Permesoj, Tenilo};
pub use mesago::{MAKSA_MESAGA_LONGO, Mesago};
pub use starto::{
    MAGIA_NUMERO, REĜIMO_SEKRETO, REĜIMO_SPIONO, SEKRETA_ADRESO, SEKRETA_DUA_ADRESO,
    SEKRETA_ID_DEŜOVO,
};
