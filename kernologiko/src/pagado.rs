//! Arkitekture sendependaj paĝtabelaj reguloj: indeksoj, aliniiĝo, la
//! posedregistro de privataj subtabeloj kaj la konataj enskribaj aranĝoj.
//!
//! La funkcioj estas puraj, do ili testiĝas sur la gastiganto kaj la
//! Kani-pruvoj alvokas ilin rekte. La aparataran legadon kaj skribadon faras
//! la arkitekturaj moduloj.

use crate::memoro::KADRA_GRANDO;

/// La nombro da paĝtabelaj niveloj en ambaŭ subtenataj arkoj.
pub const NIVELOJ: u32 = 4;
/// La nombro da eroj en unu paĝtabela nivelo.
pub const EROJ: usize = 512;

/// Ĉu la adreso estas kadre aliniiĝinta?
pub const fn aliniiĝinta(adreso: u64) -> bool {
    adreso.is_multiple_of(KADRA_GRANDO)
}

/// La amplekso de unu ero en la donita nivelo (0 = PT, 3 = PML4/L0).
pub const fn nivela_amplekso(nivelo: u32) -> u64 {
    1u64 << (12 + nivelo * 9)
}

/// La indekso de `adreso` en la tabelo de la donita nivelo.
pub const fn paĝa_indekso(adreso: u64, nivelo: u32) -> usize {
    ((adreso >> (12 + nivelo * 9)) & 0x1FF) as usize
}

/// La baza virtuala adreso, kiun kovras la ero `indekso` de la donita nivelo.
pub const fn indeksa_bazo(nivelo: u32, indekso: usize) -> u64 {
    indekso as u64 * nivela_amplekso(nivelo)
}

/// Posedregistro de la subtabeloj, kiujn unu adresa spaco alokis private.
///
/// La produkta instanco havas 512 bitojn (unu por ĉiu radika ero). La
/// parametro `VORTOJ` ekzistas por la pruvoj, kiuj uzas malpli grandan
/// instancon de la sama tipo.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PrivatajSubtabeloj<const VORTOJ: usize = 8> {
    vortoj: [u64; VORTOJ],
}

impl<const VORTOJ: usize> PrivatajSubtabeloj<VORTOJ> {
    pub const fn nova() -> Self {
        Self {
            vortoj: [0; VORTOJ],
        }
    }

    /// La nombro da indeksoj, kiujn la registro povas teni.
    pub const fn kapacito() -> usize {
        VORTOJ * 64
    }

    /// Registru `indekso` kiel privatan. Indekso ekster la kapacito estas
    /// senefika.
    pub fn registri(&mut self, indekso: usize) {
        if let Some((vorto, bito)) = Self::loko(indekso) {
            self.vortoj[vorto] |= 1u64 << bito;
        }
    }

    /// Forigu `indekso` el la registro. Indekso ekster la kapacito aŭ
    /// neregistrita indekso estas senefika.
    pub fn malregistri(&mut self, indekso: usize) {
        if let Some((vorto, bito)) = Self::loko(indekso) {
            self.vortoj[vorto] &= !(1u64 << bito);
        }
    }

    /// Ĉu `indekso` estas registrita kiel privata?
    pub fn estas(&self, indekso: usize) -> bool {
        Self::loko(indekso).is_some_and(|(vorto, bito)| self.vortoj[vorto] & (1u64 << bito) != 0)
    }

    /// La registritaj indeksoj, kreskante.
    pub fn indeksoj(&self) -> impl Iterator<Item = usize> + '_ {
        (0..Self::kapacito()).filter(|indekso| self.estas(*indekso))
    }

    /// Kiom da indeksoj estas registritaj.
    pub fn nombro(&self) -> usize {
        self.vortoj
            .iter()
            .map(|vorto| vorto.count_ones() as usize)
            .sum()
    }

    /// Ĉu la registro estas malplena?
    pub fn malplena(&self) -> bool {
        self.vortoj.iter().all(|vorto| *vorto == 0)
    }

    fn loko(indekso: usize) -> Option<(usize, u32)> {
        if indekso >= Self::kapacito() {
            return None;
        }
        Some((indekso / 64, (indekso % 64) as u32))
    }
}

/// La x86_64-aranĝo de foliaj PTE-oj.
pub mod x86_64 {
    use super::EROJ;
    use crate::adreso::PaĝaPermesoj;

    pub const PREZENTA: u64 = 1 << 0;
    pub const SKRIBEBA: u64 = 1 << 1;
    pub const UZANTA: u64 = 1 << 2;
    pub const NEKAŜEBLA: u64 = 1 << 4;
    pub const GRANDA: u64 = 1 << 7;
    pub const NE_PLENUMEBLA: u64 = 1 << 63;

    /// Bitoj 51:12 — la fizika kadro de folia ero.
    pub const ADRESO_MASKO: u64 = 0x000F_FFFF_FFFF_F000;
    /// Bitoj 51:21 — la bazo de 2 MiB grandega paĝo.
    pub const DU_MIB_BAZA_MASKO: u64 = 0x000F_FFFF_FFE0_0000;
    /// La nombro da vortoj por la posedregistro de la 512 radikaj eroj.
    pub const PRIVATAJ_VORTOJ: usize = EROJ / 64;

    /// La permesbitoj de folia PTE. `skribebla` kaj `plenumebla` estas
    /// sendependaj en la aranĝo; la W^X-politiko apartenas al la alvokanto.
    pub const fn permesaj_eroj(permesoj: PaĝaPermesoj) -> u64 {
        let mut valoro = 0;
        if permesoj.skribebla {
            valoro |= SKRIBEBA;
        }
        if permesoj.uzanta {
            valoro |= UZANTA;
        }
        if !permesoj.plenumebla {
            valoro |= NE_PLENUMEBLA;
        }
        if !permesoj.kaŝebla {
            valoro |= NEKAŜEBLA;
        }
        valoro
    }

    /// La permesoj, kiujn folia PTE donas al la tradukita paĝo.
    pub const fn permesoj_el_ero(ero: u64) -> PaĝaPermesoj {
        PaĝaPermesoj {
            skribebla: ero & SKRIBEBA != 0,
            plenumebla: ero & NE_PLENUMEBLA == 0,
            kaŝebla: ero & NEKAŜEBLA == 0,
            uzanta: ero & UZANTA != 0,
        }
    }
}

/// La arm64-aranĝo de foliaj kaj blokaj priskriboj.
pub mod arm64 {
    use crate::adreso::PaĝaPermesoj;

    pub const BLOKO: u64 = 1;
    pub const TABELO: u64 = 0b11;
    pub const PAĜO: u64 = 0b11;
    pub const AF: u64 = 1 << 10;
    pub const SH_ENA: u64 = 3 << 8;
    pub const PXN: u64 = 1 << 53;
    pub const UXN: u64 = 1 << 54;
    pub const AP_EL0: u64 = 1 << 6;
    pub const AP_RO: u64 = 1 << 7;
    pub const ATTR_MASK: u64 = 0b111 << 2;
    pub const ATTR_APARATO: u64 = 1 << 2;
    pub const ADRESA_MASKO: u64 = 0x0000_FFFF_FFFF_F000;
    pub const BLOKA_MASKO: u64 = 0x0000_FFFF_C000_0000;
    pub const DU_MIB_MASKO: u64 = 0x0000_FFFF_FFE0_0000;

    /// La atributbitoj de folia aŭ bloka priskribo.
    pub const fn atributo(permesoj: PaĝaPermesoj) -> u64 {
        let mut ero = AF;
        if permesoj.kaŝebla {
            ero |= SH_ENA;
        } else {
            ero |= ATTR_APARATO;
        }
        if permesoj.uzanta {
            ero |= AP_EL0 | PXN;
            if !permesoj.plenumebla {
                ero |= UXN;
            }
        } else {
            ero |= UXN;
            if !permesoj.plenumebla {
                ero |= PXN;
            }
        }
        if !permesoj.skribebla {
            ero |= AP_RO;
        }
        ero
    }

    /// La permesoj, kiujn priskribo donas al la tradukita paĝo.
    pub const fn el_descriptor(ero: u64) -> PaĝaPermesoj {
        let uzanta = ero & AP_EL0 != 0;
        PaĝaPermesoj {
            skribebla: ero & AP_RO == 0,
            plenumebla: if uzanta {
                ero & UXN == 0
            } else {
                ero & PXN == 0
            },
            kaŝebla: ero & ATTR_MASK == 0,
            uzanta,
        }
    }
}

#[cfg(test)]
mod testoj {
    use super::arm64;
    use super::x86_64;
    use super::{EROJ, NIVELOJ, PrivatajSubtabeloj, aliniiĝinta, indeksa_bazo, paĝa_indekso};
    use crate::adreso::PaĝaPermesoj;
    use crate::memoro::KADRA_GRANDO;

    fn permesoj(skribebla: bool, plenumebla: bool, kaŝebla: bool, uzanta: bool) -> PaĝaPermesoj {
        PaĝaPermesoj {
            skribebla,
            plenumebla,
            kaŝebla,
            uzanta,
        }
    }

    fn samaj(dekstra: PaĝaPermesoj, maldekstra: PaĝaPermesoj) -> bool {
        dekstra.skribebla == maldekstra.skribebla
            && dekstra.plenumebla == maldekstra.plenumebla
            && dekstra.kaŝebla == maldekstra.kaŝebla
            && dekstra.uzanta == maldekstra.uzanta
    }

    #[test]
    fn aliniiĝo_postulas_kadran_oblecon() {
        assert!(aliniiĝinta(0));
        assert!(aliniiĝinta(KADRA_GRANDO));
        assert!(!aliniiĝinta(KADRA_GRANDO - 1));
        assert!(!aliniiĝinta(KADRA_GRANDO + 1));
        assert!(aliniiĝinta(u64::MAX & !(KADRA_GRANDO - 1)));
        assert!(!aliniiĝinta(u64::MAX));
    }

    #[test]
    fn indeksoj_kaj_bazoj_estas_inversaj() {
        for nivelo in 0..NIVELOJ {
            for indekso in [0, 1, 511] {
                let bazo = indeksa_bazo(nivelo, indekso);
                assert_eq!(paĝa_indekso(bazo, nivelo), indekso);
                assert_eq!(paĝa_indekso(bazo + 1, nivelo), indekso);
            }
        }
    }

    #[test]
    fn indeksoj_restas_en_la_tabelo() {
        for nivelo in 0..NIVELOJ {
            for adreso in [0, 0xFFF, KADRA_GRANDO, 0x7FFF_FFFF_FFFF, u64::MAX] {
                assert!(paĝa_indekso(adreso, nivelo) < EROJ);
            }
        }
    }

    #[test]
    fn privata_registro_kalkulas_sen_duplikatoj() {
        let mut registro = PrivatajSubtabeloj::<1>::nova();
        assert!(registro.malplena());
        assert_eq!(registro.nombro(), 0);

        registro.registri(3);
        registro.registri(3);
        registro.registri(63);
        assert_eq!(registro.nombro(), 2);
        assert!(registro.estas(3));
        assert!(registro.estas(63));
        assert!(!registro.estas(2));

        registro.malregistri(3);
        assert_eq!(registro.nombro(), 1);
        assert!(!registro.estas(3));
        registro.malregistri(63);
        assert!(registro.malplena());
    }

    #[test]
    fn privataj_indeksoj_estas_traireblaj() {
        let mut registro = PrivatajSubtabeloj::<1>::nova();
        registro.registri(5);
        registro.registri(63);
        let mut viditaj = [0usize; 2];
        let mut nombro = 0;
        for indekso in registro.indeksoj() {
            viditaj[nombro] = indekso;
            nombro += 1;
        }
        assert_eq!(nombro, 2);
        assert_eq!(viditaj, [5, 63]);
    }

    #[test]
    fn privata_registro_ignoras_indeksojn_ekster_la_kapacito() {
        let mut registro = PrivatajSubtabeloj::<1>::nova();
        registro.registri(64);
        registro.registri(usize::MAX);
        assert!(registro.malplena());
        assert!(!registro.estas(64));
        registro.malregistri(64);
    }

    #[test]
    fn x86_permesoj_estas_reverseblaj() {
        for skribebla in [false, true] {
            for plenumebla in [false, true] {
                for kaŝebla in [false, true] {
                    for uzanta in [false, true] {
                        let originalo = permesoj(skribebla, plenumebla, kaŝebla, uzanta);
                        let ero = x86_64::permesaj_eroj(originalo);
                        let malkodita = x86_64::permesoj_el_ero(ero);
                        assert!(samaj(originalo, malkodita));
                    }
                }
            }
        }
    }

    #[test]
    fn x86_kodo_estas_rx_kaj_datumoj_rw_nx() {
        let kodo = x86_64::permesaj_eroj(PaĝaPermesoj::KERNA_KODO);
        assert_eq!(kodo & x86_64::SKRIBEBA, 0);
        assert_eq!(kodo & x86_64::NE_PLENUMEBLA, 0);

        let datumoj = x86_64::permesaj_eroj(PaĝaPermesoj::KERNA_DATUMOJ);
        assert_ne!(datumoj & x86_64::SKRIBEBA, 0);
        assert_ne!(datumoj & x86_64::NE_PLENUMEBLA, 0);
    }

    #[test]
    fn arm_permesoj_estas_reverseblaj() {
        for skribebla in [false, true] {
            for plenumebla in [false, true] {
                for kaŝebla in [false, true] {
                    for uzanta in [false, true] {
                        let originalo = permesoj(skribebla, plenumebla, kaŝebla, uzanta);
                        let ero = arm64::atributo(originalo);
                        let malkodita = arm64::el_descriptor(ero | arm64::PAĜO);
                        assert!(samaj(originalo, malkodita));
                    }
                }
            }
        }
    }

    #[test]
    fn arm_kodo_estas_rx_kaj_datumoj_rw_nx() {
        let kodo = arm64::atributo(PaĝaPermesoj::KERNA_KODO) | arm64::PAĜO;
        assert_eq!(arm64::el_descriptor(kodo).skribebla, false);
        assert!(arm64::el_descriptor(kodo).plenumebla);

        let datumoj = arm64::atributo(PaĝaPermesoj::KERNA_DATUMOJ) | arm64::PAĜO;
        assert!(arm64::el_descriptor(datumoj).skribebla);
        assert!(!arm64::el_descriptor(datumoj).plenumebla);
    }
}
