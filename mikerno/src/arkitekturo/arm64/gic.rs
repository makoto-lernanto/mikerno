//! Minimuma GICv2-surmetilo: distribuilo kaj CPU-interfaco.
//!
//! QEMU `virt` metas la distribuilon ĉe `0x0800_0000` kaj la CPU-interfacon
//! ĉe `0x0801_0000`. A2 bezonas nur la fizikan temporizilan PPI (numero 30).

use core::arch::asm;
use core::ptr::{read_volatile, write_volatile};

const DISTRIBUILO: usize = 0x0800_0000;
const CPU_INTERFACO: usize = 0x0801_0000;

const GICD_CTLR: usize = 0x0000;
const GICD_ISENABLER: usize = 0x0100;
const GICD_IPRIORITYR: usize = 0x0400;
const GICD_ITARGETSR: usize = 0x0800;

const GICC_CTLR: usize = 0x0000;
const GICC_PMR: usize = 0x0004;
const GICC_IAR: usize = 0x000C;
const GICC_EOIR: usize = 0x0010;

/// La fizika temporizila PPI de la ĝenerala temporizilo.
pub const TEMPILA_PPI: u32 = 30;
/// Neniu akceptita interrompo (spuria).
const SPURIA: u32 = 1023;

fn legi(bazo: usize, deŝovo: usize) -> u32 {
    // SAFETY: la GIC-registroj estas MMIO, mapitaj kiel aparataro en la
    // identeca mapo de la unua gigabajto.
    unsafe { read_volatile((bazo + deŝovo) as *const u32) }
}

fn skribi(bazo: usize, deŝovo: usize, valoro: u32) {
    // SAFETY: sama MMIO-regiono kiel supre; la aliroj estas u32-aliniiĝintaj.
    unsafe { write_volatile((bazo + deŝovo) as *mut u32, valoro) };
}

fn skribi_bajton(bazo: usize, deŝovo: usize, valoro: u8) {
    // SAFETY: la distributoraj prioritataj kaj celaj registroj estas
    // bajte-atingeblaj.
    unsafe { write_volatile((bazo + deŝovo) as *mut u8, valoro) };
}

pub fn pravalorizi() {
    skribi(DISTRIBUILO, GICD_CTLR, 0);
    // PPI 30, unu bito en la unua ISENABLER-registro.
    skribi(DISTRIBUILO, GICD_ISENABLER, 1 << (TEMPILA_PPI % 32));
    skribi_bajton(DISTRIBUILO, GICD_IPRIORITYR + TEMPILA_PPI as usize, 0);
    skribi_bajton(DISTRIBUILO, GICD_ITARGETSR + TEMPILA_PPI as usize, 0x01);
    skribi(DISTRIBUILO, GICD_CTLR, 1);

    skribi(CPU_INTERFACO, GICC_PMR, 0xF0);
    skribi(CPU_INTERFACO, GICC_CTLR, 1);
    // SAFETY: barieroj certigas, ke la GIC-agordoj estas videblaj antaŭ la
    // unua interrompo.
    unsafe { asm!("dsb sy", "isb", options(nomem, nostack)) };
}

fn akcepti() -> u32 {
    legi(CPU_INTERFACO, GICC_IAR) & 0x3FF
}

fn fini(id: u32) {
    skribi(CPU_INTERFACO, GICC_EOIR, id);
}

/// Traktu unu IRQ: akceptu la identigilon, konfirmu ĝin kaj voku la
/// registritan traktilon. La konfirmo okazas antaŭ la traktilo, ĉar la
/// traktilo rajtas ŝanĝi la kuntekston.
pub fn trakti(kadro: &mut super::interrompo::InterrompaKadro) {
    let id = akcepti();
    if id == SPURIA {
        return;
    }
    fini(id);
    if !crate::interrompo::trakti_vektoron(id as usize, kadro) {
        crate::skribu!("[kerno] neatendita interrompo: GIC-id {}\n", id);
    }
}
