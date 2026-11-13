//! Kerna firmigo de la Ring 0/Ring 3 limo: `CR4.SMEP` kaj `CR4.SMAP`.
//!
//! SMEP malpermesas al Ring 0 plenumi foliojn kun la bito `U`. SMAP same
//! malpermesas legi aŭ skribi ilin, krom en mallonga fenestro ŝaltita per
//! `STAC` kaj fermita per `CLAC`. La subteno venas el `procesoro::trajtoj()`;
//! se la procesoro ne havas la koncernan biton, la kerno daŭras sen la barilo
//! kaj la memprovo estas preterlasita.

use core::arch::asm;

use super::pagxado::X86AdresaSpaco;
use super::procesoro;
use crate::memoro::KADRA_GRANDO;
use crate::memoro::kadro::KadraAlokilo;
use crate::memoro::virtuala::{AdresaSpaco, PaĝaPermesoj, PaĝaTablo, UZANTA_BAZO, VirtuaAdreso};

const SMEP_BITO: u64 = 1 << 20;
const SMAP_BITO: u64 = 1 << 21;
/// `RFLAGS.AC` (bito 18): la SMAP-permeso de Ring 0.
const FLAGO_AC: u64 = 1 << 18;

/// Ŝaltu `CR4.SMEP` kaj `CR4.SMAP` se la procesoro subtenas ilin.
pub fn pravalorizi() {
    let trajtoj = procesoro::trajtoj();
    let bitoj =
        (if trajtoj.smep { SMEP_BITO } else { 0 }) | (if trajtoj.smap { SMAP_BITO } else { 0 });
    if bitoj == 0 {
        return;
    }
    // SAFETY: nur la subtenataj firmigaj bitoj estas ŝaltataj; ĉiuj ceteraj
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
            in(reg) cr4 | bitoj,
            options(nomem, nostack, preserves_flags),
        );
    }
}

/// Ĉu `CR4.SMEP` estas efektive ŝaltita?
pub fn aktiva() -> bool {
    cr4_aktiva(SMEP_BITO)
}

/// Ĉu `CR4.SMAP` estas efektive ŝaltita?
pub fn smap_aktiva() -> bool {
    cr4_aktiva(SMAP_BITO)
}

fn cr4_aktiva(bito: u64) -> bool {
    let cr4: u64;
    // SAFETY: legado de `CR4` ne havas kromefikojn.
    unsafe {
        asm!(
            "mov {}, cr4",
            out(reg) cr4,
            options(nomem, nostack, preserves_flags),
        );
    }
    cr4 & bito != 0
}

/// Malfermu la SMAP-fenestron: Ring 0 rajtas legi kaj skribi uzantajn paĝojn
/// ĝis la sekva `clac`.
fn stac() {
    // SAFETY: `STAC` nur ŝaltas `RFLAGS.AC`. La solaj vokantoj estas
    // `StacGardo::nova`, kiu unue kontrolas `CR4.SMAP`; sen ĝi la instrukcio
    // kaŭzus `#UD`.
    unsafe { asm!("stac", options(nostack)) };
}

/// Fermu la SMAP-fenestron.
pub fn clac() {
    // SAFETY: `CLAC` nur malŝaltas `RFLAGS.AC`. Ankaŭ ĉi tie la vokantoj
    // certigas, ke `CR4.SMAP` estas ŝaltita.
    unsafe { asm!("clac", options(nostack)) };
}

fn ac_aktiva() -> bool {
    let flagoj: u64;
    // SAFETY: `pushfq`/`pop` nur legas `RFLAGS`.
    unsafe {
        asm!(
            "pushfq",
            "pop {}",
            out(reg) flagoj,
            options(preserves_flags),
        );
    }
    flagoj & FLAGO_AC != 0
}

/// RAII-gardo por mallonga aliro al uzantaj paĝoj sub SMAP.
///
/// `nova` malfermas la fenestron nur se `CR4.SMAP` estas ŝaltita kaj `AC`
/// ankoraŭ estas malŝaltita; `Drop` restarigas la antaŭan staton. Nestitaj
/// gardoj do ne fermas fenestron, kiun malfermis la ekstera gardo.
pub struct StacGardo {
    agordis: bool,
}

impl StacGardo {
    pub fn nova() -> Self {
        if !smap_aktiva() || ac_aktiva() {
            return Self { agordis: false };
        }
        stac();
        Self { agordis: true }
    }
}

impl Drop for StacGardo {
    fn drop(&mut self) {
        if self.agordis {
            clac();
        }
    }
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

/// Pruvu, ke Ring 0 ne povas legi uzantan datumfolion sen `stac`.
///
/// La funkcio mapas unu `U=1`-datumfolion en la aktivan kernan spacon kaj legas
/// ĝin per `read_volatile` sen malfermi `StacGardo`. Se SMAP funkcias, la `#PF`-
/// traktilo rekonas la legadon (Ring 0, `I=0`, `AC=0`, uzanta `CR2`) kaj
/// haltigas la kernon; la legado neniam revenas. Reveno signifus, ke la barilo
/// mankas.
pub fn testi_smap(spaco: &mut X86AdresaSpaco, alokilo: &mut KadraAlokilo) {
    if let Err(eraro) = spaco.mapi_uzantan_regionon(
        VirtuaAdreso::nova(UZANTA_BAZO),
        KADRA_GRANDO as usize,
        PaĝaPermesoj::UZANTA_DATUMOJ,
        alokilo,
    ) {
        crate::skribu!("[memoro] SMAP-provo: mapado malsukcesis: {}\n", eraro);
        return;
    }
    let Some(fizika) = spaco.traduki(VirtuaAdreso::nova(UZANTA_BAZO)) else {
        crate::skribu!("[memoro] SMAP-provo: traduko mankas\n");
        return;
    };
    // SAFETY: la mapado liveris freŝan kadron, atingeblan per la identeca mapo;
    // ni posedas ĝin kaj skribas unu konatan bajton por la revena kazo.
    unsafe {
        core::ptr::write_volatile(fizika.adreso() as *mut u8, 0xA5);
    }

    // La mapo devas esti videbla en la aktiva `CR3`; interrompoj estas
    // malŝaltitaj en ĉi tiu punkto de la starto.
    spaco.ŝalti();

    let permesoj = spaco.permesoj(VirtuaAdreso::nova(UZANTA_BAZO));
    if !matches!(permesoj, Some(p) if p.uzanta && p.skribebla && !p.plenumebla) {
        crate::skribu!("[memoro] SMAP-provo: la folio ne estas U=1 RW NX\n");
        return;
    }
    // `clac` estas leĝa ĉar la alvokanto kontrolis `CR4.SMAP`; ĝi certigas, ke
    // eventuala `AC` de la praŝarĝa ŝelko ne malfermas la fenestron.
    clac();
    crate::skribu!(
        "[memoro] SMAP-provo: Ring 0 provos legi la uzantan folion ĉe 0x{:x}\n",
        UZANTA_BAZO
    );

    // SAFETY: la celo estas intence uzanta, nur-Ring-3 datumfolio; sen `STAC`
    // SMAP devas rifuzi la legadon per `#PF` antaŭ ol la valoro alvenas. La
    // `read_volatile` malhelpas al la kompililo forigi la provon.
    let valoro = unsafe { core::ptr::read_volatile(UZANTA_BAZO as *const u8) };
    crate::skribu!(
        "[memoro] SMAP (MALBONE): Ring 0 legis uzantan paĝon (valoro 0x{:x})\n",
        valoro
    );
}
