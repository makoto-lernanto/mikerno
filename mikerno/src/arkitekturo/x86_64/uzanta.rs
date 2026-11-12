//! Transiro Ring 0 → Ring 3 per `iretq`.
//!
//! `iretq` restarigas `RIP`, `CS`, `RFLAGS`, `RSP` kaj `SS`. Kiam la cela `CS`
//! havas RPL 3, la procesoro ankaŭ ŝaltas la uzantan stakon; la TSS jam tenas
//! `RSP0`, do la unua reveno Ring 3 → Ring 0 (interrompo aŭ `int 0x80`) trovas
//! la kernan stakon.

use core::arch::asm;

/// La GDT-selektiloj el `gdt.rs`: uzantaj datumoj `0x18`, uzanta kodo `0x20`.
const UZANTA_CS: u64 = 0x23;
const UZANTA_SS: u64 = 0x1B;
/// `IF=1` kaj la ĉiama rezervita bito 1.
const UZANTA_RFLAGS: u64 = 0x202;

/// Transiru al Ring 3 kaj neniam revenu.
///
/// `reĝimo` kaj `identigilo` estas transdonataj en `RDI` kaj `RSI`, do
/// `extern "C"`-enirpunkto ricevas ilin kiel siajn unuajn du argumentojn.
///
/// # Sekureco
/// `enirpunkto` devas esti plenumebla uzanta adreso de la ŝargita bildo, kaj
/// `stakpinto` devas esti la pinto de mapita, skribebla uzanta stako. La
/// alvokanto devas esti en Ring 0 kun la TSS ŝargita, ĉar la unua reveno al la
/// kerno uzos `RSP0`.
pub unsafe fn transiri_al_ring3(
    enirpunkto: u64,
    stakpinto: u64,
    reĝimo: u64,
    identigilo: u64,
) -> ! {
    // La x86-64-ABI atendas `rsp % 16 == 8` ĉe funkcia eniro (kvazaŭ `call`
    // ĵus puŝus revenadreson); `iretq` ne puŝas unu, do ni mem malaltigas la
    // stakpinton.
    let uzanta_rsp = stakpinto - 8;
    unsafe {
        asm!(
            "cli",
            "push {ss}",
            "push {sp}",
            "push {rflags}",
            "push {cs}",
            "push {rip}",
            "iretq",
            ss = in(reg) UZANTA_SS,
            sp = in(reg) uzanta_rsp,
            rflags = in(reg) UZANTA_RFLAGS,
            cs = in(reg) UZANTA_CS,
            rip = in(reg) enirpunkto,
            in("rdi") reĝimo,
            in("rsi") identigilo,
            options(noreturn),
        );
    }
}
