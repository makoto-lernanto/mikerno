//! Arkitekture komunaj abstraktaĵoj de la fasado.
//!
//! Ĉi tiuj tipoj permesas al la nukleo pritrakti kadrojn kaj firmigajn
//! raportojn sen koni la platforman aranĝon de la registroj.
//!
//! A1 ne ankoraŭ uzas ilin sur `arm64`; ili estos ligitaj en A3/A4.

#![cfg_attr(target_arch = "aarch64", allow(dead_code))]

/// Speco de procesora escepto, rekonita de la malantaŭo.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EsceptoTipo {
    PaĝaEraro,
    ProtektaEraro,
    Nekonata(u64),
}

/// Aliro al la konservita interrompa kadro sen koni la platformajn kampojn.
pub trait SistemvokaKadro {
    /// La sistemvoka numero aŭ, por aliaj esceptoj, la vektoro.
    fn numero(&self) -> usize;
    /// La `indekso`-a argumento (0–3).
    fn argumento(&self, indekso: usize) -> u64;
    /// Skribu la rezulton reen en la kadron.
    fn agordi_rezulton(&mut self, rezulto: u64);
    /// Ĉu la escepto venis el uzanta reĝimo?
    fn el_uzanto(&self) -> bool;
    /// La platforma vektoro aŭ interrompa identigilo.
    fn vektoro(&self) -> u64;
    /// La platforma erarkodo (aŭ `0` se la platformo ne havas unu).
    fn erarkodo(&self) -> u64;
    /// La adreso de la interrompita instrukcio.
    fn instrukcipunkto(&self) -> u64;
    /// La stakmontrilo de la interrompita kunteksto.
    fn stakmontrilo(&self) -> u64;
    /// La procesora stata registro (aŭ ĝia platforma ekvivalento).
    fn stato(&self) -> u64;
}

/// Unu firmiga trajto raportata ĉe la starto.
#[derive(Clone, Copy)]
pub struct FirmigaTrajto {
    pub nomo: &'static str,
    pub subtenata: bool,
    pub aktiva: bool,
    pub priskribo: &'static str,
}

impl FirmigaTrajto {
    const MALPLENA: Self = Self {
        nomo: "",
        subtenata: false,
        aktiva: false,
        priskribo: "",
    };
}

pub const MAKSAJ_FIRMIGAJ_TRAJTOJ: usize = 4;

/// La firmigaj trajtoj de la aktiva platformo.
pub struct FirmigaRaporto {
    trajtoj: [FirmigaTrajto; MAKSAJ_FIRMIGAJ_TRAJTOJ],
    nombro: usize,
}

impl FirmigaRaporto {
    pub const fn nova() -> Self {
        Self {
            trajtoj: [FirmigaTrajto::MALPLENA; MAKSAJ_FIRMIGAJ_TRAJTOJ],
            nombro: 0,
        }
    }

    pub fn aldonu(&mut self, trajto: FirmigaTrajto) {
        if self.nombro < self.trajtoj.len() {
            self.trajtoj[self.nombro] = trajto;
            self.nombro += 1;
        }
    }

    pub fn trajtoj(&self) -> &[FirmigaTrajto] {
        &self.trajtoj[..self.nombro]
    }
}
