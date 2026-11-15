//! Bitmapa alokilo de fizikaj kadroj (4 KiB) en la unuaj 4 GiB.

use super::{KADRA_GRANDO, MemoraMapo, MemoraRegiono};

pub const MAKSA_FIZIKA_MEMORO: u64 = 4 * 1024 * 1024 * 1024;
pub const MAKSA_KADROJ: usize = (MAKSA_FIZIKA_MEMORO / KADRA_GRANDO) as usize;
const BITMAPO_BAJTOJ: usize = MAKSA_KADROJ / 8;

unsafe extern "C" {
    static __kerna_komenco: u8;
    static __kerna_fino: u8;
}

#[derive(Clone, Copy)]
pub struct Kadro(u64);

impl Kadro {
    pub const fn adreso(&self) -> u64 {
        self.0 * KADRA_GRANDO
    }

    pub const fn indekso(&self) -> usize {
        self.0 as usize
    }

    /// La kadro, kiu enhavas la donitan fizikan adreson.
    pub const fn el_adreso(adreso: u64) -> Self {
        Self(adreso / KADRA_GRANDO)
    }
}

pub struct KadraAlokilo {
    bitmapo: [u8; BITMAPO_BAJTOJ],
    sekva: usize,
}

impl KadraAlokilo {
    pub const fn nova() -> Self {
        Self {
            bitmapo: [0; BITMAPO_BAJTOJ],
            sekva: 0,
        }
    }

    /// `origino` estas la adreso, de kiu la KASLR-ŝelko kopiis la bildon; ĝiaj
    /// kadroj restas rezervitaj, ĉar la aktiva praŝarĝa paĝtabelo loĝas tie
    /// ĝis la unua `CR3`-ŝalto.
    pub fn pravalorizi(&mut self, mapo: &MemoraMapo, rezervindaj: &[MemoraRegiono], origino: u64) {
        self.bitmapo.fill(0xFF);

        for regiono in mapo.regionoj().iter().filter(|regiono| regiono.uzebla()) {
            let komenco = (regiono.komenco + KADRA_GRANDO - 1) & !(KADRA_GRANDO - 1);
            let fino = regiono.fino() & !(KADRA_GRANDO - 1);
            self.agordi_intervalon(komenco, fino, false);
        }

        // Malalta memoro enhavas BIOS/legacy-strukturojn.
        self.agordi_intervalon(0, 0x10_0000, true);
        let bildo_komenco = &raw const __kerna_komenco as u64;
        let bildo_fino = &raw const __kerna_fino as u64;
        self.agordi_intervalon(bildo_komenco, bildo_fino, true);
        // Post KASLR la praŝarĝa bildo restas ĉe `origino`; ĝiaj kadroj ne
        // rajtas esti reuzataj, eĉ kiam la nukleo mem loĝas alie.
        if origino != bildo_komenco {
            self.agordi_intervalon(origino, origino + (bildo_fino - bildo_komenco), true);
        }
        for regiono in rezervindaj {
            self.agordi_intervalon(regiono.komenco, regiono.fino(), true);
        }
    }

    pub fn aloki(&mut self) -> Option<Kadro> {
        while self.sekva < MAKSA_KADROJ {
            let indekso = self.sekva;
            self.sekva += 1;
            if !self.kadro_uzata(indekso) {
                self.marki(indekso, true);
                return Some(Kadro(indekso as u64));
            }
        }
        None
    }

    /// Aloku `n` sinsekvajn kadrojn kaj redonu la unuan. `None` se ne ekzistas
    /// sufiĉe longa libera intervalo. La kadroj restas okupataj ĝis eksplicita
    /// `liberigi`. Nur `arm64` uzas ĝin (`ramfb`).
    #[cfg(target_arch = "aarch64")]
    pub fn aloki_kontiguajn(&mut self, n: usize) -> Option<Kadro> {
        if n == 0 || n > MAKSA_KADROJ {
            return None;
        }
        let mut indekso = 0;
        while indekso + n <= MAKSA_KADROJ {
            if self.kadro_uzata(indekso) {
                indekso += 1;
                continue;
            }
            let mut longa = 1;
            while longa < n && !self.kadro_uzata(indekso + longa) {
                longa += 1;
            }
            if longa == n {
                for kadro in indekso..indekso + n {
                    self.marki(kadro, true);
                }
                return Some(Kadro(indekso as u64));
            }
            indekso += longa;
        }
        None
    }

    /// Redonu la kadron al la bitaro.
    ///
    /// Se la kadro jam estas libera, la voko estas senefika: duobla liberigo
    /// ne malfermas du kadrojn. La enhavo estas nuligata, por ke la sekva
    /// posedanto ne vidu malnovajn datumojn de alia procezo.
    pub fn liberigi(&mut self, kadro: Kadro) {
        let indekso = kadro.indekso();
        if indekso >= MAKSA_KADROJ || !self.kadro_uzata(indekso) {
            return;
        }
        // SAFETY: la kadro venas de la bitaro kaj kuŝas en la unuaj 4 GiB, kiuj
        // estas identece mapitaj; ĝi estas uzata, do ĝia enhavo estas nia.
        unsafe {
            core::ptr::write_bytes(kadro.adreso() as *mut u8, 0, KADRA_GRANDO as usize);
        }
        self.marki(indekso, false);
        // La skanilo nur antaŭeniras; retreatigu ĝin, por ke la kadro estu
        // reuzata antaŭ ol la tuta bitaro estos denove traserĉata.
        if indekso < self.sekva {
            self.sekva = indekso;
        }
    }

    /// Kiom da kadroj restas liberaj.
    pub fn liberaj(&self) -> usize {
        let uzataj: usize = self
            .bitmapo
            .iter()
            .map(|bajto| bajto.count_ones() as usize)
            .sum();
        MAKSA_KADROJ - uzataj
    }

    fn kadro_uzata(&self, indekso: usize) -> bool {
        self.bitmapo[indekso / 8] & (1 << (indekso % 8)) != 0
    }

    fn marki(&mut self, indekso: usize, uzata: bool) {
        let bito = 1u8 << (indekso % 8);
        if uzata {
            self.bitmapo[indekso / 8] |= bito;
        } else {
            self.bitmapo[indekso / 8] &= !bito;
        }
    }

    fn agordi_intervalon(&mut self, komenco: u64, fino: u64, uzata: bool) {
        if fino <= komenco {
            return;
        }
        let unua = (komenco / KADRA_GRANDO) as usize;
        let lasta = (fino.div_ceil(KADRA_GRANDO) as usize).min(MAKSA_KADROJ);
        for indekso in unua..lasta {
            self.marki(indekso, uzata);
        }
    }
}

#[cfg(kani)]
mod kani_proofs {
    //! Kani-pruvoj por la bitmapa kadro-alokilo.
    //!
    //! La produkta `KadraAlokilo` pritraktas pli ol unu milionon da kadroj kaj
    //! skanas ilin per `while`-buklo, do rekta pruvo eksplodus la stato-spacon.
    //! Ĉi tiu modulo spegulas ĝian bit-logikon sur 64 kadroj kaj limigas la
    //! enigojn per `kani::assume`, por ke la pruvoj restu trakteblaj.

    const PROVAJ_KADROJ: usize = 64;
    const PROVAJ_BAJTOJ: usize = PROVAJ_KADROJ / 8;

    /// Spegulo de `KadraAlokilo`: la samaj operacioj kaj bit-aranĝo, sed kun
    /// 64 kadroj anstataŭ `MAKSA_KADROJ`.
    struct Modelo {
        bitmapo: [u8; PROVAJ_BAJTOJ],
        sekva: usize,
    }

    impl Modelo {
        const fn nova() -> Self {
            Self {
                bitmapo: [0; PROVAJ_BAJTOJ],
                sekva: 0,
            }
        }

        fn kadro_uzata(&self, indekso: usize) -> bool {
            self.bitmapo[indekso / 8] & (1 << (indekso % 8)) != 0
        }

        fn marki(&mut self, indekso: usize, uzata: bool) {
            let bito = 1u8 << (indekso % 8);
            if uzata {
                self.bitmapo[indekso / 8] |= bito;
            } else {
                self.bitmapo[indekso / 8] &= !bito;
            }
        }

        fn aloki(&mut self) -> Option<usize> {
            while self.sekva < PROVAJ_KADROJ {
                let indekso = self.sekva;
                self.sekva += 1;
                if !self.kadro_uzata(indekso) {
                    self.marki(indekso, true);
                    return Some(indekso);
                }
            }
            None
        }

        /// Spegulo de la produkta `liberigi`: ignoras neokupatajn kadrojn kaj
        /// retreatigas `sekva`, por ke la kadro estu reuzata.
        fn liberigi(&mut self, indekso: usize) {
            if indekso >= PROVAJ_KADROJ || !self.kadro_uzata(indekso) {
                return;
            }
            self.marki(indekso, false);
            if indekso < self.sekva {
                self.sekva = indekso;
            }
        }

        fn agordi_intervalon(&mut self, komenco: u64, fino: u64, uzata: bool) {
            if fino <= komenco {
                return;
            }
            let unua = (komenco / super::KADRA_GRANDO) as usize;
            let lasta = (fino.div_ceil(super::KADRA_GRANDO) as usize).min(PROVAJ_KADROJ);
            for indekso in unua..lasta {
                self.marki(indekso, uzata);
            }
        }

        /// Konstruas staton, en kiu la unuaj `uzataj` kadroj estas okupataj.
        fn kun_uzataj(uzataj: usize) -> Self {
            let mut modelo = Self::nova();
            let mut indekso = 0;
            while indekso < uzataj {
                modelo.marki(indekso, true);
                indekso += 1;
            }
            // Produktaj `aloki`-vokoj ankaŭ antaŭenigus `sekva`.
            modelo.sekva = uzataj;
            modelo
        }

        /// Konstruas staton, en kiu ĉiuj kadroj krom `libera` estas okupataj.
        fn kun_unu_libera(libera: usize) -> Self {
            let mut modelo = Self::nova();
            let mut indekso = 0;
            while indekso < PROVAJ_KADROJ {
                if indekso != libera {
                    modelo.marki(indekso, true);
                }
                indekso += 1;
            }
            modelo
        }
    }

    /// `aloki` neniam eliras la bitaron, resendas kadron en limoj kaj
    /// preterpasas okupatajn kadrojn. Dua aloko ne redonas la saman kadron.
    #[kani::proof]
    #[kani::unwind(70)]
    fn aloki_preterpasas_uzatajn_kadrojn() {
        let uzataj: usize = kani::any();
        kani::assume(uzataj <= PROVAJ_KADROJ);
        let mut modelo = Modelo::kun_uzataj(uzataj);

        let unua = modelo.aloki();
        if uzataj == PROVAJ_KADROJ {
            assert_eq!(unua, None);
        } else {
            assert_eq!(unua, Some(uzataj));
            assert!(modelo.kadro_uzata(uzataj));

            let dua = modelo.aloki();
            if uzataj + 1 == PROVAJ_KADROJ {
                assert_eq!(dua, None);
            } else {
                assert_eq!(dua, Some(uzataj + 1));
                assert_ne!(dua, unua);
                assert!(modelo.kadro_uzata(uzataj + 1));
            }
        }
    }

    /// Kun ekzakte unu libera kadro, `aloki` redonas ĝin; poste la bitaro estas
    /// plena, do dua aloko redonas `None` anstataŭ duoblan alokon.
    #[kani::proof]
    #[kani::unwind(70)]
    fn neniom_da_duobla_alokado() {
        let libera: usize = kani::any();
        kani::assume(libera < PROVAJ_KADROJ);
        let mut modelo = Modelo::kun_unu_libera(libera);

        assert_eq!(modelo.aloki(), Some(libera));
        assert!(modelo.kadro_uzata(libera));
        assert_eq!(modelo.aloki(), None);
        assert_eq!(modelo.sekva, PROVAJ_KADROJ);
    }

    /// Liberigo malmarkas nur la donitan kadron, retreatigas `sekva` kaj
    /// permesas al `aloki` tuj reuzi ĝin.
    #[kani::proof]
    #[kani::unwind(70)]
    fn liberigo_lasas_la_kadron_reuzebla() {
        let indekso: usize = kani::any();
        kani::assume(indekso < PROVAJ_KADROJ);
        let mut modelo = Modelo::kun_uzataj(PROVAJ_KADROJ);

        assert!(modelo.kadro_uzata(indekso));
        modelo.liberigi(indekso);
        assert!(!modelo.kadro_uzata(indekso));
        assert_eq!(modelo.sekva, indekso);

        assert_eq!(modelo.aloki(), Some(indekso));
        assert!(modelo.kadro_uzata(indekso));
    }

    /// Duobla liberigo estas senefika: la dua voko ne ŝanĝas la bitaron kaj
    /// ne malfermas plian kadron.
    #[kani::proof]
    #[kani::unwind(70)]
    fn duobla_liberigo_estas_senefika() {
        let indekso: usize = kani::any();
        kani::assume(indekso < PROVAJ_KADROJ);
        let mut modelo = Modelo::kun_uzataj(PROVAJ_KADROJ);

        modelo.liberigi(indekso);
        let bitmapo = modelo.bitmapo;
        let sekva = modelo.sekva;
        modelo.liberigi(indekso);

        assert_eq!(modelo.bitmapo, bitmapo);
        assert_eq!(modelo.sekva, sekva);
    }

    /// `marki` kaj ĝia malo sekurigas la biton sen eliri la bitaron.
    #[kani::proof]
    fn marki_kaj_malmarki_sen_paniko() {
        let indekso: usize = kani::any();
        kani::assume(indekso < PROVAJ_KADROJ);
        let mut modelo = Modelo::nova();

        modelo.marki(indekso, true);
        assert!(modelo.kadro_uzata(indekso));
        modelo.marki(indekso, false);
        assert!(!modelo.kadro_uzata(indekso));
    }

    /// `agordi_intervalon` akceptas arbitrajn 64-bitajn limojn sen eliri la
    /// bitaron; ĉiuj kadroj ene de la intervalo ricevas la petitan staton.
    #[kani::proof]
    #[kani::unwind(70)]
    fn agordi_intervalon_sen_paniko() {
        let mut modelo = Modelo::nova();
        let komenco: u64 = kani::any();
        let fino: u64 = kani::any();
        let uzata: bool = kani::any();

        modelo.agordi_intervalon(komenco, fino, uzata);

        if fino > komenco {
            let unua = (komenco / super::KADRA_GRANDO) as usize;
            let lasta = (fino.div_ceil(super::KADRA_GRANDO) as usize).min(PROVAJ_KADROJ);
            let mut indekso = unua.min(PROVAJ_KADROJ);
            while indekso < lasta {
                assert_eq!(modelo.kadro_uzata(indekso), uzata);
                indekso += 1;
            }
        }
    }
}
