//! Arkitektura fasado: la nukleo traktas la platformon nur per ĉi tiu modulo.

use core::fmt;

use crate::memoro::kadro::KadraAlokilo;
use crate::memoro::virtuala::{AdresaSpaco, FizikaAdreso, PaĝaEraro};
use crate::memoro::{EniraEraro, EniraInformo};

pub mod komuna;

pub use komuna::{EsceptoTipo, FirmigaRaporto, SistemvokaKadro};

#[cfg(target_arch = "x86_64")]
mod x86_64;

#[cfg(target_arch = "x86_64")]
use x86_64 as malantaŭo;

pub use malantaŭo::interrompo::{InterrompaKadro, escepto_speco};
pub use malantaŭo::planado::Kunteksto;
pub use malantaŭo::uzanta::ELF_MAŜINO;

pub fn pravalorizi() {
    malantaŭo::pravalorizi();
}

/// Unua kerna enirpunkto: la KASLR-ŝelko kopias la bildon al hazarda bazo kaj
/// saltas tien; se tio ne eblas, la funkcio daŭrigas ĉe la liga bazo. Neniam
/// revenas.
pub fn hazardigi(starta_informo: *const u8, magio: u32) -> ! {
    malantaŭo::hazardigo::hazardigi(starta_informo, magio)
}

/// Hazarda valoro por la KASLR-bazo (aparata `RDRAND` aŭ miksita `RDTSC`).
pub fn hazardo() -> u64 {
    malantaŭo::procesoro::hazardo()
}

pub use malantaŭo::firmigo::{
    StacGardo, ac_aktiva, aktiva as smep_aktiva, clac, smap_aktiva, testi_ac, testi_smap,
    testi_smep,
};

/// La firmigaj trajtoj faritaj de la platformo, por la starta raporto.
pub fn firmiga_raporto() -> FirmigaRaporto {
    malantaŭo::firmigo::raporto()
}

/// Malŝaltu la uzantaliran fenestron de Ring 0 se la platformo havas unu.
///
/// Vokata de la interrompa dispetĉo: la uzanto povas lasi `RFLAGS.AC=1`, kaj
/// la kerno devas fermi la fenestron antaŭ ol tuŝi uzantajn montrilojn.
pub fn malŝalti_ac() {
    if smap_aktiva() {
        clac();
    }
}

pub fn halti() -> ! {
    malantaŭo::procesoro::halti()
}

/// Ne enlinie: la diagnozaj vokoj estas multaj, kaj unu kunhavata kopio de la
/// formatiga vojo tenas `.text` malgranda.
#[inline(never)]
pub fn skribi_formaton(argumentoj: fmt::Arguments) {
    let _gardo = crate::interrompo::InterrompaGardo::nova();
    malantaŭo::seria::skribi_formaton(argumentoj);
    crate::ekrano::skribi_formaton(argumentoj);
}

pub fn analizi_eniron(starta_informo: *const u8, magio: u32) -> Result<EniraInformo, EniraEraro> {
    if magio == malantaŭo::mb2::MAGIO {
        malantaŭo::mb2::analizi(starta_informo)
    } else {
        malantaŭo::pvh::analizi(starta_informo)
    }
}

/// Konkreta adresa spaco de la aktiva platformo. La nukleo bezonas nomon por
/// konservi unu spacon por ĉiu procezo; la cetera kodo uzas la trait-on.
pub type Spaco = malantaŭo::pagxado::X86AdresaSpaco;

/// Kreu freŝan, malplenan adresan spacon por la nukleo mem.
pub fn krei_adresan_spacon(alokilo: &mut KadraAlokilo) -> Result<Spaco, PaĝaEraro> {
    <Spaco as AdresaSpaco>::krei(alokilo)
}

pub use malantaŭo::procesoro::{
    ebligi_interrompojn, interrompoj_ebligitaj, malebligi_interrompojn,
};

pub fn atendi_interrompon() {
    malantaŭo::procesoro::atendi_interrompon();
}

/// La adreso, kiu kaŭzis la lastan paĝan eraron (`CR2`).
pub fn fuŝa_adreso() -> u64 {
    malantaŭo::procesoro::fuŝa_adreso()
}

/// Ĉu `CR4.PCIDE` estas malŝaltita? La `CR3`-vojo de MiKerno postulas tion.
pub fn ne_pcide() -> bool {
    malantaŭo::procesoro::ne_pcide()
}

pub fn konfirmi_interrompon() {
    malantaŭo::apic::konfirmi()
}

/// Kiom da aparataj `EOI`-oj estis senditaj ekde la starto?
pub fn konfirmoj_interrompoj() -> u64 {
    malantaŭo::apic::konfirmoj()
}

pub fn pravalorizi_apicon(
    spaco: &mut dyn AdresaSpaco,
    alokilo: &mut KadraAlokilo,
) -> Result<u32, PaĝaEraro> {
    malantaŭo::apic::pravalorizi(spaco, alokilo)
}

pub fn pravalorizi_temporizilon() {
    malantaŭo::temporizilo::pravalorizi();
}

/// La fizika radiko, kiun `CR3` nun montras.
pub fn aktiva_adreso_spaco() -> u64 {
    malantaŭo::pagxado::aktiva_radiko().adreso()
}

/// Ŝaltu `CR3` al la donita radika paĝtabelo.
///
/// # Sekureco
/// Vidu `pagxado::ŝalti_radikon`.
pub unsafe fn ŝalti_adresan_spacon(radiko: u64) {
    unsafe { malantaŭo::pagxado::ŝalti_radikon(FizikaAdreso::nova(radiko)) }
}

/// Pretigu la kernan stakon por transiroj Ring 3 → Ring 0.
pub fn inicialigi_kernostakon() {
    malantaŭo::tss::inicialigi_kernostakon();
}

/// Agordu `TSS.RSP0` al la kerna stako de la nuna procezo.
///
/// `pinto` devas esti 16-bajte aliniiĝinta kaj mapita en la kerna identeca
/// mapo; la CPU uzos ĝin ĉe la sekva transiro Ring 3 → Ring 0.
pub fn agordi_kernostakon(pinto: u64) {
    malantaŭo::tss::agordi_kernostakon(pinto);
}

/// La defaŭlta kerna stakpinto de la nukleo (procezo `0`).
pub fn defaŭlta_kernostako() -> u64 {
    malantaŭo::tss::defaŭlta_pinto()
}

/// Transiru al la uzanta reĝimo; la voko neniam revenas.
///
/// `reĝimo` kaj `identigilo` estas transdonataj kiel la unuaj du argumentoj
/// de la enirpunkto.
///
/// # Sekureco
/// La enirpunkto kaj la stakpinto devas aparteni al mapita uzanta bildo kaj
/// stako (vidu `uzanta::eniri_uzanton`).
pub unsafe fn eniri_uzanton(enirpunkto: u64, stakpinto: u64, reĝimo: u64, identigilo: u64) -> ! {
    unsafe { malantaŭo::uzanta::eniri_uzanton(enirpunkto, stakpinto, reĝimo, identigilo) }
}

pub fn temporizilaj_tikoj() -> u64 {
    malantaŭo::temporizilo::tikoj()
}

/// Instalu la platforman enirmekanismon de sistemvokoj kaj ĝian traktilon.
pub fn instali_sistemvokon() {
    malantaŭo::sistemvoko::instali();
}

pub fn prepari_kuntekston(konteksto: &mut Kunteksto, stako_pinto: u64) {
    malantaŭo::planado::prepari_kuntekston(konteksto, stako_pinto);
}

/// # Sekureco
/// Vidu `planado::ŝalti_kuntekston`.
pub unsafe fn ŝalti_kuntekston(malnova: *mut Kunteksto, nova: *const Kunteksto) {
    unsafe { malantaŭo::planado::ŝalti_kuntekston(malnova, nova) }
}
