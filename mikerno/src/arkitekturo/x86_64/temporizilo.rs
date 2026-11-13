//! APIC-temporizilo en perioda reĝimo (100 Hz).
//!
//! La APIC-busa frekvenco ne estas arkitekture legebla, do ni mezuras ĝin
//! kontraŭ la PIT (1,193182 MHz) per unuopa 50 ms kalibrado. Sen tio la
//! komenca kalkulo estus ĝusta nur por 100 MHz-an buson havantaj CPU-oj.

use core::arch::asm;
use core::sync::atomic::{AtomicU64, Ordering};

use crate::interrompo::{InterrompaKadro, InterrompaTraktilo};

use super::apic;

pub const VEKTORO: u8 = 0x20;
pub const TIKOJ_PO_SEKUNDO: u32 = 100;

/// Divido 16: `REGISTRO_TEMPORIZILA_DIVIDO` = 0x3 donas `count_shift` 4.
const DIVIDO_PER_16: u32 = 0x3;
const PERIODA_BITO: u32 = 1 << 17;
const MASKA_BITO: u32 = 1 << 16;

const PIT_FREKVENTO: u32 = 1_193_182;
const PIT_KANALO_2: u16 = 0x42;
const PIT_KOMANDO: u16 = 0x43;
const PIT_PORTO_61: u16 = 0x61;
/// ≈ 50 ms da PIT-tempo.
const KALIBRA_KALKULO: u16 = 59_659;
/// Sekuriga limo, se la PIT neniam raportos finon.
const KALIBRA_LIMO: u32 = 1_000_000;
/// Norma APIC-buso (100 MHz) dividita per 16, se la kalibrado malsukcesas.
const NORMA_PO_SEKUNDO: u32 = 6_250_000;

static TIKOJ: AtomicU64 = AtomicU64::new(0);

struct TemporizilaTraktilo;

impl InterrompaTraktilo for TemporizilaTraktilo {
    fn trakta(&self, _kadro: &mut InterrompaKadro) {
        let tiko = TIKOJ.fetch_add(1, Ordering::Relaxed) + 1;
        if tiko <= 5 || tiko.is_multiple_of(100) {
            crate::skribu!("[horloĝo] tiko {}\n", tiko);
        }
        // La aparata EOI jam estis sendita de la dispetĉo.
        crate::kerno::planisto::planumi();
    }
}

static TRAKTILO: TemporizilaTraktilo = TemporizilaTraktilo;

fn elporti(porto: u16) -> u8 {
    let valoro: u8;
    unsafe {
        asm!(
            "in al, dx",
            out("al") valoro,
            in("dx") porto,
            options(nomem, nostack, preserves_flags),
        );
    }
    valoro
}

fn elportigi(porto: u16, valoro: u8) {
    unsafe {
        asm!(
            "out dx, al",
            in("dx") porto,
            in("al") valoro,
            options(nomem, nostack, preserves_flags),
        );
    }
}

/// Mezuru la APIC-kalkulojn dum ≈50 ms da PIT-tempo.
///
/// La APIC-temporizilo jam funkcias unuope kun plena kalkulo; la PIT-kanalo 2
/// estas programita samtempe, do la mezuro kovras ĝuste unu PIT-periodon.
fn kalibri() -> Option<u32> {
    let stato = elporti(PIT_PORTO_61);
    // Kanala pordego alta, laŭtparolilo malaktiva.
    elportigi(PIT_PORTO_61, (stato & !0x02) | 0x01);

    apic::skribi(apic::REGISTRO_LVT_TEMPORIZILO, VEKTORO as u32 | MASKA_BITO);
    apic::skribi(apic::REGISTRO_TEMPORIZILA_KOMENCA, u32::MAX);

    // Kanala 2, modo 0 (unuopa), malalta/post alta.
    elportigi(PIT_KOMANDO, 0xB0);
    elportigi(PIT_KANALO_2, KALIBRA_KALKULO as u8);
    elportigi(PIT_KANALO_2, (KALIBRA_KALKULO >> 8) as u8);

    let unua = apic::legi(apic::REGISTRO_TEMPORIZILA_NUNA);
    let mut provoj = 0u32;
    while elporti(PIT_PORTO_61) & 0x20 == 0 {
        provoj += 1;
        if provoj >= KALIBRA_LIMO {
            elportigi(PIT_PORTO_61, stato);
            return None;
        }
        core::hint::spin_loop();
    }
    let lasta = apic::legi(apic::REGISTRO_TEMPORIZILA_NUNA);
    elportigi(PIT_PORTO_61, stato);

    let pasintaj = unua.wrapping_sub(lasta) as u64;
    let po_sekundo = pasintaj * PIT_FREKVENTO as u64 / KALIBRA_KALKULO as u64;
    u32::try_from(po_sekundo).ok().filter(|&valoro| valoro > 0)
}

pub fn pravalorizi() {
    crate::interrompo::registri(VEKTORO, &TRAKTILO);

    apic::skribi(apic::REGISTRO_TEMPORIZILA_DIVIDO, DIVIDO_PER_16);

    let po_sekundo = match kalibri() {
        Some(valoro) => valoro,
        None => {
            crate::skribu!(
                "[temporizilo] kalibrado malsukcesis; supozu norman 100 MHz APIC-buson\n"
            );
            NORMA_PO_SEKUNDO
        }
    };
    let komenca = (po_sekundo / TIKOJ_PO_SEKUNDO).max(1);
    crate::skribu!(
        "[temporizilo] kalibrado: {} kalkuloj/s, komenca {}\n",
        po_sekundo,
        komenca
    );

    apic::skribi(
        apic::REGISTRO_LVT_TEMPORIZILO,
        VEKTORO as u32 | PERIODA_BITO | MASKA_BITO,
    );
    apic::skribi(apic::REGISTRO_TEMPORIZILA_KOMENCA, komenca);
    apic::skribi(
        apic::REGISTRO_LVT_TEMPORIZILO,
        VEKTORO as u32 | PERIODA_BITO,
    );
}

pub fn tikoj() -> u64 {
    TIKOJ.load(Ordering::Relaxed)
}
