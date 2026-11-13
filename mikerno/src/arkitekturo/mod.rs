//! Arkitektura fasado: la nukleo traktas la platformon nur per ĉi tiu modulo.

use core::fmt;

use crate::kerno::objekto::Kunteksto;
use crate::memoro::kadro::KadraAlokilo;
use crate::memoro::virtuala::{AdresaSpaco, FizikaAdreso, PaĝaEraro};
use crate::memoro::{EniraEraro, EniraInformo};

#[cfg(target_arch = "x86_64")]
mod x86_64;

#[cfg(target_arch = "x86_64")]
use x86_64 as malantaŭo;

pub fn pravalorizi() {
    malantaŭo::pravalorizi();
}

pub use malantaŭo::firmigo::{aktiva as smep_aktiva, testi_smep};
pub use malantaŭo::procesoro::Trajtoj as ProcesorajTrajtoj;

/// Legu la procesorajn trajtojn uzatajn de la kerna firmigo.
pub fn procesoraj_trajtoj() -> ProcesorajTrajtoj {
    malantaŭo::procesoro::trajtoj()
}

pub fn halti() -> ! {
    malantaŭo::procesoro::halti()
}

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

pub fn konfirmi_interrompon() {
    malantaŭo::apic::konfirmi()
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

/// Transiru al Ring 3 per `iretq`; la voko neniam revenas.
///
/// `reĝimo` kaj `identigilo` estas transdonataj en `RDI` kaj `RSI` kiel la
/// unuaj du argumentoj de la enirpunkto.
///
/// # Sekureco
/// La enirpunkto kaj la stakpinto devas aparteni al mapita uzanta bildo kaj
/// stako (vidu `uzanta::transiri_al_ring3`).
pub unsafe fn transiri_al_ring3(
    enirpunkto: u64,
    stakpinto: u64,
    reĝimo: u64,
    identigilo: u64,
) -> ! {
    unsafe { malantaŭo::uzanta::transiri_al_ring3(enirpunkto, stakpinto, reĝimo, identigilo) }
}

pub fn temporizilaj_tikoj() -> u64 {
    malantaŭo::temporizilo::tikoj()
}

/// Dispetĉu sistemvokon el la interrompa kadro.
pub fn sistemvoka_dispetĉo(kadro: &mut crate::interrompo::InterrompaKadro) {
    malantaŭo::sistemvoko::sistemvoka_dispetĉo(kadro);
}

pub fn prepari_kuntekston(konteksto: &mut Kunteksto, stako_pinto: u64) {
    malantaŭo::planado::prepari_kuntekston(konteksto, stako_pinto);
}

/// # Sekureco
/// Vidu `planado::ŝalti_kuntekston`.
pub unsafe fn ŝalti_kuntekston(malnova: *mut Kunteksto, nova: *const Kunteksto) {
    unsafe { malantaŭo::planado::ŝalti_kuntekston(malnova, nova) }
}
