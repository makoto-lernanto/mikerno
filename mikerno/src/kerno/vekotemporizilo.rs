//! Minimum-kupo de dormantaj fadenoj, ordigita laŭ absoluta temporizila tiko.
//!
//! La efektivigo loĝas en `kernologiko::vekotemporizilo`, kie ĝi testeĝas sur
//! la gastiganto; ĉi tiu modulo nur re-eksportas ĝin por la planisto.

pub use kernologiko::vekotemporizilo::Vekotemporizilo;

#[cfg(kani)]
mod kani_proofs {
    //! Pruvoj por la veka kupo: la pozicia tabelo, la kupo-ordo kaj la ordo de
    //! `elpreni`. La baza stato estas determinisma — nur la reenmetoj portas
    //! arbitrajn limdatojn — por ke CBMC restu malpeza en CI.

    use super::*;
    use kernologiko::MAKSAJ_FADENOJ;

    /// Limdatoj de la baza stato: malsamaj valoroj kun egalo (dufoje `7`) por
    /// ekzerci la komparojn de la kupo. La `0` de la fadeno `4` fiksas la
    /// minimumon, do la pruvoj povas aserti ĝin sendepende de la arbitraj
    /// reenmetoj.
    const BAZAJ: [u64; MAKSAJ_FADENOJ] = [5, 2, 7, 0, 6, 3, 7, 1];

    /// Plenigu la kupon per la baza stato.
    fn plenigu(kupo: &mut Vekotemporizilo) {
        for id in 1..=MAKSAJ_FADENOJ as u64 {
            kupo.enmeti(id, BAZAJ[id as usize - 1]);
        }
    }

    /// Arbitra reenmeto konservas la kupon kohera kaj lasas la pinton je la
    /// fiksa minimumo `0`.
    #[kani::proof]
    #[kani::unwind(11)]
    fn pinto_estas_minimumo() {
        let mut kupo = Vekotemporizilo::nova();
        plenigu(&mut kupo);
        kupo.enmeti(6, kani::any::<u8>() as u64);
        assert!(kupo.kohera());
        assert_eq!(kupo.nombro(), MAKSAJ_FADENOJ);
        assert_eq!(kupo.pinti().map(|(limdato, _)| limdato), Some(0));
    }

    /// `elpreni` redonas ĉiujn erojn laŭ ne-malpliiĝanta limdato kaj lasas la
    /// kupon kohera.
    #[kani::proof]
    #[kani::unwind(11)]
    fn elpreni_ordonas() {
        let mut kupo = Vekotemporizilo::nova();
        plenigu(&mut kupo);
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
        assert_eq!(kupo.pinti(), None);
    }

    /// Reenmeto ĝisdatigas la limdaton de ekzistanta fadeno sen duobligi la
    /// eron kaj restarigas koheran kupon.
    #[kani::proof]
    #[kani::unwind(11)]
    fn reenmeto_konservas_la_kupon() {
        let mut kupo = Vekotemporizilo::nova();
        plenigu(&mut kupo);
        let celo = (kani::any::<u8>() % MAKSAJ_FADENOJ as u8 + 1) as u64;
        kupo.enmeti(celo, kani::any::<u8>() as u64);
        assert!(kupo.kohera());
        assert_eq!(kupo.nombro(), MAKSAJ_FADENOJ);
    }

    /// Forigo redonas la limdaton de la celo, restarigas koheran kupon kaj
    /// dua forigo de la sama celo redonas `None`.
    #[kani::proof]
    #[kani::unwind(11)]
    fn forigo_konservas_la_kupon() {
        let mut kupo = Vekotemporizilo::nova();
        plenigu(&mut kupo);
        let celo = (kani::any::<u8>() % MAKSAJ_FADENOJ as u8 + 1) as u64;
        assert_eq!(kupo.forigi(celo), Some(BAZAJ[celo as usize - 1]));
        assert!(kupo.kohera());
        assert_eq!(kupo.nombro(), MAKSAJ_FADENOJ - 1);
        assert_eq!(kupo.forigi(celo), None);
    }

    /// Forigo post reenmeto redonas la novan limdaton; la kohero post
    /// reenmeto estas aparte pruvita, do ĉi tiu pruvo restas malpeza.
    #[kani::proof]
    #[kani::unwind(11)]
    fn forigo_redonas_la_novan_limdaton() {
        let mut kupo = Vekotemporizilo::nova();
        plenigu(&mut kupo);
        let nova = kani::any::<u8>() as u64;
        kupo.enmeti(4, nova);
        assert_eq!(kupo.forigi(4), Some(nova));
    }
}
