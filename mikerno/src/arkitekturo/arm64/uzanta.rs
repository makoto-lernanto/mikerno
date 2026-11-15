//! Transiro `EL1` → `EL0` per `eret`.
//!
//! `eret` restarigas `ELR_EL1`, `SPSR_EL1` kaj, se `SPSR_EL1.M` estas `EL0t`,
//! ŝaltas `SP_EL0`. La pli fruaj vektora ŝelko kaj la per-proceza kerna stako
//! jam pretas, do la unua reveno `EL0` → `EL1` (interrompo aŭ `svc #0`) trovas
//! la procezan stakon en `SP_EL1`.

use core::arch::global_asm;

/// La ELF-maŝina identigilo de `aarch64` (norma `EM_AARCH64`).
pub const ELF_MAŜINO: u16 = 183;

global_asm!(
    r#"
    .section .text
    .global eniri_uzanton_arm64
eniri_uzanton_arm64:
    /* x0 = enirpunkto, x1 = stakpinto, x2 = reĝimo, x3 = identigilo */
    msr sp_el0, x1
    msr elr_el1, x0
    msr spsr_el1, xzr
    mov x0, x2
    mov x1, x3
    mov x2, xzr
    mov x3, xzr
    mov x4, xzr
    mov x5, xzr
    mov x6, xzr
    mov x7, xzr
    mov x8, xzr
    mov x9, xzr
    mov x10, xzr
    mov x11, xzr
    mov x12, xzr
    mov x13, xzr
    mov x14, xzr
    mov x15, xzr
    mov x16, xzr
    mov x17, xzr
    mov x18, xzr
    mov x19, xzr
    mov x20, xzr
    mov x21, xzr
    mov x22, xzr
    mov x23, xzr
    mov x24, xzr
    mov x25, xzr
    mov x26, xzr
    mov x27, xzr
    mov x28, xzr
    mov x29, xzr
    mov x30, xzr
    eret
    "#
);

unsafe extern "C" {
    fn eniri_uzanton_arm64(enirpunkto: u64, stakpinto: u64, reĝimo: u64, identigilo: u64) -> !;
}

/// Transiru al `EL0` kaj neniam revenu.
///
/// `reĝimo` kaj `identigilo` estas transdonataj en `x0` kaj `x1`, do
/// `extern "C"`-enirpunkto ricevas ilin kiel siajn unuajn du argumentojn.
///
/// # Sekureco
/// `enirpunkto` devas esti plenumebla uzanta adreso de la ŝargita bildo, kaj
/// `stakpinto` devas esti la pinto de mapita, skribebla uzanta stako. La
/// alvokanto devas plenumiĝi je `EL1` kun `SPSel = 1`.
pub unsafe fn eniri_uzanton(enirpunkto: u64, stakpinto: u64, reĝimo: u64, identigilo: u64) -> ! {
    // SAFETY: la alvokanto garantias la antaŭkondiĉojn; la funkcio neniam
    // revenas.
    unsafe { eniri_uzanton_arm64(enirpunkto, stakpinto, reĝimo, identigilo) }
}
