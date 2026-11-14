//! La ĝenerala temporizilo de `arm64` (`CNTP_*_EL0`) kaj ĝia GIC-PPI.
//!
//! La traktilo plenumas unu tikon, reprogramas la komparilon kaj lasas la
//! planiston antaŭokupi la nunan fadenon.

use core::arch::asm;
use core::sync::atomic::{AtomicU64, Ordering};

use crate::arkitekturo::InterrompaKadro;
use crate::interrompo::InterrompaTraktilo;

/// La tika frekvenco de la planisto.
pub const FREKVENCO: u64 = 100;

static TIKOJ: AtomicU64 = AtomicU64::new(0);

struct TemporizilaTraktilo;

impl InterrompaTraktilo for TemporizilaTraktilo {
    fn trakta(&self, _kadro: &mut InterrompaKadro) {
        let tiko = TIKOJ.fetch_add(1, Ordering::Relaxed) + 1;
        programi();
        if tiko <= 5 || tiko.is_multiple_of(FREKVENCO) {
            crate::skribu!("[horloĝo] tiko {}\n", tiko);
        }
        crate::kerno::planisto::planumi();
    }
}

static TRAKTILO: TemporizilaTraktilo = TemporizilaTraktilo;

/// Registru la traktilon kaj ŝaltu la fizikan temporizilon.
pub fn pravalorizi() {
    crate::interrompo::registri(super::gic::TEMPILA_PPI as u8, &TRAKTILO);
    programi();
    // SAFETY: `CNTP_CTL_EL0.ENABLE = 1` kaj `IMASK = 0`; la komparilo jam havas
    // validan valoron el `programi`.
    unsafe {
        asm!("msr cntp_ctl_el0, {}", in(reg) 1u64, options(nomem, nostack));
    }
}

fn programi() {
    let intervalo = (super::procesoro::temporizila_frekvenco() / FREKVENCO).max(1);
    // SAFETY: skribo al la fizika temporizila komparilo nur agordas la sekvan
    // tikon; ĝi ne tuŝas memoron.
    unsafe {
        asm!("msr cntp_tval_el0, {}", in(reg) intervalo, options(nomem, nostack));
    }
}

pub fn tikoj() -> u64 {
    TIKOJ.load(Ordering::Relaxed)
}
