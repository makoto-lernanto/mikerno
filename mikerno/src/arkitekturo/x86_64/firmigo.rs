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
use crate::arkitekturo::komuna::{FirmigaRaporto, FirmigaTrajto};
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
        asm!(
            "mov cr4, {}",
            in(reg) cr4() | bitoj,
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

/// La firmigaj trajtoj de la procesoro por la starta raporto.
pub fn raporto() -> FirmigaRaporto {
    let trajtoj = procesoro::trajtoj();
    let mut raporto = FirmigaRaporto::nova();
    raporto.aldonu(FirmigaTrajto {
        nomo: "SMEP",
        subtenata: trajtoj.smep,
        aktiva: aktiva(),
        priskribo: "Ring 0 ne povas plenumi uzantajn paĝojn.",
    });
    raporto.aldonu(FirmigaTrajto {
        nomo: "SMAP",
        subtenata: trajtoj.smap,
        aktiva: smap_aktiva(),
        priskribo: "Ring 0 legas uzantajn bufrojn nur en STAC-fenestroj.",
    });
    raporto
}

fn cr4() -> u64 {
    let cr4: u64;
    // SAFETY: legado de `CR4` ne havas kromefikojn.
    unsafe {
        asm!(
            "mov {}, cr4",
            out(reg) cr4,
            options(nomem, nostack, preserves_flags),
        );
    }
    cr4
}

fn cr4_aktiva(bito: u64) -> bool {
    cr4() & bito != 0
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

/// Ĉu `RFLAGS.AC` estas ŝaltita? Uzata de la `STAC`-gardo kaj de la aserto en
/// la interrompa dispetĉo.
pub fn ac_aktiva() -> bool {
    procesoro::flagoj() & FLAGO_AC != 0
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

/// Pruvu la disciplinon de la `STAC`-fenestro.
///
/// La provo kontrolas kvar aferojn: la fenestro estas fermita antaŭ ol la unua
/// gardo, la gardo malfermas ĝin, nestita gardo ne fermas la eksteran fenestron
/// ĉe sia `Drop`, kaj la lasta gardo restarigas la antaŭan staton. Fine ĝi
/// ŝaltas `AC` sen gardo — kvazaŭ Ring 3 havus ĝin — kaj pruvas, ke
/// `malŝalti_ac` forigas ankaŭ tian fenestron.
pub fn testi_ac() {
    if !smap_aktiva() {
        crate::skribu!("[memoro] AC-provo preterlasita: SMAP ne estas ŝaltita\n");
        return;
    }

    let fermita = !ac_aktiva();
    let ekstera = StacGardo::nova();
    let malfermita = ac_aktiva();
    let interna = StacGardo::nova();
    let ankoraŭ_malfermita = ac_aktiva();
    drop(interna);
    let post_interna = ac_aktiva();
    drop(ekstera);
    let fermita_post = !ac_aktiva();

    stac();
    let ŝtelita = ac_aktiva();
    crate::arkitekturo::malŝalti_ac();
    let purigita = !ac_aktiva();

    let bone = fermita
        && malfermita
        && ankoraŭ_malfermita
        && post_interna
        && fermita_post
        && ŝtelita
        && purigita;
    crate::skribu!(
        "[memoro] AC ({}): fermita={}, malfermita={}, interna={}, post_interna={}, fermita_post={}, ŝtelita={}, purigita={}\n",
        if bone { "bone" } else { "MALBONE" },
        fermita,
        malfermita,
        ankoraŭ_malfermita,
        post_interna,
        fermita_post,
        ŝtelita,
        purigita
    );
}

/// Mapu unu `U=1`-provfolion ĉe `UZANTA_BAZO`, skribu `valoro` en ĝin kaj
/// ŝaltu la spacon. `false` se la mapado aŭ la traduko malsukcesas.
fn prepari_provan_folion(
    nomo: &str,
    spaco: &mut X86AdresaSpaco,
    alokilo: &mut KadraAlokilo,
    permesoj: PaĝaPermesoj,
    valoro: u8,
) -> bool {
    if let Err(eraro) = spaco.mapi_uzantan_regionon(
        VirtuaAdreso::nova(UZANTA_BAZO),
        KADRA_GRANDO as usize,
        permesoj,
        alokilo,
    ) {
        crate::skribu!("[memoro] {nomo}-provo: mapado malsukcesis: {eraro}\n");
        return false;
    }
    let Some(fizika) = spaco.traduki(VirtuaAdreso::nova(UZANTA_BAZO)) else {
        crate::skribu!("[memoro] {nomo}-provo: traduko mankas\n");
        return false;
    };
    // SAFETY: la mapado liveris freŝan kadron, atingeblan per la identeca mapo;
    // ni posedas ĝin kaj skribas unu kontrolitan bajton.
    unsafe {
        core::ptr::write_volatile(fizika.adreso() as *mut u8, valoro);
    }
    // La mapo devas esti videbla en la aktiva `CR3`; interrompoj estas
    // malŝaltitaj en ĉi tiu punkto de la starto.
    spaco.ŝalti();
    true
}

/// Pruvu, ke Ring 0 ne povas plenumi uzantan folion.
///
/// La funkcio mapas unu `U=1`-kodon en la aktivan kernan spacon, skribas solan
/// `ret`-instrukciaron kaj vokas ĝin per `call`. Se SMEP funkcias, la `#PF`-
/// traktilo rekonas la plenuman provon kaj haltigas la kernon; `call` neniam
/// revenas. Reveno signifus, ke la barilo mankas.
pub fn testi_smep(spaco: &mut X86AdresaSpaco, alokilo: &mut KadraAlokilo) {
    if !prepari_provan_folion("SMEP", spaco, alokilo, PaĝaPermesoj::UZANTA_KODO, 0xC3) {
        return;
    }

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
    if !prepari_provan_folion("SMAP", spaco, alokilo, PaĝaPermesoj::UZANTA_DATUMOJ, 0xA5) {
        return;
    }

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
