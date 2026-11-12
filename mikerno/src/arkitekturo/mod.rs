//! Arkitektura fasado: la nukleo traktas la platformon nur per ĉi tiu modulo.

use core::fmt;

use crate::kerno::objekto::Kunteksto;
use crate::memoro::kadro::KadraAlokilo;
use crate::memoro::virtuala::{AdresaSpaco, PaĝaEraro};
use crate::memoro::{EniraEraro, EniraInformo};

#[cfg(target_arch = "x86_64")]
mod x86_64;

#[cfg(target_arch = "x86_64")]
use x86_64 as malantaŭo;

pub fn pravalorizi() {
    malantaŭo::pravalorizi();
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

/// Kreu freŝan adresan spacon por la nukleo.
pub fn krei_adresan_spacon(
    alokilo: &mut KadraAlokilo,
) -> Result<impl AdresaSpaco + use<>, PaĝaEraro> {
    <malantaŭo::pagxado::X86AdresaSpaco as AdresaSpaco>::krei(alokilo)
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

/// Pretigu la kernan stakon por transiroj Ring 3 → Ring 0.
pub fn inicialigi_kernostakon() {
    malantaŭo::tss::inicialigi_kernostakon();
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
