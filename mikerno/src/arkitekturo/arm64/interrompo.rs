//! `arm64`-esceptoj: la vektora tabelo `VBAR_EL1`, la interrompa kadro kaj la
//! dispetĉo de `SVC` el `EL0`.
//!
//! La ŝelko konservas `x0`–`x30`, `SP_EL0`, `ELR_EL1`, `SPSR_EL1`, `ESR_EL1`
//! kaj `FAR_EL1` sur la aktiva stako (la kerna stako `SP_EL1` se la escepto
//! venis el `EL0`), vokas Rust-on kaj revenas per `eret`.

use core::arch::{asm, global_asm};

use crate::arkitekturo::komuna::SistemvokaKadro;

/// `ESR_EL1.EC` de `SVC` el AArch64.
const EC_SVC64: u64 = 0x15;
/// `ESR_EL1.EC` de nekonata instrukcio.
const EC_NEKONATA: u64 = 0x10;
const EC_INSTRUKCIA_ABORTO_EL0: u64 = 0x20;
const EC_INSTRUKCIA_ABORTO_EL1: u64 = 0x21;
const EC_DATENA_ABORTO_EL0: u64 = 0x24;
const EC_DATENA_ABORTO_EL1: u64 = 0x25;

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
    /// La uzanta stakpinto (`SP_EL0`); sen signifo por kadroj el `EL1`.
    pub sp_el0: u64,
    _indekso: u64,
}

const _: () = assert!(core::mem::size_of::<InterrompaKadro>() == 320);

impl InterrompaKadro {
    /// La escepta kodo el `ESR_EL1`.
    pub fn escepta_kodo(&self) -> u64 {
        (self.esr >> 26) & 0x3F
    }

    /// Ĉu la escepto estas `SVC` (AArch64) kaj do portas sistemvokan numeron?
    pub fn estas_svc(&self) -> bool {
        self.escepta_kodo() == EC_SVC64
    }
}

impl SistemvokaKadro for InterrompaKadro {
    fn numero(&self) -> usize {
        if self.estas_svc() {
            (self.x[8] & 0xFFFF_FFFF) as usize
        } else {
            0
        }
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
        self.escepta_kodo()
    }

    fn instrukcipunkto(&self) -> u64 {
        self.elr
    }

    fn stakmontrilo(&self) -> u64 {
        if self.el_uzanto() {
            self.sp_el0
        } else {
            self.sp
        }
    }

    fn stato(&self) -> u64 {
        self.spsr
    }
}

unsafe extern "C" {
    static __vektoroj: u8;
}

// Deŝovoj de la kadro (vidu `InterrompaKadro`).
const O_SP_EL0: usize = 304;
const O_SP: usize = 296;
const O_INDEKSO: usize = 312;

global_asm!(
    r#"
    .section .text

    /* Konstruu la kadron; x9 ankoraŭ portas la interrompitan valoron. */
    .macro VEKTORO indekso, celo
        .align 7
        sub sp, sp, #320
        str x9, [sp, #72]
        mov x9, #\indekso
        str x9, [sp, {o_indekso}]
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
    ldr x9, [sp, {o_indekso}]
    str x9, [sp, #280]
    str xzr, [sp, #288]
    add x9, sp, #320
    str x9, [sp, {o_sp}]
    mrs x9, sp_el0
    str x9, [sp, {o_sp_el0}]
    mov x0, sp
    bl interrompa_dispetcho_arm64

    ldr x9, [sp, #248]
    msr elr_el1, x9
    ldr x9, [sp, #256]
    msr spsr_el1, x9
    ldr x9, [sp, {o_sp_el0}]
    msr sp_el0, x9
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
    "#,
    o_indekso = const O_INDEKSO,
    o_sp = const O_SP,
    o_sp_el0 = const O_SP_EL0,
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

/// Ĉu la escepta kodo indikas aborton, kiun la uzanta procezo kaŭzis?
fn uzanta_aborto(ec: u64) -> bool {
    matches!(
        ec,
        EC_NEKONATA
            | EC_INSTRUKCIA_ABORTO_EL0
            | EC_INSTRUKCIA_ABORTO_EL1
            | EC_DATENA_ABORTO_EL0
            | EC_DATENA_ABORTO_EL1
    )
}

fn aborta_nomo(ec: u64) -> &'static str {
    match ec {
        EC_INSTRUKCIA_ABORTO_EL0 | EC_INSTRUKCIA_ABORTO_EL1 => "instrukcia aborto",
        EC_DATENA_ABORTO_EL0 | EC_DATENA_ABORTO_EL1 => "datena aborto",
        _ => "nekonata escepto",
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn interrompa_dispetcho_arm64(kadro: *mut InterrompaKadro) {
    let kadro = unsafe { &mut *kadro };
    match kadro.vektoro {
        1 | 5 | 9 | 13 => super::gic::trakti(kadro),
        0 | 4 | 8 | 12 => {
            let ec = kadro.escepta_kodo();
            if ec == EC_SVC64 {
                let rezulto = crate::kerno::sistemvoko::voki(
                    kadro.numero(),
                    kadro.x[0],
                    kadro.x[1],
                    kadro.x[2],
                    kadro.el_uzanto(),
                );
                kadro.agordi_rezulton(rezulto);
                return;
            }
            if kadro.el_uzanto() && uzanta_aborto(ec) {
                crate::skribu!(
                    "\n[eraro] procezo {}: {} ĉe rip=0x{:016x}, far=0x{:016x}, esr=0x{:x}; la procezo estas mortigata\n",
                    crate::kerno::planisto::nuna_procezo(),
                    aborta_nomo(ec),
                    kadro.elr,
                    kadro.far,
                    kadro.esr
                );
                // `morti` detruas la procezan adresan spacon, elektas alian
                // fadenon kaj neniam revenas al ĉi tiu kadro.
                crate::kerno::planisto::morti()
            }
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
