//! Minimum-kupo de dormantaj fadenoj, ordigita laŭ absoluta temporizila tiko.
//!
//! Ĉiu fadeno havas maksimume unu eron. `pozicioj` tenas la indicon en la
//! tabelo, do enigo, forigo kaj anstataŭigo restas O(log n) sen memora
//! alokado: la kapacito estas la nombro de planeblaj fadenoj.

use super::objekto::FadenoId;
use super::planisto::MAKSAJ_FADENOJ;

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

#[cfg(kani)]
mod kani_proofs {
    //! Pruvoj por la veka kupo: la pozicia tabelo, la kupo-ordo kaj la ordo de
    //! `elpreni`.

    use super::*;

    /// Ĉu la kupo estas kohera: ĉiu ero havas ĝustan indicon en `pozicioj`, la
    /// identigiloj estas unikaj, la kupo-ordo validas kaj ambaŭ tabeloj kongruas.
    fn kohera(kupo: &Vekotemporizilo) -> bool {
        if kupo.nombro > MAKSAJ_FADENOJ {
            return false;
        }
        let mut vidita = [false; MAKSAJ_FADENOJ + 1];
        let mut indekso = 0;
        while indekso < kupo.nombro {
            let (_, id) = kupo.eroj[indekso];
            let id = id as usize;
            if id == 0 || id > MAKSAJ_FADENOJ || vidita[id] || kupo.pozicioj[id] != indekso {
                return false;
            }
            vidita[id] = true;
            if indekso > 0 && kupo.eroj[indekso] < kupo.eroj[(indekso - 1) / 2] {
                return false;
            }
            indekso += 1;
        }
        let mut id = 0;
        while id <= MAKSAJ_FADENOJ {
            if (kupo.pozicioj[id] != MANKAS) != vidita[id] {
                return false;
            }
            id += 1;
        }
        true
    }

    /// Enmeto de ĉiuj fadenoj kun arbitraj limdatoj lasas koheran kupon, kies
    /// pinto estas la minimumo.
    #[kani::proof]
    #[kani::unwind(12)]
    fn pinto_estas_minimumo() {
        let mut kupo = Vekotemporizilo::nova();
        let mut minimumo = u64::MAX;
        for id in 1..=MAKSAJ_FADENOJ as u64 {
            let limdato: u64 = kani::any();
            kupo.enmeti(id, limdato);
            minimumo = minimumo.min(limdato);
        }
        assert!(kohera(&kupo));
        assert_eq!(kupo.nombro, MAKSAJ_FADENOJ);
        assert_eq!(kupo.pinti().map(|(limdato, _)| limdato), Some(minimumo));
    }

    /// `elpreni` redonas ĉiujn erojn laŭ ne-malpliiĝanta limdato kaj lasas la
    /// kupon kohera post ĉiu paŝo.
    #[kani::proof]
    #[kani::unwind(12)]
    fn elpreni_ordonas() {
        let mut kupo = Vekotemporizilo::nova();
        for id in 1..=MAKSAJ_FADENOJ as u64 {
            kupo.enmeti(id, kani::any());
        }
        let mut antaŭa = 0;
        let mut nombro = 0;
        while let Some((limdato, id)) = kupo.elpreni() {
            assert!(limdato >= antaŭa);
            assert!((1..=MAKSAJ_FADENOJ as u64).contains(&id));
            antaŭa = limdato;
            nombro += 1;
            assert!(kohera(&kupo));
        }
        assert_eq!(nombro, MAKSAJ_FADENOJ);
        assert_eq!(kupo.pinti(), None);
    }

    /// Reenmeto ĝisdatigas la limdaton sen duobligi la eron; forigo redonas la
    /// limdaton kaj restarigas koheran kupon.
    #[kani::proof]
    #[kani::unwind(14)]
    fn reenmeto_kaj_forigo() {
        let mut kupo = Vekotemporizilo::nova();
        for id in 1..=MAKSAJ_FADENOJ as u64 {
            kupo.enmeti(id, kani::any());
        }
        let id: u64 = kani::any();
        kani::assume((1..=MAKSAJ_FADENOJ as u64).contains(&id));
        let nova: u64 = kani::any();
        kupo.enmeti(id, nova);
        assert!(kohera(&kupo));
        assert_eq!(kupo.nombro, MAKSAJ_FADENOJ);
        assert_eq!(kupo.forigi(id), Some(nova));
        assert!(kohera(&kupo));
        assert_eq!(kupo.nombro, MAKSAJ_FADENOJ - 1);
        assert_eq!(kupo.forigi(id), None);
    }
}
