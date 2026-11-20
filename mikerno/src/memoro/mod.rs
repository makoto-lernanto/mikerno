//! Arkitekture sendependaj priskriboj de fizika memoro kaj praŝarĝaj informoj.

pub mod kadro;
pub mod virtuala;

#[cfg(target_arch = "x86_64")]
use core::fmt;

pub use kernologiko::memoro::{
    KADRA_GRANDO, MAKSAJ_MODULOJ, MemoraMapo, MemoraRegiono, Modulo, RegionaTipo,
};

/// Arkitekture sendependa praŝarĝa handoff.
pub struct EniraInformo {
    pub memora_mapo: &'static MemoraMapo,
    /// Regionoj de praŝarĝaj strukturoj, kiuj devas resti rezervitaj.
    pub rezervindaj: &'static [MemoraRegiono],
    /// Frambufo liverita de la praŝarĝilo; `arm64` ricevas `None` kaj la
    /// plata malantaŭo kreas sian propran (`ramfb`).
    pub frambufo: Option<crate::ekrano::FrambufaInformo>,
    /// Moduloj liveritaj de la praŝarĝilo, en la ordo de la protokolo.
    pub moduloj: &'static [Modulo],
}

#[cfg(target_arch = "x86_64")]
#[derive(Debug, Clone, Copy)]
pub enum EniraEraro {
    NevalidaStarto,
    NevalidaMagio,
    NesubtenataVersio,
    NeniuMemoraMapo,
    TroMultajRegionoj,
    NevalidaModulo,
    TroMultajModuloj,
}

#[cfg(target_arch = "x86_64")]
impl fmt::Display for EniraEraro {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::NevalidaStarto => "nevalida praŝarĝa montrilo",
            Self::NevalidaMagio => "nevalida praŝarĝa magio",
            Self::NesubtenataVersio => "nesubtenata PVH-versio",
            Self::NeniuMemoraMapo => "mankas memora mapo",
            Self::TroMultajRegionoj => "tro da regionoj en la memora mapo",
            Self::NevalidaModulo => "nevalida praŝarĝa modulo",
            Self::TroMultajModuloj => "tro da praŝarĝaj moduloj",
        })
    }
}
