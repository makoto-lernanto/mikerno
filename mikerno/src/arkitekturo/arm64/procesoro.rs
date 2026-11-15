//! Procesora kontrolo de `arm64`: `DAIF`, `WFI`, la ĝenerala temporizilo kaj
//! PSCI-haltigo.

use core::arch::asm;

/// PSCI `SYSTEM_OFF` (32-bita funkcia identigilo).
const PSCI_SYSTEM_OFF: u64 = 0x8400_0008;
/// `PSTATE.I` (bit 7): `IRQ` estas maskita.
const DAIF_I: u64 = 1 << 7;

/// Haltigu la maŝinon per PSCI `SYSTEM_OFF`; se la hipervizitilo rifuzas,
/// restu en `WFI`-buklo.
pub fn halti() -> ! {
    // SAFETY: `hvc #0` kun PSCI-funkcia identigilo estas la dokumentita vojo;
    // QEMU `virt` liveras la PSCI-servon per la sama kanalo.
    unsafe {
        asm!(
            "hvc #0",
            in("x0") PSCI_SYSTEM_OFF,
            options(nomem, nostack),
        );
    }
    loop {
        atendi_interrompon();
    }
}

/// La nuna `DAIF`-stato.
pub fn daif() -> u64 {
    let valoro: u64;
    // SAFETY: legado de `DAIF` ne havas kromefikojn.
    unsafe { asm!("mrs {}, daif", out(reg) valoro, options(nomem, nostack)) };
    valoro
}

pub fn interrompoj_ebligitaj() -> bool {
    daif() & DAIF_I == 0
}

pub fn malebligi_interrompojn() {
    // SAFETY: `DAIFSet` nur maskas `IRQ`.
    unsafe { asm!("msr daifset, #2", options(nomem, nostack)) };
}

pub fn ebligi_interrompojn() {
    // SAFETY: `DAIFClr` nur malmaskas `IRQ`.
    unsafe { asm!("msr daifclr, #2", options(nomem, nostack)) };
}

pub fn atendi_interrompon() {
    // SAFETY: `wfi` dormigas la procesoron ĝis interrompo.
    unsafe { asm!("wfi", options(nomem, nostack)) };
}

/// La frekvenco de la ĝenerala temporizilo (`CNTFRQ_EL0`).
pub fn temporizila_frekvenco() -> u64 {
    let valoro: u64;
    // SAFETY: `CNTFRQ_EL0` estas nurlega systema registro.
    unsafe { asm!("mrs {}, cntfrq_el0", out(reg) valoro, options(nomem, nostack)) };
    valoro
}

/// Ĉu la procesoro havas `PAN`-on (`ID_AA64MMFR1_EL1.PAN`)?
pub fn pan_subtenata() -> bool {
    let valoro: u64;
    // SAFETY: `ID_AA64MMFR1_EL1` estas nurlega identiga registro.
    unsafe { asm!("mrs {}, id_aa64mmfr1_el1", out(reg) valoro, options(nomem, nostack)) };
    (valoro >> 20) & 0xF != 0
}

/// Malfermu la `PAN`-fenestron: `EL1` rajtas legi kaj skribi uzantajn paĝojn.
///
/// Vokenda nur se `pan_subtenata()`; malfermita fenestro fermiĝas per
/// `fermi_pan` aŭ per la sekva escepta eniro (`SCTLR_EL1.SPAN = 0`).
pub fn malfermi_pan() {
    // SAFETY: la alvokanto certigas, ke `PAN` estas efektivigita.
    unsafe { asm!("msr pan, #0", options(nomem, nostack)) };
}

/// Fermu la `PAN`-fenestron.
pub fn fermi_pan() {
    // SAFETY: la alvokanto certigas, ke `PAN` estas efektivigita.
    unsafe { asm!("msr pan, #1", options(nomem, nostack)) };
}
