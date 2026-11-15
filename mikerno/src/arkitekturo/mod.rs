//! Arkitektura fasado: la nukleo traktas la platformon nur per ĉi tiu modulo.

use core::fmt;

pub mod komuna;

#[cfg_attr(target_arch = "aarch64", allow(unused_imports))]
pub use komuna::{EsceptoTipo, FirmigaRaporto, SistemvokaKadro};

#[cfg(target_arch = "x86_64")]
mod x86_64;
#[cfg(target_arch = "x86_64")]
use x86_64 as malantaŭo;

#[cfg(target_arch = "aarch64")]
pub mod arm64;

#[cfg(target_arch = "x86_64")]
use crate::memoro::kadro::KadraAlokilo;
#[cfg(target_arch = "x86_64")]
use crate::memoro::virtuala::{AdresaSpaco, FizikaAdreso, PaĝaEraro};
#[cfg(target_arch = "x86_64")]
use crate::memoro::{EniraEraro, EniraInformo};

#[cfg(target_arch = "aarch64")]
pub use arm64::interrompo::InterrompaKadro;
#[cfg(target_arch = "aarch64")]
pub use arm64::planado::Kunteksto;
#[cfg(target_arch = "aarch64")]
pub use arm64::uzanta::ELF_MAŜINO;
#[cfg(target_arch = "aarch64")]
pub use arm64::{StacGardo, pan_aktiva, testi_pan};
#[cfg(target_arch = "x86_64")]
pub use malantaŭo::interrompo::{InterrompaKadro, escepto_speco};
#[cfg(target_arch = "x86_64")]
pub use malantaŭo::planado::Kunteksto;
#[cfg(target_arch = "x86_64")]
pub use malantaŭo::uzanta::ELF_MAŜINO;

/// Skribu formatitan tekston al la seria konzolo kaj al la frambufferio.
#[cfg(target_arch = "aarch64")]
pub fn skribi_formaton(argumentoj: fmt::Arguments) {
    arm64::seria::skribi_formaton(argumentoj);
    crate::ekrano::skribi_formaton(argumentoj);
}

/// Skribu formatitan tekston al la seria konzolo kaj al la frambuffer.
#[cfg(target_arch = "x86_64")]
pub fn skribi_formaton(argumentoj: fmt::Arguments) {
    let _gardo = crate::interrompo::InterrompaGardo::nova();
    malantaŭo::seria::skribi_formaton(argumentoj);
    crate::ekrano::skribi_formaton(argumentoj);
}

/// Haltigu la maŝinon.
#[cfg(target_arch = "aarch64")]
pub fn halti() -> ! {
    arm64::procesoro::halti()
}

#[cfg(target_arch = "x86_64")]
pub fn halti() -> ! {
    malantaŭo::procesoro::halti()
}

#[cfg(target_arch = "x86_64")]
pub fn pravalorizi() {
    malantaŭo::pravalorizi();
}

/// Unua kerna enirpunkto: la KASLR-ŝelko kopias la bildon al hazarda bazo kaj
/// saltas tien; se tio ne eblas, la funkcio daŭrigas ĉe la liga bazo. Neniam
/// revenas.
#[cfg(target_arch = "x86_64")]
pub fn hazardigi(starta_informo: *const u8, magio: u32) -> ! {
    malantaŭo::hazardigo::hazardigi(starta_informo, magio)
}

/// Hazarda valoro por la KASLR-bazo (aparata `RDRAND` aŭ miksita `RDTSC`).
#[cfg(target_arch = "x86_64")]
pub fn hazardo() -> u64 {
    malantaŭo::procesoro::hazardo()
}

#[cfg(target_arch = "x86_64")]
pub use malantaŭo::firmigo::{
    StacGardo, aktiva as smep_aktiva, clac, smap_aktiva, testi_smap, testi_smep,
};

/// Kreu frambufferion sur platformo, kiu ne ricevas unu de la praŝarĝilo.
///
/// `x86_64` ricevas la frambufferion per Multiboot2, do ĉi tiu funkcio nur
/// redonas `None`; `arm64` anoncas RAM-bufron al `ramfb` per `fw_cfg`.
#[cfg(target_arch = "x86_64")]
pub fn krei_frambufferon(_alokilo: &mut KadraAlokilo) -> Option<crate::ekrano::FramebufferInformo> {
    None
}

#[cfg(target_arch = "aarch64")]
pub fn krei_frambufferon(
    alokilo: &mut crate::memoro::kadro::KadraAlokilo,
) -> Option<crate::ekrano::FramebufferInformo> {
    arm64::fw_cfg::krei_frambufferon(alokilo)
}

/// La firmigaj trajtoj faritaj de la platformo, por la starta raporto.
#[cfg(target_arch = "x86_64")]
pub fn firmiga_raporto() -> FirmigaRaporto {
    malantaŭo::firmigo::raporto()
}

#[cfg(target_arch = "aarch64")]
pub fn firmiga_raporto() -> FirmigaRaporto {
    arm64::firmiga_raporto()
}

/// Malŝaltu la uzantaliran fenestron de Ring 0 se la platformo havas unu.
///
/// Vokata de la interrompa dispetĉo: la uzanto povas lasi `RFLAGS.AC=1`, kaj
/// la kerno devas fermi la fenestron antaŭ ol tuŝi uzantajn montrilojn.
#[cfg(target_arch = "x86_64")]
pub fn malŝalti_ac() {
    if smap_aktiva() {
        clac();
    }
}

#[cfg(target_arch = "x86_64")]
pub fn analizi_eniron(starta_informo: *const u8, magio: u32) -> Result<EniraInformo, EniraEraro> {
    if magio == malantaŭo::mb2::MAGIO {
        malantaŭo::mb2::analizi(starta_informo)
    } else {
        malantaŭo::pvh::analizi(starta_informo)
    }
}

/// Konkreta adresa spaco de la aktiva platformo. La nukleo bezonas nomon por
/// konservi unu spacon por ĉiu procezo; la cetera kodo uzas la trait-on.
#[cfg(target_arch = "x86_64")]
pub type Spaco = malantaŭo::pagxado::X86AdresaSpaco;
#[cfg(target_arch = "aarch64")]
pub type Spaco = arm64::pagxado::ArmAdresaSpaco;

/// Kreu freŝan, malplenan adresan spacon por la nukleo mem.
#[cfg(target_arch = "x86_64")]
pub fn krei_adresan_spacon(alokilo: &mut KadraAlokilo) -> Result<Spaco, PaĝaEraro> {
    <Spaco as AdresaSpaco>::krei(alokilo)
}

#[cfg(target_arch = "aarch64")]
pub fn krei_adresan_spacon(
    alokilo: &mut crate::memoro::kadro::KadraAlokilo,
) -> Result<Spaco, crate::memoro::virtuala::PaĝaEraro> {
    <Spaco as crate::memoro::virtuala::AdresaSpaco>::krei(alokilo)
}

#[cfg(target_arch = "aarch64")]
pub use arm64::procesoro::{ebligi_interrompojn, interrompoj_ebligitaj, malebligi_interrompojn};
#[cfg(target_arch = "x86_64")]
pub use malantaŭo::procesoro::{
    ebligi_interrompojn, interrompoj_ebligitaj, malebligi_interrompojn,
};

#[cfg(target_arch = "x86_64")]
pub fn atendi_interrompon() {
    malantaŭo::procesoro::atendi_interrompon();
}
#[cfg(target_arch = "aarch64")]
pub fn atendi_interrompon() {
    arm64::procesoro::atendi_interrompon();
}

/// La adreso, kiu kaŭzis la lastan paĝan eraron (`CR2`).
#[cfg(target_arch = "x86_64")]
pub fn fuŝa_adreso() -> u64 {
    malantaŭo::procesoro::fuŝa_adreso()
}

#[cfg(target_arch = "x86_64")]
pub fn konfirmi_interrompon() {
    malantaŭo::apic::konfirmi()
}

#[cfg(target_arch = "x86_64")]
pub fn pravalorizi_apicon(
    spaco: &mut dyn AdresaSpaco,
    alokilo: &mut KadraAlokilo,
) -> Result<u32, PaĝaEraro> {
    malantaŭo::apic::pravalorizi(spaco, alokilo)
}

#[cfg(target_arch = "x86_64")]
pub fn pravalorizi_temporizilon() {
    malantaŭo::temporizilo::pravalorizi();
}

#[cfg(target_arch = "aarch64")]
pub fn pravalorizi_temporizilon() {
    arm64::temporizilo::pravalorizi();
}

/// La fizika radiko, kiun `CR3` (aŭ `TTBR0_EL1`) nun montras.
#[cfg(target_arch = "x86_64")]
pub fn aktiva_adreso_spaco() -> u64 {
    malantaŭo::pagxado::aktiva_radiko().adreso()
}

#[cfg(target_arch = "aarch64")]
pub fn aktiva_adreso_spaco() -> u64 {
    arm64::pagxado::aktiva_radiko()
}

/// Ŝaltu `CR3` (aŭ `TTBR0_EL1`) al la donita radika paĝtabelo.
///
/// # Sekureco
/// Vidu `pagxado::ŝalti_radikon`.
#[cfg(target_arch = "x86_64")]
pub unsafe fn ŝalti_adresan_spacon(radiko: u64) {
    unsafe { malantaŭo::pagxado::ŝalti_radikon(FizikaAdreso::nova(radiko)) }
}

#[cfg(target_arch = "aarch64")]
pub unsafe fn ŝalti_adresan_spacon(radiko: u64) {
    // SAFETY: la alvokanto garantias vivan radikon kaj interrompojn malŝaltitaj.
    unsafe { arm64::pagxado::ŝalti_radikon(radiko) }
}

/// Pretigu la kernan stakon por transiroj Ring 3 → Ring 0.
///
/// Sur `arm64` la procesoro mem ŝaltas al `SP_EL1` ĉe escepto el `EL0`; la
/// kerna stako venas el la planista kunteksto, do nenio agordendas.
#[cfg(target_arch = "x86_64")]
pub fn inicialigi_kernostakon() {
    malantaŭo::tss::inicialigi_kernostakon();
}

#[cfg(target_arch = "aarch64")]
pub fn inicialigi_kernostakon() {}

/// Agordu `TSS.RSP0` al la kerna stako de la nuna procezo.
///
/// `pinto` devas esti 16-bajte aliniiĝinta kaj mapita en la kerna identeca
/// mapo; la CPU uzos ĝin ĉe la sekva transiro Ring 3 → Ring 0.
#[cfg(target_arch = "x86_64")]
pub fn agordi_kernostakon(pinto: u64) {
    malantaŭo::tss::agordi_kernostakon(pinto);
}

/// La defaŭlta kerna stakpinto de la nukleo (procezo `0`).
#[cfg(target_arch = "x86_64")]
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
#[cfg(target_arch = "x86_64")]
pub unsafe fn eniri_uzanton(enirpunkto: u64, stakpinto: u64, reĝimo: u64, identigilo: u64) -> ! {
    unsafe { malantaŭo::uzanta::eniri_uzanton(enirpunkto, stakpinto, reĝimo, identigilo) }
}

/// Transiru al `EL0`; la voko neniam revenas.
///
/// # Sekureco
/// Vidu `arm64::uzanta::eniri_uzanton`.
#[cfg(target_arch = "aarch64")]
pub unsafe fn eniri_uzanton(enirpunkto: u64, stakpinto: u64, reĝimo: u64, identigilo: u64) -> ! {
    unsafe { arm64::uzanta::eniri_uzanton(enirpunkto, stakpinto, reĝimo, identigilo) }
}

/// Voku la platforman sistemvokon el Ring 0/EL1.
#[cfg(target_arch = "aarch64")]
pub fn invoki_sistemvokon(numero: usize, argumentoj: [u64; 4]) -> u64 {
    arm64::sistemvoko::invoki(numero, argumentoj)
}

#[cfg(target_arch = "x86_64")]
pub fn temporizilaj_tikoj() -> u64 {
    malantaŭo::temporizilo::tikoj()
}
#[cfg(target_arch = "aarch64")]
pub fn temporizilaj_tikoj() -> u64 {
    arm64::temporizilo::tikoj()
}

/// Instalu la platforman enirmekanismon de sistemvokoj kaj ĝian traktilon.
#[cfg(target_arch = "x86_64")]
pub fn instali_sistemvokon() {
    malantaŭo::sistemvoko::instali();
}

#[cfg(target_arch = "aarch64")]
pub fn instali_sistemvokon() {
    arm64::instali_sistemvokon();
}

#[cfg(target_arch = "x86_64")]
pub fn prepari_kuntekston(konteksto: &mut Kunteksto, stako_pinto: u64) {
    malantaŭo::planado::prepari_kuntekston(konteksto, stako_pinto);
}
#[cfg(target_arch = "aarch64")]
pub fn prepari_kuntekston(konteksto: &mut Kunteksto, stako_pinto: u64) {
    arm64::planado::prepari_kuntekston(konteksto, stako_pinto);
}

/// # Sekureco
/// Vidu `planado::ŝalti_kuntekston`.
#[cfg(target_arch = "x86_64")]
pub unsafe fn ŝalti_kuntekston(malnova: *mut Kunteksto, nova: *const Kunteksto) {
    unsafe { malantaŭo::planado::ŝalti_kuntekston(malnova, nova) }
}
/// # Sekureco
/// Vidu `planado::ŝalti_kuntekston`.
#[cfg(target_arch = "aarch64")]
pub unsafe fn ŝalti_kuntekston(malnova: *mut Kunteksto, nova: *const Kunteksto) {
    unsafe { arm64::planado::ŝalti_kuntekston(malnova, nova) }
}
