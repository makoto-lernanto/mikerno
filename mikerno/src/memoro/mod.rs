//! Arkitekture sendependaj priskriboj de fizika memoro kaj praŝarĝaj informoj.

pub mod kadro;
pub mod virtuala;

use core::fmt;

pub const KADRA_GRANDO: u64 = 4096;
pub const MAKSAJ_REGIONOJ: usize = 128;
pub const MAKSAJ_MODULOJ: usize = 8;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RegionaTipo {
    // La unua varianto estas la implicita nulo, do `MALPLENA` restas nula.
    Nekonata,
    Uzebla,
    Rezervita,
    Acpi,
    Nvs,
    Neuzebla,
    Malŝaltita,
    Pmem,
}

#[derive(Clone, Copy)]
pub struct MemoraRegiono {
    pub komenco: u64,
    pub longo: u64,
    pub tipo: RegionaTipo,
}

impl MemoraRegiono {
    pub const MALPLENA: Self = Self {
        komenco: 0,
        longo: 0,
        tipo: RegionaTipo::Nekonata,
    };

    pub const fn fino(&self) -> u64 {
        self.komenco + self.longo
    }

    pub const fn uzebla(&self) -> bool {
        matches!(self.tipo, RegionaTipo::Uzebla)
    }
}

pub struct MemoraMapo {
    regionoj: [MemoraRegiono; MAKSAJ_REGIONOJ],
    nombro: usize,
}

impl MemoraMapo {
    pub const fn nova() -> Self {
        Self {
            regionoj: [MemoraRegiono::MALPLENA; MAKSAJ_REGIONOJ],
            nombro: 0,
        }
    }

    pub fn aldoni(&mut self, regiono: MemoraRegiono) {
        if self.nombro < self.regionoj.len() {
            self.regionoj[self.nombro] = regiono;
            self.nombro += 1;
        }
    }

    pub fn regionoj(&self) -> &[MemoraRegiono] {
        &self.regionoj[..self.nombro]
    }

    pub fn uzebla_memoro(&self) -> u64 {
        self.regionoj()
            .iter()
            .filter(|regiono| regiono.uzebla())
            .map(|regiono| regiono.longo)
            .sum()
    }
}

/// Praŝarĝa modulo: fizika bildo liverita de la praŝarĝilo (Multiboot2 tipo 3
/// aŭ PVH-modlisto).
#[derive(Clone, Copy)]
pub struct Modulo {
    pub komenco: u64,
    pub longo: u64,
}

impl Modulo {
    pub const fn fino(&self) -> u64 {
        self.komenco + self.longo
    }
}

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
