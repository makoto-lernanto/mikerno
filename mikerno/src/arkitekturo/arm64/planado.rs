//! `arm64`-kuntekstŝanĝo kaj la unua eniro de nova fadeno.
//!
//! La ŝalto konservas la kaliprezerbitajn registrojn `x19`–`x28`, la kadran
//! montrilon `x29`, la revenadreson `x30`, la stakmontrilon kaj `DAIF`. La
//! reveno okazas per `ret`, do `x30` montras al la revenadreso de la alvoko.
//!
//! Nova fadeno komenciĝas per artefarita stako: `x30` montras al
//! `fadeno_trampolino`, `x19` portas la fadenan identigilon, kaj `sp` montras
//! al la pinto de la fadena stako. La trampolino malmaskas `IRQ` kaj transdonas
//! la identigilon al Rust.

use core::arch::global_asm;
use core::mem::{offset_of, size_of};

/// La konservita kunteksto de fadeno.
///
/// La aranĝo estas fiksa (`#[repr(C)]`): la muntita ŝanĝo uzas fiksajn
/// deŝovojn.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Kunteksto {
    pub x19: u64,
    pub x20: u64,
    pub x21: u64,
    pub x22: u64,
    pub x23: u64,
    pub x24: u64,
    pub x25: u64,
    pub x26: u64,
    pub x27: u64,
    pub x28: u64,
    pub x29: u64,
    pub x30: u64,
    pub sp: u64,
    pub daif: u64,
}

impl Kunteksto {
    pub const fn nova() -> Self {
        Self {
            x19: 0,
            x20: 0,
            x21: 0,
            x22: 0,
            x23: 0,
            x24: 0,
            x25: 0,
            x26: 0,
            x27: 0,
            x28: 0,
            x29: 0,
            x30: 0,
            sp: 0,
            daif: 0,
        }
    }

    /// Metu la unuan argumenton, kiun la trampolino transdonas al Rust.
    pub fn agordi_unuan_argumenton(&mut self, identigilo: u64) {
        self.x19 = identigilo;
    }
}

const O_X19: usize = offset_of!(Kunteksto, x19);
const O_X21: usize = offset_of!(Kunteksto, x21);
const O_X23: usize = offset_of!(Kunteksto, x23);
const O_X25: usize = offset_of!(Kunteksto, x25);
const O_X27: usize = offset_of!(Kunteksto, x27);
const O_X29: usize = offset_of!(Kunteksto, x29);
const O_SP: usize = offset_of!(Kunteksto, sp);
const O_DAIF: usize = offset_of!(Kunteksto, daif);

const _: () = assert!(size_of::<Kunteksto>() == 14 * 8);

global_asm!(
    r#"
    .section .text

    /* x0 = malnova kunteksto, x1 = nova kunteksto */
    .p2align 4
    .global sxalti_kuntekston_asm
sxalti_kuntekston_asm:
    stp x19, x20, [x0, {o_x19}]
    stp x21, x22, [x0, {o_x21}]
    stp x23, x24, [x0, {o_x23}]
    stp x25, x26, [x0, {o_x25}]
    stp x27, x28, [x0, {o_x27}]
    stp x29, x30, [x0, {o_x29}]
    mov x9, sp
    str x9, [x0, {o_sp}]
    mrs x9, daif
    str x9, [x0, {o_daif}]

    ldp x19, x20, [x1, {o_x19}]
    ldp x21, x22, [x1, {o_x21}]
    ldp x23, x24, [x1, {o_x23}]
    ldp x25, x26, [x1, {o_x25}]
    ldp x27, x28, [x1, {o_x27}]
    ldp x29, x30, [x1, {o_x29}]
    ldr x9, [x1, {o_sp}]
    mov sp, x9
    ldr x9, [x1, {o_daif}]
    msr daif, x9
    ret

    /* Unua eniro: x19 = fadeno-id, sp = pinto de la fadena stako. */
    .p2align 4
    .global fadeno_trampolino
fadeno_trampolino:
    msr daifclr, #2
    mov x0, x19
    bl fadeno_eniro
    brk #0
    "#,
    o_x19 = const O_X19,
    o_x21 = const O_X21,
    o_x23 = const O_X23,
    o_x25 = const O_X25,
    o_x27 = const O_X27,
    o_x29 = const O_X29,
    o_sp = const O_SP,
    o_daif = const O_DAIF,
);

unsafe extern "C" {
    fn sxalti_kuntekston_asm(malnova: *mut Kunteksto, nova: *const Kunteksto);
    static fadeno_trampolino: u8;
}

/// Konservu la nunan kuntekston kaj restarigu la donitan.
///
/// # Sekureco
/// Ambaŭ montriloj devas montri al vivantaj, ĝuste aliniiĝintaj `Kunteksto`
/// valoroj. La funkcio neniam revenas per ordinara vojo se la celo estas
/// neniam planita; alie ĝi revenas, kiam la nuna kunteksto restariĝas.
pub unsafe fn ŝalti_kuntekston(malnova: *mut Kunteksto, nova: *const Kunteksto) {
    unsafe { sxalti_kuntekston_asm(malnova, nova) }
}

/// Pretigu kuntekston tiel, ke la unua `ret` eniros `fadeno_trampolino`.
pub fn prepari_kuntekston(konteksto: &mut Kunteksto, stako_pinto: u64) {
    *konteksto = Kunteksto::nova();
    konteksto.x30 = &raw const fadeno_trampolino as u64;
    konteksto.sp = stako_pinto;
    // Ĉiuj interrompoj restas maskitaj ĝis la trampolino; tiel la ŝalto ne
    // povas esti interrompita antaŭ ol la nova fadeno havas validan stakon.
    konteksto.daif = 0x3C0;
}
