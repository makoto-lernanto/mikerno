//! Cirkla vico de pretaj fadenoj.
//!
//! La vico havas unu plian lokon ol daŭre akcepteblaj fadenoj, do
//! `kapo == vosto` ĉiam signifas malplena.

use crate::{FadenoId, MAKSAJ_FADENOJ};

pub struct PretaVico {
    eroj: [FadenoId; MAKSAJ_FADENOJ + 1],
    kapo: usize,
    vosto: usize,
}

impl PretaVico {
    pub const fn nova() -> Self {
        Self {
            eroj: [0; MAKSAJ_FADENOJ + 1],
            kapo: 0,
            vosto: 0,
        }
    }

    pub fn puŝi(&mut self, id: FadenoId) {
        self.eroj[self.vosto] = id;
        self.vosto = (self.vosto + 1) % self.eroj.len();
    }

    pub fn pop(&mut self) -> Option<FadenoId> {
        if self.kapo == self.vosto {
            return None;
        }
        let id = self.eroj[self.kapo];
        self.kapo = (self.kapo + 1) % self.eroj.len();
        Some(id)
    }
}

#[cfg(test)]
mod testoj {
    use super::*;

    #[test]
    fn malplena_vico_redonas_nenion() {
        let mut vico = PretaVico::nova();
        assert_eq!(vico.pop(), None);
    }

    #[test]
    fn fifo_ordo_estas_konservata() {
        let mut vico = PretaVico::nova();
        for id in 1..=MAKSAJ_FADENOJ as FadenoId {
            vico.puŝi(id);
        }
        for id in 1..=MAKSAJ_FADENOJ as FadenoId {
            assert_eq!(vico.pop(), Some(id));
        }
        assert_eq!(vico.pop(), None);
    }

    #[test]
    fn cikla_reuzo_konservas_la_ordon() {
        let mut vico = PretaVico::nova();
        for id in 1..=MAKSAJ_FADENOJ as FadenoId {
            vico.puŝi(id);
        }
        for id in 1..=4 {
            assert_eq!(vico.pop(), Some(id));
        }
        for id in 20..=23 {
            vico.puŝi(id);
        }
        for id in 5..=8 {
            assert_eq!(vico.pop(), Some(id));
        }
        for id in 20..=23 {
            assert_eq!(vico.pop(), Some(id));
        }
        assert_eq!(vico.pop(), None);
    }

    #[test]
    fn plurfoja_malplenigo_estas_senefika() {
        let mut vico = PretaVico::nova();
        vico.puŝi(3);
        assert_eq!(vico.pop(), Some(3));
        assert_eq!(vico.pop(), None);
        assert_eq!(vico.pop(), None);
        vico.puŝi(4);
        assert_eq!(vico.pop(), Some(4));
    }

    #[test]
    fn kapacito_estas_la_planebla_nombro() {
        assert_eq!(PretaVico::nova().eroj.len(), MAKSAJ_FADENOJ + 1);
    }
}
