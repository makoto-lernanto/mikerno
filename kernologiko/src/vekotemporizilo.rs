//! Minimum-kupo de dormantaj fadenoj, ordigita laŭ absoluta temporizila tiko.
//!
//! Ĉiu fadeno havas maksimume unu eron. `pozicioj` tenas la indicon en la
//! tabelo, do enigo, forigo kaj anstataŭigo restas O(log n) sen memora
//! alokado: la kapacito estas la nombro de planeblaj fadenoj.

use crate::{FadenoId, MAKSAJ_FADENOJ};

/// Mankanta indico en `pozicioj`.
const MANKAS: usize = usize::MAX;

pub struct Vekotemporizilo {
    eroj: [(u64, FadenoId); MAKSAJ_FADENOJ],
    pozicioj: [usize; MAKSAJ_FADENOJ + 1],
    nombro: usize,
}

impl Vekotemporizilo {
    pub const fn nova() -> Self {
        Self {
            eroj: [(0, 0); MAKSAJ_FADENOJ],
            pozicioj: [MANKAS; MAKSAJ_FADENOJ + 1],
            nombro: 0,
        }
    }

    fn pozicio(&self, id: FadenoId) -> Option<usize> {
        let pozicio = *self.pozicioj.get(id as usize)?;
        (pozicio != MANKAS).then_some(pozicio)
    }

    /// Enigu la fadenon aŭ anstataŭigu ĝian limdaton.
    pub fn enmeti(&mut self, id: FadenoId, limdato: u64) {
        if let Some(pozicio) = self.pozicio(id) {
            self.eroj[pozicio] = (limdato, id);
            self.supren(pozicio);
            self.malsupren(pozicio);
            return;
        }
        assert!(self.nombro < self.eroj.len(), "la veka kupo superpleniĝis");
        let pozicio = self.nombro;
        self.nombro += 1;
        self.eroj[pozicio] = (limdato, id);
        self.pozicioj[id as usize] = pozicio;
        self.supren(pozicio);
    }

    /// Forigu la fadenon kaj redonu ĝian limdaton, se ĝi estis en la kupo.
    pub fn forigi(&mut self, id: FadenoId) -> Option<u64> {
        let pozicio = self.pozicio(id)?;
        let limdato = self.eroj[pozicio].0;
        self.pozicioj[id as usize] = MANKAS;
        self.nombro -= 1;
        if pozicio != self.nombro {
            let lasta = self.eroj[self.nombro];
            self.eroj[pozicio] = lasta;
            self.pozicioj[lasta.1 as usize] = pozicio;
            self.supren(pozicio);
            self.malsupren(pozicio);
        }
        Some(limdato)
    }

    /// La plej frua limdato, sen forigo.
    pub fn pinti(&self) -> Option<(u64, FadenoId)> {
        (self.nombro > 0).then(|| self.eroj[0])
    }

    /// Forigu kaj redonu la eron kun la plej frua limdato.
    pub fn elpreni(&mut self) -> Option<(u64, FadenoId)> {
        let (limdato, id) = self.pinti()?;
        self.forigi(id);
        Some((limdato, id))
    }

    /// La nombro da fadenoj en la kupo.
    #[doc(hidden)]
    pub const fn nombro(&self) -> usize {
        self.nombro
    }

    /// Ĉu la kupo estas kohera: ĉiu ero havas ĝustan indicon en `pozicioj`, la
    /// identigiloj estas unikaj, la kupo-ordo validas kaj ambaŭ tabeloj
    /// kongruas. Por la testoj kaj la formalaj pruvoj.
    #[doc(hidden)]
    pub fn kohera(&self) -> bool {
        if self.nombro > MAKSAJ_FADENOJ {
            return false;
        }
        let mut vidita = [false; MAKSAJ_FADENOJ + 1];
        let mut indekso = 0;
        while indekso < self.nombro {
            let (_, id) = self.eroj[indekso];
            let id = id as usize;
            if id == 0 || id > MAKSAJ_FADENOJ || vidita[id] || self.pozicioj[id] != indekso {
                return false;
            }
            vidita[id] = true;
            if indekso > 0 && self.eroj[indekso] < self.eroj[(indekso - 1) / 2] {
                return false;
            }
            indekso += 1;
        }
        let mut id = 0;
        while id <= MAKSAJ_FADENOJ {
            if (self.pozicioj[id] != MANKAS) != vidita[id] {
                return false;
            }
            id += 1;
        }
        true
    }

    fn interŝanĝi(&mut self, a: usize, b: usize) {
        if a == b {
            return;
        }
        self.eroj.swap(a, b);
        self.pozicioj[self.eroj[a].1 as usize] = a;
        self.pozicioj[self.eroj[b].1 as usize] = b;
    }

    fn supren(&mut self, mut pozicio: usize) {
        while pozicio > 0 {
            let patro = (pozicio - 1) / 2;
            if self.eroj[pozicio] >= self.eroj[patro] {
                break;
            }
            self.interŝanĝi(pozicio, patro);
            pozicio = patro;
        }
    }

    fn malsupren(&mut self, mut pozicio: usize) {
        loop {
            let maldekstra = 2 * pozicio + 1;
            if maldekstra >= self.nombro {
                break;
            }
            let dekstra = maldekstra + 1;
            let infano = if dekstra < self.nombro && self.eroj[dekstra] < self.eroj[maldekstra] {
                dekstra
            } else {
                maldekstra
            };
            if self.eroj[pozicio] <= self.eroj[infano] {
                break;
            }
            self.interŝanĝi(pozicio, infano);
            pozicio = infano;
        }
    }
}

#[cfg(test)]
mod testoj {
    use super::*;

    #[test]
    fn malplena_kupo_redonas_nenion() {
        let mut kupo = Vekotemporizilo::nova();
        assert_eq!(kupo.pinti(), None);
        assert_eq!(kupo.elpreni(), None);
        assert_eq!(kupo.forigi(1), None);
        assert_eq!(kupo.nombro(), 0);
    }

    #[test]
    fn elpreni_laŭkreskante_ordigas() {
        let mut kupo = Vekotemporizilo::nova();
        for (id, limdato) in [
            (1, 5),
            (2, 2),
            (3, 7),
            (4, 0),
            (5, 6),
            (6, 3),
            (7, 7),
            (8, 1),
        ] {
            kupo.enmeti(id, limdato);
        }
        assert_eq!(kupo.pinti().map(|(limdato, _)| limdato), Some(0));
        let mut antaŭa = 0;
        let mut nombro = 0;
        while let Some((limdato, id)) = kupo.elpreni() {
            assert!(limdato >= antaŭa);
            assert!((1..=MAKSAJ_FADENOJ as u64).contains(&id));
            antaŭa = limdato;
            nombro += 1;
        }
        assert_eq!(nombro, MAKSAJ_FADENOJ);
        assert!(kupo.kohera());
    }

    #[test]
    fn anstataŭigo_ĝisdatigas_la_limdaton() {
        let mut kupo = Vekotemporizilo::nova();
        kupo.enmeti(1, 5);
        kupo.enmeti(2, 3);
        assert_eq!(kupo.pinti(), Some((3, 2)));

        kupo.enmeti(1, 1);
        assert_eq!(kupo.pinti(), Some((1, 1)));
        assert_eq!(kupo.nombro(), 2);
        assert!(kupo.kohera());
    }

    #[test]
    fn forigi_redonas_la_limdaton() {
        let mut kupo = Vekotemporizilo::nova();
        kupo.enmeti(1, 4);
        kupo.enmeti(2, 2);
        assert_eq!(kupo.forigi(1), Some(4));
        assert_eq!(kupo.pinti(), Some((2, 2)));
        assert_eq!(kupo.forigi(1), None);
        assert!(kupo.kohera());
    }

    #[test]
    fn forigi_la_pinton_restarigas_la_kupon() {
        let mut kupo = Vekotemporizilo::nova();
        for (id, limdato) in [(1, 5), (2, 2), (3, 7), (4, 9)] {
            kupo.enmeti(id, limdato);
        }
        assert_eq!(kupo.elpreni(), Some((2, 2)));
        assert_eq!(kupo.nombro(), 3);
        assert!(kupo.kohera());
        assert_eq!(kupo.elpreni(), Some((5, 1)));
    }

    #[test]
    fn anstataŭigo_ne_duobligas() {
        let mut kupo = Vekotemporizilo::nova();
        for id in 1..=MAKSAJ_FADENOJ as FadenoId {
            kupo.enmeti(id, id);
        }
        kupo.enmeti(3, 100);
        assert_eq!(kupo.nombro(), MAKSAJ_FADENOJ);
        assert!(kupo.kohera());
    }

    #[test]
    fn kohero_restas_post_pluraj_operacioj() {
        let mut kupo = Vekotemporizilo::nova();
        for (id, limdato) in [(1, 5), (2, 2), (3, 7), (4, 0), (5, 6)] {
            kupo.enmeti(id, limdato);
        }
        kupo.forigi(2);
        kupo.enmeti(6, 4);
        kupo.enmeti(5, 1);
        assert!(kupo.kohera());
        assert_eq!(kupo.nombro(), 5);
    }

    #[test]
    #[should_panic(expected = "la veka kupo superpleniĝis")]
    fn superplenigo_panikas() {
        let mut kupo = Vekotemporizilo::nova();
        for id in 1..=MAKSAJ_FADENOJ as FadenoId + 1 {
            kupo.enmeti(id, id);
        }
    }
}
