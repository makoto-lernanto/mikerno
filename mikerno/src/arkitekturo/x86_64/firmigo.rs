//! Kerna firmigo de la Ring 0/Ring 3 limo: `CR4.SMEP`.
//!
//! SMEP malpermesas al Ring 0 plenumi foliojn kun la bito `U`. La subteno
//! venas el `procesoro::trajtoj()`; se la procesoro ne havas ĝin, la kerno
//! daŭras sen la barilo kaj la memprovo estas preterlasita.

use core::arch::asm;

use super::pagxado::X86AdresaSpaco;
use super::procesoro;
use crate::memoro::KADRA_GRANDO;
use crate::memoro::kadro::KadraAlokilo;
use crate::memoro::virtuala::{AdresaSpaco, PaĝaPermesoj, PaĝaTablo, UZANTA_BAZO, VirtuaAdreso};

const SMEP_BITO: u64 = 1 << 20;

/// Ŝaltu `CR4.SMEP` se la procesoro subtenas ĝin.
pub fn pravalorizi() {
    if !procesoro::trajtoj().smep {
        return;
    }
    // SAFETY: `CR4.SMEP` (bito 20) estas la sola bito ŝanĝata; la ceteraj
    // `CR4`-bitoj restas kiel la praŝarĝa ŝelko lasis ilin.
    unsafe {
        let mut cr4: u64;
        asm!(
            "mov {}, cr4",
            out(reg) cr4,
            options(nomem, nostack, preserves_flags),
        );
        asm!(
            "mov cr4, {}",
            in(reg) cr4 | SMEP_BITO,
            options(nomem, nostack, preserves_flags),
        );
    }
}

/// Ĉu `CR4.SMEP` estas efektive ŝaltita?
pub fn aktiva() -> bool {
    let cr4: u64;
    // SAFETY: legado de `CR4` ne havas kromefikojn.
    unsafe {
        asm!(
            "mov {}, cr4",
            out(reg) cr4,
            options(nomem, nostack, preserves_flags),
        );
    }
    cr4 & SMEP_BITO != 0
}

/// Pruvu, ke Ring 0 ne povas plenumi uzantan folion.
///
/// La funkcio mapas unu `U=1`-kodon en la aktivan kernan spacon, skribas solan
/// `ret`-instrukciaron kaj vokas ĝin per `call`. Se SMEP funkcias, la `#PF`-
/// traktilo rekonas la plenuman provon kaj haltigas la kernon; `call` neniam
/// revenas. Reveno signifus, ke la barilo mankas.
pub fn testi_smep(spaco: &mut X86AdresaSpaco, alokilo: &mut KadraAlokilo) {
    if let Err(eraro) = spaco.mapi_uzantan_regionon(
        VirtuaAdreso::nova(UZANTA_BAZO),
        KADRA_GRANDO as usize,
        PaĝaPermesoj::UZANTA_KODO,
        alokilo,
    ) {
        crate::skribu!("[memoro] SMEP-provo: mapado malsukcesis: {}\n", eraro);
        return;
    }
    let Some(fizika) = spaco.traduki(VirtuaAdreso::nova(UZANTA_BAZO)) else {
        crate::skribu!("[memoro] SMEP-provo: traduko mankas\n");
        return;
    };
    // SAFETY: la mapado liveris freŝan kadron, atingeblan per la identeca mapo;
    // ni posedas ĝin kaj skribas solan `ret` (0xC3).
    unsafe {
        core::ptr::write_volatile(fizika.adreso() as *mut u8, 0xC3);
    }

    // La mapo devas esti videbla en la aktiva `CR3`; interrompoj estas
    // malŝaltitaj en ĉi tiu punkto de la starto.
    spaco.ŝalti();

    let permesoj = spaco.permesoj(VirtuaAdreso::nova(UZANTA_BAZO));
    if !matches!(permesoj, Some(p) if p.uzanta && p.plenumebla) {
        crate::skribu!("[memoro] SMEP-provo: la folio ne estas U=1 RX\n");
        return;
    }
    crate::skribu!(
        "[memoro] SMEP-provo: Ring 0 provos plenumi la uzantan folion ĉe 0x{:x}\n",
        UZANTA_BAZO
    );

    // SAFETY: la celo estas intence uzanta, plenumebla folio; SMEP devas
    // rifuzi la plenuman provon per `#PF` antaŭ ol ekzekuti la unuan
    // instrukciaron. La alvokita kodo estas nia propra `ret`, do se SMEP
    // mankas, `call` revenas sen tuŝi la stakon krom la return-adreso.
    unsafe {
        asm!(
            "call {celo}",
            celo = in(reg) UZANTA_BAZO,
            clobber_abi("C"),
        );
    }
    crate::skribu!("[memoro] SMEP (MALBONE): Ring 0 plenumis uzantan paĝon\n");
}
