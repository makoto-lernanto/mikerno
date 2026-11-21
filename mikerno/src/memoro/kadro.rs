//! Bitmapa alokilo de fizikaj kadroj (4 KiB) en la unuaj 4 GiB.

use super::{KADRA_GRANDO, MemoraMapo, MemoraRegiono};

pub const MAKSA_FIZIKA_MEMORO: u64 = 4 * 1024 * 1024 * 1024;
pub const MAKSA_KADROJ: usize = (MAKSA_FIZIKA_MEMORO / KADRA_GRANDO) as usize;
/// Grando de la bitaro, kiu kovras `MAKSA_KADROJ` kadrojn.
pub const BITMAPO_BAJTOJ: usize = MAKSA_KADROJ / 8;

const _: () = assert!(BITMAPO_BAJTOJ * 8 == MAKSA_KADROJ);

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

/// Bitmapa alokilo, ĵenera laŭ la grando de la bitaro. La nombro da kadroj
/// estas `BAJTOJ * 8`; la defaŭlta valoro kongruas kun `MAKSA_KADROJ`.
///
/// La parametro ekzistas por la pruvoj: Kani ne traktas la pli ol unu milionon
/// da kadroj de la produkta instanco, do ĝi instancigas la saman tipon kun
/// malgranda bitaro anstataŭ kopii la logikon.
pub struct KadraAlokilo<const BAJTOJ: usize = BITMAPO_BAJTOJ> {
    bitmapo: [u8; BAJTOJ],
    sekva: usize,
}

impl<const BAJTOJ: usize> KadraAlokilo<BAJTOJ> {
    const fn kadroj() -> usize {
        BAJTOJ * 8
    }

    pub const fn nova() -> Self {
        Self {
            bitmapo: [0; BAJTOJ],
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
        while self.sekva < Self::kadroj() {
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
    /// `liberigi`. Uzata de `ramfb` (arm64) kaj de la virtio-vicoj.
    pub fn aloki_kontiguajn(&mut self, n: usize) -> Option<Kadro> {
        if n == 0 || n > Self::kadroj() {
            return None;
        }
        let mut indekso = 0;
        while indekso + n <= Self::kadroj() {
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
        // SAFETY: `liberigi_kun` alvokas `nuligi` nur post kontrolo, ke la
        // kadro estas okupata; la indekso estas malpli ol `BAJTOJ * 8`, kaj la
        // enlinia aserto de `liberigi_kun` limigas tion al la unuaj 4 GiB, kiuj
        // estas identece mapitaj.
        self.liberigi_kun(kadro, |adreso| unsafe {
            core::ptr::write_bytes(adreso as *mut u8, 0, KADRA_GRANDO as usize);
        });
    }

    /// La komuna vojo de `liberigi`: `nuligi` ricevas la fizikan adreson de
    /// kadro, kiu estis okupata. La pruvoj uzas senefikan `nuligi`, ĉar la
    /// nuligo skribas al absoluta fizika adreso, kiun CBMC ne modeligas.
    fn liberigi_kun<F: FnOnce(u64)>(&mut self, kadro: Kadro, nuligi: F) {
        const { assert!(BAJTOJ * 8 <= MAKSA_KADROJ) };
        let indekso = kadro.indekso();
        if indekso >= Self::kadroj() || !self.kadro_uzata(indekso) {
            return;
        }
        nuligi(kadro.adreso());
        self.marki(indekso, false);
        // La skanilo nur antaŭeniras; retreatigu ĝin, por ke la kadro estu
        // reuzata antaŭ ol la tuta bitaro estos denove traserĉata.
        if indekso < self.sekva {
            self.sekva = indekso;
        }
    }

    /// Kiom da kadroj restas liberaj.
    pub fn liberaj(&self) -> usize {
        // Vorta popkalkulo anstataŭ bajta `count_ones`-redukto: la `aarch64`
        // vektorigilo de la nuna rustc miskalkulas la bajtan formon en kelkaj
        // enliniaj kopioj (la ĝusta 984 598 fariĝis 8 292 689 659 472).
        let mut uzataj = 0usize;
        let (vortoj, resto) = self.bitmapo.as_chunks::<8>();
        for vorto in vortoj {
            uzataj += u64::from_le_bytes(*vorto).count_ones() as usize;
        }
        for bajto in resto {
            uzataj += bajto.count_ones() as usize;
        }
        Self::kadroj() - uzataj
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
        let lasta = (fino.div_ceil(KADRA_GRANDO) as usize).min(Self::kadroj());
        for indekso in unua..lasta {
            self.marki(indekso, uzata);
        }
    }
}

#[cfg(kani)]
mod kani_proofs {
    //! Pruvoj por la produktaj metodoj de `KadraAlokilo`.
    //!
    //! La produkta instanco havas pli ol unu milionon da kadroj kaj skanas ilin
    //! per `while`-bukloj, do rekta pruvo eksplodus la stato-spacon. La tipo
    //! estas ĵenera laŭ la bitara grando, do ĉi tiuj pruvoj instancigas la
    //! saman tipon kun 64 kadroj (`KadraAlokilo<8>`) kaj alvokas la samajn
    //! metodojn, kiujn la kerno uzas. Neniu logiko estas duobligata.

    use super::{KadraAlokilo, Kadro};

    const PROVAJ_BAJTOJ: usize = 8;
    const PROVAJ_KADROJ: usize = PROVAJ_BAJTOJ * 8;

    type Provilo = KadraAlokilo<PROVAJ_BAJTOJ>;

    /// Konstruas staton, en kiu la unuaj `uzataj` kadroj estas okupataj.
    fn kun_uzataj(uzataj: usize) -> Provilo {
        let mut alokilo = Provilo::nova();
        let mut indekso = 0;
        while indekso < uzataj {
            alokilo.marki(indekso, true);
            indekso += 1;
        }
        // Produktaj `aloki`-vokoj ankaŭ antaŭenigus `sekva`.
        alokilo.sekva = uzataj;
        alokilo
    }

    /// Konstruas staton, en kiu ĉiuj kadroj krom `libera` estas okupataj.
    fn kun_unu_libera(libera: usize) -> Provilo {
        let mut alokilo = Provilo::nova();
        let mut indekso = 0;
        while indekso < PROVAJ_KADROJ {
            if indekso != libera {
                alokilo.marki(indekso, true);
            }
            indekso += 1;
        }
        alokilo
    }

    /// `aloki` neniam eliras la bitaron, resendas kadron en limoj kaj
    /// preterpasas okupatajn kadrojn. Dua aloko ne redonas la saman kadron.
    #[kani::proof]
    #[kani::unwind(70)]
    fn aloki_preterpasas_uzatajn_kadrojn() {
        let uzataj: usize = kani::any();
        kani::assume(uzataj <= PROVAJ_KADROJ);
        let mut alokilo = kun_uzataj(uzataj);

        let unua = alokilo.aloki();
        if uzataj == PROVAJ_KADROJ {
            assert!(unua.is_none());
        } else {
            assert_eq!(unua.map(|kadro| kadro.indekso()), Some(uzataj));
            assert!(alokilo.kadro_uzata(uzataj));

            let dua = alokilo.aloki();
            if uzataj + 1 == PROVAJ_KADROJ {
                assert!(dua.is_none());
            } else {
                assert_eq!(dua.map(|kadro| kadro.indekso()), Some(uzataj + 1));
                assert_ne!(
                    dua.map(|kadro| kadro.indekso()),
                    unua.map(|kadro| kadro.indekso())
                );
                assert!(alokilo.kadro_uzata(uzataj + 1));
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
        let mut alokilo = kun_unu_libera(libera);

        assert_eq!(alokilo.aloki().map(|kadro| kadro.indekso()), Some(libera));
        assert!(alokilo.kadro_uzata(libera));
        assert!(alokilo.aloki().is_none());
        assert_eq!(alokilo.sekva, PROVAJ_KADROJ);
    }

    /// Liberigo malmarkas nur la donitan kadron, retreatigas `sekva` kaj
    /// permesas al `aloki` tuj reuzi ĝin.
    #[kani::proof]
    #[kani::unwind(70)]
    fn liberigo_lasas_la_kadron_reuzebla() {
        let indekso: usize = kani::any();
        kani::assume(indekso < PROVAJ_KADROJ);
        let mut alokilo = kun_uzataj(PROVAJ_KADROJ);

        assert!(alokilo.kadro_uzata(indekso));
        alokilo.liberigi_kun(Kadro(indekso as u64), |_| {});
        assert!(!alokilo.kadro_uzata(indekso));
        assert_eq!(alokilo.sekva, indekso);

        assert_eq!(alokilo.aloki().map(|kadro| kadro.indekso()), Some(indekso));
        assert!(alokilo.kadro_uzata(indekso));
    }

    /// Duobla liberigo estas senefika: la dua voko ne ŝanĝas la bitaron kaj
    /// ne malfermas plian kadron.
    #[kani::proof]
    #[kani::unwind(70)]
    fn duobla_liberigo_estas_senefika() {
        let indekso: usize = kani::any();
        kani::assume(indekso < PROVAJ_KADROJ);
        let mut alokilo = kun_uzataj(PROVAJ_KADROJ);

        alokilo.liberigi_kun(Kadro(indekso as u64), |_| {});
        let bitmapo = alokilo.bitmapo;
        let sekva = alokilo.sekva;
        alokilo.liberigi_kun(Kadro(indekso as u64), |_| {});

        assert_eq!(alokilo.bitmapo, bitmapo);
        assert_eq!(alokilo.sekva, sekva);
    }

    /// `marki` kaj ĝia malo sekurigas la biton sen eliri la bitaron.
    #[kani::proof]
    fn marki_kaj_malmarki_sen_paniko() {
        let indekso: usize = kani::any();
        kani::assume(indekso < PROVAJ_KADROJ);
        let mut alokilo = Provilo::nova();

        alokilo.marki(indekso, true);
        assert!(alokilo.kadro_uzata(indekso));
        alokilo.marki(indekso, false);
        assert!(!alokilo.kadro_uzata(indekso));
    }

    /// `agordi_intervalon` akceptas arbitrajn 64-bitajn limojn sen eliri la
    /// bitaron; ĉiuj kadroj ene de la intervalo ricevas la petitan staton.
    #[kani::proof]
    #[kani::unwind(70)]
    fn agordi_intervalon_sen_paniko() {
        let mut alokilo = Provilo::nova();
        let komenco: u64 = kani::any();
        let fino: u64 = kani::any();
        let uzata: bool = kani::any();

        alokilo.agordi_intervalon(komenco, fino, uzata);

        if fino > komenco {
            let unua = (komenco / super::KADRA_GRANDO) as usize;
            let lasta = (fino.div_ceil(super::KADRA_GRANDO) as usize).min(PROVAJ_KADROJ);
            let mut indekso = unua.min(PROVAJ_KADROJ);
            while indekso < lasta {
                assert_eq!(alokilo.kadro_uzata(indekso), uzata);
                indekso += 1;
            }
        }
    }
}
