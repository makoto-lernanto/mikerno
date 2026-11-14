//! Identeca MMU por la `arm64`-skeleto.
//!
//! `L0`-tabelo montras al unu `L1`-tabelo, kiu mapas la unuajn 4 GiB per
//! 1 GiB-blokoj: la unua gigabajto estas aparataro (`Device-nGnRnE`, PL011 kaj
//! GIC), la ceteraj estas normala
//! RAM (`Write-Back`, inner-shareable). La mapo estas identeca, do la bildo,
//! la stako kaj la FDT restas atingeblaj sen relokoj.

use core::arch::asm;
use core::ptr::write_volatile;

const BLOKO: u64 = 1;
const AF: u64 = 1 << 10;
const SH_ENA: u64 = 3 << 8;
const PXN: u64 = 1 << 53;
const UXN: u64 = 1 << 54;
const APARATO_ATTR: u64 = 1 << 2;

#[repr(C, align(4096))]
struct Tabelo([u64; 512]);

static mut L0: Tabelo = Tabelo([0; 512]);
static mut L1: Tabelo = Tabelo([0; 512]);

const fn normala(bazo: u64) -> u64 {
    bazo | BLOKO | AF | SH_ENA | UXN
}

const fn aparato(bazo: u64) -> u64 {
    bazo | BLOKO | AF | APARATO_ATTR | PXN | UXN
}

pub fn pravalorizi() {
    let tabelo0 = (&raw mut L0) as *mut u64;
    let tabelo1 = (&raw mut L1) as *mut u64;
    let bazo0 = tabelo0 as u64;
    let bazo1 = tabelo1 as u64;
    // SAFETY: `L0` kaj `L1` estas niaj propraj, 4 KiB-aliniiĝintaj statikaj
    // tabeloj; la skriboj okazas antaŭ ol la MMU estas ŝaltita, do ili estas
    // fizikaj.
    unsafe {
        write_volatile(tabelo0, bazo1 | 0b11); // L0[0] → L1 (valid, table)
        write_volatile(tabelo1, aparato(0));
        for indekso in 1..4u64 {
            write_volatile(tabelo1.add(indekso as usize), normala(indekso << 30));
        }

        // MAIR: atributo 0 = normala WB, atributo 1 = Device-nGnRnE.
        asm!("msr mair_el1, {}", in(reg) 0xffu64, options(nomem, nostack));

        // TCR: 48-bitaj adresoj (L0), 4 KiB-granuloj, inner WB, EPD1 (sen
        // TTBR1). IPS=2: 40-bitaj fizikaj adresoj, kiujn cortex-a53 havas.
        let tcr = 16u64 | (1 << 8) | (1 << 10) | (3 << 12) | (1 << 23) | (2 << 32);
        asm!("msr tcr_el1, {}", in(reg) tcr, options(nomem, nostack));
        asm!("msr ttbr0_el1, {}", in(reg) bazo0, options(nomem, nostack));
        asm!("dsb ish", "isb", options(nomem, nostack));
        asm!("tlbi vmalle1", "dsb ish", "isb", options(nomem, nostack));

        let mut sctlr: u64;
        asm!("mrs {}, sctlr_el1", out(reg) sctlr, options(nomem, nostack));
        sctlr |= (1 << 0) | (1 << 2) | (1 << 12); // M | C | I
        asm!("msr sctlr_el1, {}", in(reg) sctlr, options(nomem, nostack));
        asm!("isb", options(nomem, nostack));
    }
}
