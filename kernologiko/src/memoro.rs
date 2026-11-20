//! Arkitekture sendependaj priskriboj de fizika memoro kaj praŝarĝaj
//! moduloj.

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

#[cfg(test)]
mod testoj {
    use super::*;

    fn regiono(komenco: u64, longo: u64, tipo: RegionaTipo) -> MemoraRegiono {
        MemoraRegiono {
            komenco,
            longo,
            tipo,
        }
    }

    #[test]
    fn regiona_fino_kaj_uzeblo() {
        let uzebla = regiono(0x1000, 0x2000, RegionaTipo::Uzebla);
        assert_eq!(uzebla.fino(), 0x3000);
        assert!(uzebla.uzebla());

        let rezervita = regiono(0, 0x1000, RegionaTipo::Rezervita);
        assert!(!rezervita.uzebla());
        let malplena = MemoraRegiono::MALPLENA;
        assert!(!malplena.uzebla());
        assert_eq!(malplena.fino(), 0);
    }

    #[test]
    fn malplena_mapo_havas_neniom() {
        let mapo = MemoraMapo::nova();
        assert!(mapo.regionoj().is_empty());
        assert_eq!(mapo.uzebla_memoro(), 0);
    }

    #[test]
    fn aldoni_konservas_la_ordon() {
        let mut mapo = MemoraMapo::nova();
        mapo.aldoni(regiono(0x1000, 0x1000, RegionaTipo::Uzebla));
        mapo.aldoni(regiono(0x9000, 0x1000, RegionaTipo::Rezervita));
        let regionoj = mapo.regionoj();
        assert_eq!(regionoj.len(), 2);
        assert_eq!(regionoj[0].komenco, 0x1000);
        assert_eq!(regionoj[1].komenco, 0x9000);
    }

    #[test]
    fn kapacito_ignoras_pliajn_regionojn() {
        let mut mapo = MemoraMapo::nova();
        for indekso in 0..=MAKSAJ_REGIONOJ {
            mapo.aldoni(regiono(
                indekso as u64 * 0x1000,
                0x1000,
                RegionaTipo::Uzebla,
            ));
        }
        let regionoj = mapo.regionoj();
        assert_eq!(regionoj.len(), MAKSAJ_REGIONOJ);
        assert_eq!(regionoj[MAKSAJ_REGIONOJ - 1].komenco, 0x1000 * 127);
    }

    #[test]
    fn uzebla_memoro_sumas_nur_uzeblajn_regionojn() {
        let mut mapo = MemoraMapo::nova();
        mapo.aldoni(regiono(0, 0x1000, RegionaTipo::Rezervita));
        mapo.aldoni(regiono(0x1000, 0x4000, RegionaTipo::Uzebla));
        mapo.aldoni(regiono(0x5000, 0x1000, RegionaTipo::Acpi));
        mapo.aldoni(regiono(0x6000, 0x2000, RegionaTipo::Uzebla));
        assert_eq!(mapo.uzebla_memoro(), 0x6000);
    }

    #[test]
    fn modulo_fino_kunsumas_komencon_kaj_longon() {
        let modulo = Modulo {
            komenco: 0x40_0000,
            longo: 0x1234,
        };
        assert_eq!(modulo.fino(), 0x40_1234);
    }
}
