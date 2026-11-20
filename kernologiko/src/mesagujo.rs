//! Finpunkta mesaĝujo: fiksa FIFO, kiu konservas la ordon kaj ebligas fontan
//! filtron.

use libmikerno::Mesago;

use crate::FadenoId;

/// La mesaĝvica kapacito; la testa premrega scenaro uzas `+ 1` mesaĝojn.
pub const MAKSAJ_MESAGOJ: usize = 16;

#[derive(Clone, Copy)]
pub struct Ero {
    pub fonto: FadenoId,
    pub mesago: Mesago,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Eraro {
    Plena,
}

#[derive(Clone, Copy)]
pub struct Mesagujo {
    eroj: [Option<Ero>; MAKSAJ_MESAGOJ],
    nombro: usize,
}

impl Mesagujo {
    pub const fn nova() -> Self {
        Self {
            eroj: [None; MAKSAJ_MESAGOJ],
            nombro: 0,
        }
    }

    pub const fn plena(&self) -> bool {
        self.nombro >= MAKSAJ_MESAGOJ
    }

    pub fn puŝi(&mut self, ero: Ero) -> Result<(), Eraro> {
        if self.plena() {
            return Err(Eraro::Plena);
        }
        self.eroj[self.nombro] = Some(ero);
        self.nombro += 1;
        Ok(())
    }

    /// Prenu la unuan mesaĝon; `Some(fonto)` limigas al tiu sendinto, `None`
    /// akceptas ajnan fonton (la fonta filtro de `RICEVI_TEMPO`, C4).
    pub fn preni_de(&mut self, fonto: Option<FadenoId>) -> Option<Mesago> {
        let indekso = (0..self.nombro)
            .find(|&i| self.eroj[i].is_some_and(|e| fonto.is_none_or(|f| e.fonto == f)))?;
        let ero = self.eroj[indekso].take()?;
        for movo in indekso..self.nombro - 1 {
            self.eroj[movo] = self.eroj[movo + 1];
        }
        self.nombro -= 1;
        self.eroj[self.nombro] = None;
        Some(ero.mesago)
    }

    /// La nombro da konservitaj mesaĝoj.
    #[doc(hidden)]
    pub const fn nombro(&self) -> usize {
        self.nombro
    }

    /// La ero ĉe `indekso`, se ĝi ekzistas. Por diagnozo kaj la formalaj
    /// pruvoj, kiuj kontrolas, ke la fendoj post `nombro` restas malplenaj.
    #[doc(hidden)]
    pub fn ero(&self, indekso: usize) -> Option<&Ero> {
        self.eroj.get(indekso)?.as_ref()
    }
}

#[cfg(test)]
mod testoj {
    use super::*;

    fn mesaĝo(tipo: u64) -> Mesago {
        Mesago::nova(tipo, 0, 0, &[])
    }

    fn ero(fonto: FadenoId, tipo: u64) -> Ero {
        Ero {
            fonto,
            mesago: mesaĝo(tipo),
        }
    }

    #[test]
    fn malplena_ujo_prenas_nenion() {
        let mut ujo = Mesagujo::nova();
        assert!(ujo.preni_de(Some(1)).is_none());
        assert!(ujo.preni_de(None).is_none());
        assert_eq!(ujo.nombro(), 0);
    }

    #[test]
    fn fifo_ordo_estas_konservata() {
        let mut ujo = Mesagujo::nova();
        assert!(ujo.puŝi(ero(1, 10)).is_ok());
        assert!(ujo.puŝi(ero(1, 11)).is_ok());
        assert_eq!(ujo.preni_de(Some(1)).map(|m| m.tipo), Some(10));
        assert_eq!(ujo.preni_de(Some(1)).map(|m| m.tipo), Some(11));
        assert!(ujo.preni_de(Some(1)).is_none());
        assert_eq!(ujo.nombro(), 0);
    }

    #[test]
    fn fonta_filtro_konservas_fremdajn() {
        let mut ujo = Mesagujo::nova();
        assert!(ujo.puŝi(ero(1, 1)).is_ok());
        assert!(ujo.puŝi(ero(2, 2)).is_ok());
        assert!(ujo.puŝi(ero(1, 3)).is_ok());

        assert_eq!(ujo.preni_de(Some(1)).map(|m| m.tipo), Some(1));
        assert_eq!(ujo.nombro(), 2);
        assert_eq!(ujo.preni_de(Some(1)).map(|m| m.tipo), Some(3));
        assert_eq!(ujo.nombro(), 1);
        assert_eq!(ujo.preni_de(None).map(|m| m.tipo), Some(2));
        assert!(ujo.preni_de(Some(2)).is_none());
        assert_eq!(ujo.nombro(), 0);
    }

    #[test]
    fn senfiltra_preno_prenas_la_unuan() {
        let mut ujo = Mesagujo::nova();
        assert!(ujo.puŝi(ero(7, 5)).is_ok());
        assert_eq!(ujo.preni_de(None).map(|m| m.tipo), Some(5));
    }

    #[test]
    fn plena_ujo_rifuzas_pliajn() {
        let mut ujo = Mesagujo::nova();
        for indekso in 0..MAKSAJ_MESAGOJ {
            assert!(ujo.puŝi(ero(1, indekso as u64)).is_ok());
        }
        assert!(ujo.plena());
        assert_eq!(ujo.puŝi(ero(1, 99)), Err(Eraro::Plena));
        assert_eq!(ujo.nombro(), MAKSAJ_MESAGOJ);
    }

    #[test]
    fn post_preno_nova_ero_eniras() {
        let mut ujo = Mesagujo::nova();
        for indekso in 0..MAKSAJ_MESAGOJ {
            assert!(ujo.puŝi(ero(1, indekso as u64)).is_ok());
        }
        assert_eq!(ujo.preni_de(None).map(|m| m.tipo), Some(0));
        assert!(ujo.puŝi(ero(1, 100)).is_ok());
        assert_eq!(ujo.nombro(), MAKSAJ_MESAGOJ);
    }

    #[test]
    fn fendoj_post_nombro_restas_malplenaj() {
        let mut ujo = Mesagujo::nova();
        assert!(ujo.puŝi(ero(1, 1)).is_ok());
        assert!(ujo.puŝi(ero(1, 2)).is_ok());
        assert!(ujo.ero(0).is_some());
        assert!(ujo.ero(1).is_some());
        assert!(ujo.ero(2).is_none());

        assert!(ujo.preni_de(None).is_some());
        assert!(ujo.ero(0).is_some());
        assert!(ujo.ero(1).is_none());
        assert!(ujo.ero(MAKSAJ_MESAGOJ).is_none());
    }

    #[test]
    fn malsamaj_fontoj_ne_miksiĝas() {
        let mut ujo = Mesagujo::nova();
        assert!(ujo.puŝi(ero(3, 30)).is_ok());
        assert!(ujo.puŝi(ero(4, 40)).is_ok());
        assert_eq!(ujo.preni_de(Some(4)).map(|m| m.tipo), Some(40));
        assert!(ujo.preni_de(Some(4)).is_none());
        assert_eq!(ujo.preni_de(None).map(|m| m.tipo), Some(30));
    }
}
