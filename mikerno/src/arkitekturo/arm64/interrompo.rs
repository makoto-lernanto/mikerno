//! `arm64`-esceptoj: la vektora tabelo `VBAR_EL1` kaj la interrompa kadro.
//!
//! La ŝelko konservas `x0`–`x30`, `ELR_EL1`, `SPSR_EL1`, `ESR_EL1` kaj
//! `FAR_EL1` sur la aktiva stako, vokas Rust-on kaj revenas per `eret`.

use core::arch::{asm, global_asm};

use crate::arkitekturo::komuna::SistemvokaKadro;

/// La kadro, kiun la enira ŝelko konstruas sur la stako.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct InterrompaKadro {
    pub x: [u64; 31],
    pub elr: u64,
    pub spsr: u64,
    pub esr: u64,
    pub far: u64,
    pub vektoro: u64,
    pub erarkodo: u64,
    pub sp: u64,
    _rezervo: u64,
    _indekso: u64,
}

const _: () = assert!(core::mem::size_of::<InterrompaKadro>() == 320);

impl SistemvokaKadro for InterrompaKadro {
    fn numero(&self) -> usize {
        0
    }

    fn argumento(&self, indekso: usize) -> u64 {
        self.x.get(indekso).copied().unwrap_or(0)
    }

    fn agordi_rezulton(&mut self, rezulto: u64) {
        self.x[0] = rezulto;
    }

    fn el_uzanto(&self) -> bool {
        self.spsr & 0xF == 0
    }

    fn vektoro(&self) -> u64 {
        self.vektoro
    }

    fn erarkodo(&self) -> u64 {
        self.erarkodo
    }

    fn instrukcipunkto(&self) -> u64 {
        self.elr
    }

    fn stakmontrilo(&self) -> u64 {
        self.sp
    }

    fn stato(&self) -> u64 {
        self.spsr
    }
}

unsafe extern "C" {
    static __vektoroj: u8;
}

global_asm!(
    r#"
    .section .text

    /* Konstruu la kadron; x9 ankoraŭ portas la interrompitan valoron. */
    .macro VEKTORO indekso, celo
        .align 7
        sub sp, sp, #320
        str x9, [sp, #72]
        mov x9, #\indekso
        str x9, [sp, #312]
        b \celo
    .endm

    .align 11
    .global __vektoroj
__vektoroj:
    VEKTORO 0, .Lsync
    VEKTORO 1, .Lirq
    VEKTORO 2, .Lfatal
    VEKTORO 3, .Lfatal
    VEKTORO 4, .Lsync
    VEKTORO 5, .Lirq
    VEKTORO 6, .Lfatal
    VEKTORO 7, .Lfatal
    VEKTORO 8, .Lsync
    VEKTORO 9, .Lirq
    VEKTORO 10, .Lfatal
    VEKTORO 11, .Lfatal
    VEKTORO 12, .Lsync
    VEKTORO 13, .Lirq
    VEKTORO 14, .Lfatal
    VEKTORO 15, .Lfatal

.Lirq:
.Lsync:
.Lfatal:
    stp x0, x1, [sp, #0]
    stp x2, x3, [sp, #16]
    stp x4, x5, [sp, #32]
    stp x6, x7, [sp, #48]
    str x8, [sp, #64]
    stp x10, x11, [sp, #80]
    stp x12, x13, [sp, #96]
    stp x14, x15, [sp, #112]
    stp x16, x17, [sp, #128]
    stp x18, x19, [sp, #144]
    stp x20, x21, [sp, #160]
    stp x22, x23, [sp, #176]
    stp x24, x25, [sp, #192]
    stp x26, x27, [sp, #208]
    stp x28, x29, [sp, #224]
    str x30, [sp, #240]
    mrs x9, elr_el1
    str x9, [sp, #248]
    mrs x9, spsr_el1
    str x9, [sp, #256]
    mrs x9, esr_el1
    str x9, [sp, #264]
    mrs x9, far_el1
    str x9, [sp, #272]
    ldr x9, [sp, #312]
    str x9, [sp, #280]
    str xzr, [sp, #288]
    add x9, sp, #320
    str x9, [sp, #296]
    mov x0, sp
    bl interrompa_dispetcho_arm64

    ldr x9, [sp, #248]
    msr elr_el1, x9
    ldr x9, [sp, #256]
    msr spsr_el1, x9
    ldp x0, x1, [sp, #0]
    ldp x2, x3, [sp, #16]
    ldp x4, x5, [sp, #32]
    ldp x6, x7, [sp, #48]
    ldr x8, [sp, #64]
    ldr x9, [sp, #72]
    ldp x10, x11, [sp, #80]
    ldp x12, x13, [sp, #96]
    ldp x14, x15, [sp, #112]
    ldp x16, x17, [sp, #128]
    ldp x18, x19, [sp, #144]
    ldp x20, x21, [sp, #160]
    ldp x22, x23, [sp, #176]
    ldp x24, x25, [sp, #192]
    ldp x26, x27, [sp, #208]
    ldp x28, x29, [sp, #224]
    ldr x30, [sp, #240]
    add sp, sp, #320
    eret
    "#
);

/// Instalu la vektoran tabelon.
pub fn pravalorizi() {
    let bazo = &raw const __vektoroj as u64;
    // SAFETY: `__vektoroj` estas 2 KiB-aliniiĝinta tabelo de 16 eniroj.
    unsafe {
        asm!("msr vbar_el1, {}", in(reg) bazo, options(nomem, nostack));
        asm!("isb", options(nomem, nostack));
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn interrompa_dispetcho_arm64(kadro: *mut InterrompaKadro) {
    let kadro = unsafe { &mut *kadro };
    match kadro.vektoro {
        1 | 5 | 9 | 13 => super::gic::trakti(kadro),
        0 | 4 | 8 | 12 => {
            crate::skribu!(
                "\n[kerno] escepto: ESR=0x{:x}, FAR=0x{:x}, ELR=0x{:x}, SPSR=0x{:x}\n",
                kadro.esr,
                kadro.far,
                kadro.elr,
                kadro.spsr
            );
            crate::arkitekturo::halti()
        }
        alia => {
            crate::skribu!("\n[kerno] neatendita escepto: vektoro {}\n", alia);
            crate::arkitekturo::halti()
        }
    }
}
