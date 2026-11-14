//! Arkitekture sendependa virtuala memoro: adresoj, permesoj kaj la trait-oj,
//! per kiuj la nukleo priskribas paĝadon kaj adresajn spacojn.

use core::fmt;

use super::kadro::KadraAlokilo;
use super::{KADRA_GRANDO, MemoraMapo};

unsafe extern "C" {
    static __kerna_komenco: u8;
    static __kerna_kodo_komenco: u8;
    static __kerna_kodo_fino: u8;
    static __kerna_rodata_komenco: u8;
    static __kerna_rodata_fino: u8;
    static __kerna_datumoj_komenco: u8;
    static __kerna_fino: u8;
}

/// La sekciaj limoj de la kerna bildo. La identeca mapo uzas ilin por doni
/// al `.text` RX, al `.rodata` nur legadon kaj al `.data`/`.bss` RW+NX.
#[derive(Clone, Copy)]
pub struct KernajLimoj {
    /// Ĉapeloj kaj notoj antaŭ `.text` (Multiboot2, Xen-PVH).
    pub antaŭkodo: (u64, u64),
    pub kodo: (u64, u64),
    pub rodata: (u64, u64),
    pub datumoj: (u64, u64),
}

/// Adresoj de la sekciaj limoj el la ligilo.
pub fn kernaj_limoj() -> KernajLimoj {
    // Nur la adresoj de la simboloj estas prenitaj; la bildo mem ne estas legata.
    KernajLimoj {
        antaŭkodo: (
            &raw const __kerna_komenco as u64,
            &raw const __kerna_kodo_komenco as u64,
        ),
        kodo: (
            &raw const __kerna_kodo_komenco as u64,
            &raw const __kerna_kodo_fino as u64,
        ),
        rodata: (
            &raw const __kerna_rodata_komenco as u64,
            &raw const __kerna_rodata_fino as u64,
        ),
        datumoj: (
            &raw const __kerna_datumoj_komenco as u64,
            &raw const __kerna_fino as u64,
        ),
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirtuaAdreso(u64);

impl VirtuaAdreso {
    pub const fn nova(adreso: u64) -> Self {
        Self(adreso)
    }

    pub const fn adreso(&self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct FizikaAdreso(u64);

impl FizikaAdreso {
    pub const fn nova(adreso: u64) -> Self {
        Self(adreso)
    }

    pub const fn adreso(&self) -> u64 {
        self.0
    }
}

/// La komenco de la uzanta fenestro, super la identeca mapo de 4 GiB.
pub const UZANTA_BAZO: u64 = 0x0000_0001_0000_0000;
/// La supra limo de la uzanta fenestro en la kanonika malalta duono.
pub const UZANTA_FINO: u64 = 0x0000_0080_0000_0000;
/// Grando de la uzanta stako (64 KiB).
pub const UZANTA_STAKO_GRANDE: usize = 64 * 1024;
/// La pinto de la uzanta stako: bazo + 16 MiB; la stako kreskas malsupren.
pub const UZANTA_STAKO_SUPRO: u64 = UZANTA_BAZO + 16 * 1024 * 1024;

/// Rajtoj de unu paĝo. Legeblo ĉiam ĉeestas: la x86_64-backendo ne subtenas
/// tute nelegeblajn paĝojn.
#[derive(Clone, Copy)]
pub struct PaĝaPermesoj {
    pub skribebla: bool,
    pub plenumebla: bool,
    /// `false` por framebuffer-MMIO: la paĝo estas nekaŝebla (PCD).
    pub kaŝebla: bool,
    /// `true` por Ring 3-atingeblaj paĝoj: la bito `U` ĉe ĉiu nivelo.
    pub uzanta: bool,
}

impl PaĝaPermesoj {
    pub const SKRIBA: Self = Self {
        skribebla: true,
        plenumebla: false,
        kaŝebla: true,
        uzanta: false,
    };

    pub const SKRIBA_SENKAŜE: Self = Self {
        skribebla: true,
        plenumebla: false,
        kaŝebla: false,
        uzanta: false,
    };

    /// Kerna kodo (`.text`): Ring 0 rajtas legi kaj plenumi, ne skribi.
    pub const KERNA_KODO: Self = Self {
        skribebla: false,
        plenumebla: true,
        kaŝebla: true,
        uzanta: false,
    };

    /// Kernaj nurlegaj datumoj (`.rodata`, moduloj): sen skribado, sen plenumo.
    pub const KERNA_LEGA: Self = Self {
        skribebla: false,
        plenumebla: false,
        kaŝebla: true,
        uzanta: false,
    };

    /// Kernaj datumoj, `.bss` kaj la cetera uzebla RAM: skribebla, sen plenumo.
    pub const KERNA_DATUMOJ: Self = Self {
        skribebla: true,
        plenumebla: false,
        kaŝebla: true,
        uzanta: false,
    };

    /// Uzantaj datumoj kaj stako: Ring 3 rajtas legi kaj skribi, sen plenumo.
    pub const UZANTA_DATUMOJ: Self = Self {
        skribebla: true,
        plenumebla: false,
        kaŝebla: true,
        uzanta: true,
    };

    /// Nurlega uzanta paĝo (ekz. `.rodata`).
    pub const UZANTA_LEGA: Self = Self {
        skribebla: false,
        plenumebla: false,
        kaŝebla: true,
        uzanta: true,
    };

    /// Uzanta kodo: Ring 3 rajtas legi kaj plenumi, sen skribi.
    pub const UZANTA_KODO: Self = Self {
        skribebla: false,
        plenumebla: true,
        kaŝebla: true,
        uzanta: true,
    };
}

impl fmt::Display for PaĝaPermesoj {
    /// Mallonga formo por diagnozo: `R`, `RX`, `RW NX`, `RWX` plus `U`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("R")?;
        if self.skribebla {
            f.write_str("W")?;
        }
        if self.plenumebla {
            f.write_str("X")?;
        } else {
            f.write_str(" NX")?;
        }
        if self.uzanta {
            f.write_str(" U")?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PaĝaEraro {
    NevalidaAdreso,
    Konflikto,
    Elĉerpita,
}

impl fmt::Display for PaĝaEraro {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::NevalidaAdreso => "adreso aŭ kadro ne estas paĝ-aliniiĝinta",
            Self::Konflikto => "paĝo jam mapita aŭ grandega paĝo blokas la operacion",
            Self::Elĉerpita => "neniu libera kadro por paĝa tabelo",
        })
    }
}

pub trait PaĝaTablo {
    fn mapigi(
        &mut self,
        virtuala: VirtuaAdreso,
        fizika: FizikaAdreso,
        permesoj: PaĝaPermesoj,
        alokilo: &mut KadraAlokilo,
    ) -> Result<(), PaĝaEraro>;

    fn traduki(&self, virtuala: VirtuaAdreso) -> Option<FizikaAdreso>;

    /// La efektivaj permesoj de la folio, kiu tradukas `virtuala`; `None` se
    /// la adreso ne estas mapita. Uzata de la W^X-memtesto kaj de diagnozo.
    fn permesoj(&self, virtuala: VirtuaAdreso) -> Option<PaĝaPermesoj>;

    /// Ĉu ĉiu paĝo de `[adreso, adreso + longo)` estas atingebla de Ring 3
    /// (bitoj `U` ĉe ĉiuj niveloj) kaj, se `skriba`, ankaŭ skribebla?
    fn kontroli_uzantan(&self, adreso: u64, longo: usize, skriba: bool) -> bool;

    /// Mapu unu uzantan folion. La virtuala adreso devas kuŝi en la uzanta
    /// fenestro, kaj la bito `U` estas devigata sendepende de `permesoj`.
    fn mapi_uzantan(
        &mut self,
        virtuala: VirtuaAdreso,
        fizika: FizikaAdreso,
        permesoj: PaĝaPermesoj,
        alokilo: &mut KadraAlokilo,
    ) -> Result<(), PaĝaEraro> {
        kontroli_fenestron(virtuala.adreso(), KADRA_GRANDO as usize)?;
        let permesoj = PaĝaPermesoj {
            uzanta: true,
            ..permesoj
        };
        self.mapigi(virtuala, fizika, permesoj, alokilo)
    }
}

/// Kontrolu, ke `[adreso, adreso + longo)` kuŝas tute en la uzanta fenestro
/// sen ciferpremo. `longo == 0` estas ĉiam valida.
fn kontroli_fenestron(adreso: u64, longo: usize) -> Result<(), PaĝaEraro> {
    if longo == 0 {
        return Ok(());
    }
    let fino = adreso
        .checked_add(longo as u64)
        .ok_or(PaĝaEraro::NevalidaAdreso)?;
    if adreso < UZANTA_BAZO || fino > UZANTA_FINO {
        return Err(PaĝaEraro::NevalidaAdreso);
    }
    Ok(())
}

pub trait AdresaSpaco: PaĝaTablo {
    /// Mapu freŝajn, nuligitajn kadrojn por la uzanta intervalo `[komenco,
    /// komenco + longo)`. La limoj estas kontrolitaj antaŭ la unua aloko.
    fn mapi_uzantan_regionon(
        &mut self,
        komenco: VirtuaAdreso,
        longo: usize,
        permesoj: PaĝaPermesoj,
        alokilo: &mut KadraAlokilo,
    ) -> Result<(), PaĝaEraro> {
        kontroli_fenestron(komenco.adreso(), longo)?;
        let permesoj = PaĝaPermesoj {
            uzanta: true,
            ..permesoj
        };
        self.mapigi_freŝan_regionon(komenco, longo, permesoj, alokilo)
    }

    /// Kreu freŝan, malplenan adresan spacon: nur la radika tabelo ekzistas.
    fn krei(alokilo: &mut KadraAlokilo) -> Result<Self, PaĝaEraro>
    where
        Self: Sized;

    /// Kreu novan adresan spacon, kiu kunhavas la kernajn mapojn de `self`
    /// (erojn sen la bito `U`) kaj posedas propran, malplenan uzantan
    /// fenestron.
    ///
    /// Eroj, kiuj enhavas uzantajn mapojn aŭ kiuj enhavas parton de la uzanta
    /// fenestro, ricevas privatan subtableon; nur `U=0`-eroj estas kopiitaj.
    fn krei_kunhavantan(&self, alokilo: &mut KadraAlokilo) -> Result<Self, PaĝaEraro>
    where
        Self: Sized;

    /// La fizika adreso de la radika paĝtabelo, uzata por `CR3`.
    fn radika_adreso(&self) -> FizikaAdreso;

    /// Kiom da privataj subtabeloj la spaco posedas. Ili estis alokitaj de
    /// `krei_kunhavantan` nur por ĉi tiu spaco kaj estas liberigitaj de
    /// `detruu`; la nombro utilas por diagnozo. Sur `x86_64` temas pri
    /// PDPT-kadroj.
    fn privataj_subtabeloj(&self) -> usize;

    /// Ŝaltu la procesoran `CR3` al ĉi tiu spaco.
    fn ŝalti(&self);

    /// Malmapu la uzantajn paĝojn, liberigu la paĝtabelojn, kiuj restas
    /// malplenaj, la privatajn PDPT-ojn kaj la radikan kadron.
    ///
    /// # Antaŭkondiĉo
    /// La spaco ne rajtas esti la aktiva en `CR3`: ĝiaj mapoj malaperos.
    fn detruu(self, alokilo: &mut KadraAlokilo) -> Result<(), PaĝaEraro>
    where
        Self: Sized;

    /// Mapu la intervalon `[komenco, fino)` al la samaj fizikaj adresoj.
    fn mapigi_intervalon(
        &mut self,
        komenco: VirtuaAdreso,
        fino: VirtuaAdreso,
        permesoj: PaĝaPermesoj,
        alokilo: &mut KadraAlokilo,
    ) -> Result<(), PaĝaEraro> {
        let mut adreso = (komenco.adreso() / KADRA_GRANDO) * KADRA_GRANDO;
        let fino = fino.adreso().div_ceil(KADRA_GRANDO) * KADRA_GRANDO;
        while adreso < fino {
            let virtuala = VirtuaAdreso::nova(adreso);
            if self.traduki(virtuala).is_none() {
                self.mapigi(virtuala, FizikaAdreso::nova(adreso), permesoj, alokilo)?;
            }
            adreso += KADRA_GRANDO;
        }
        Ok(())
    }

    /// Mapu freŝajn, nuligitajn kadrojn por la intervalo `[komenco,
    /// komenco + longo)`. La virtuala adreso devas esti paĝ-aliniiĝinta.
    ///
    /// La nukleo atingu la novajn kadrojn per la identeca mapo; tio validas
    /// ĝis la per-procesaj adresspacoj de M12.
    fn mapigi_freŝan_regionon(
        &mut self,
        komenco: VirtuaAdreso,
        longo: usize,
        permesoj: PaĝaPermesoj,
        alokilo: &mut KadraAlokilo,
    ) -> Result<(), PaĝaEraro> {
        if !komenco.adreso().is_multiple_of(KADRA_GRANDO) {
            return Err(PaĝaEraro::NevalidaAdreso);
        }
        if longo == 0 {
            return Ok(());
        }

        let paĝoj = longo.div_ceil(KADRA_GRANDO as usize);
        // Kontrolu antaŭ la unua aloko, por ne lasi duonmapitan intervalon.
        for indekso in 0..paĝoj {
            let virtuala = VirtuaAdreso::nova(komenco.adreso() + indekso as u64 * KADRA_GRANDO);
            if self.traduki(virtuala).is_some() {
                return Err(PaĝaEraro::Konflikto);
            }
        }

        for indekso in 0..paĝoj {
            let kadro = alokilo.aloki().ok_or(PaĝaEraro::Elĉerpita)?;
            // La kadro estas identece mapita, do la nukleo povas nuligi ĝin.
            unsafe {
                core::ptr::write_bytes(kadro.adreso() as *mut u8, 0, KADRA_GRANDO as usize);
            }
            self.mapigi(
                VirtuaAdreso::nova(komenco.adreso() + indekso as u64 * KADRA_GRANDO),
                FizikaAdreso::nova(kadro.adreso()),
                permesoj,
                alokilo,
            )?;
        }
        Ok(())
    }

    /// Identeca mapo: la malalta 1 MiB, la kerna bildo laŭ sekciaj permesoj
    /// (W^X) kaj ĉiuj uzeblaj regionoj kiel RW+NX. La bildo estas mapita
    /// aparte, ĉar iuj praŝarĝiloj raportas ĝin kiel rezervitan.
    fn idente_mapigi(
        &mut self,
        mapo: &MemoraMapo,
        alokilo: &mut KadraAlokilo,
    ) -> Result<(), PaĝaEraro> {
        self.mapigi_intervalon(
            VirtuaAdreso::nova(0),
            VirtuaAdreso::nova(0x10_0000),
            PaĝaPermesoj::KERNA_DATUMOJ,
            alokilo,
        )?;

        let limoj = kernaj_limoj();
        self.mapigi_intervalon(
            VirtuaAdreso::nova(limoj.antaŭkodo.0),
            VirtuaAdreso::nova(limoj.antaŭkodo.1),
            PaĝaPermesoj::KERNA_LEGA,
            alokilo,
        )?;
        self.mapigi_intervalon(
            VirtuaAdreso::nova(limoj.kodo.0),
            VirtuaAdreso::nova(limoj.kodo.1),
            PaĝaPermesoj::KERNA_KODO,
            alokilo,
        )?;
        self.mapigi_intervalon(
            VirtuaAdreso::nova(limoj.rodata.0),
            VirtuaAdreso::nova(limoj.rodata.1),
            PaĝaPermesoj::KERNA_LEGA,
            alokilo,
        )?;
        self.mapigi_intervalon(
            VirtuaAdreso::nova(limoj.datumoj.0),
            VirtuaAdreso::nova(limoj.datumoj.1),
            PaĝaPermesoj::KERNA_DATUMOJ,
            alokilo,
        )?;

        for regiono in mapo.regionoj().iter().filter(|regiono| regiono.uzebla()) {
            self.mapigi_intervalon(
                VirtuaAdreso::nova(regiono.komenco),
                VirtuaAdreso::nova(regiono.fino()),
                PaĝaPermesoj::KERNA_DATUMOJ,
                alokilo,
            )?;
        }
        Ok(())
    }
}
