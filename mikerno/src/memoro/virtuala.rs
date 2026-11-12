//! Arkitekture sendependa virtuala memoro: adresoj, permesoj kaj la trait-oj,
//! per kiuj la nukleo priskribas paĝadon kaj adresajn spacojn.

use core::fmt;

use super::kadro::KadraAlokilo;
use super::{KADRA_GRANDO, MemoraMapo};

unsafe extern "C" {
    static __kerna_komenco: u8;
    static __kerna_fino: u8;
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

/// Rajtoj de unu paĝo. Legeblo ĉiam ĉeestas: la x86_64-backendo ne subtenas
/// tute nelegeblajn paĝojn.
#[derive(Clone, Copy)]
pub struct PaĝaPermesoj {
    pub skribebla: bool,
    pub plenumebla: bool,
    /// `false` por framebuffer-MMIO: la paĝo estas nekaŝebla (PCD).
    pub kaŝebla: bool,
}

impl PaĝaPermesoj {
    pub const SKRIBA: Self = Self {
        skribebla: true,
        plenumebla: false,
        kaŝebla: true,
    };

    pub const SKRIBA_SENKAŜE: Self = Self {
        skribebla: true,
        plenumebla: false,
        kaŝebla: false,
    };

    pub const KERNELA: Self = Self {
        skribebla: true,
        plenumebla: true,
        kaŝebla: true,
    };
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
}

pub trait AdresaSpaco: PaĝaTablo {
    fn krei(alokilo: &mut KadraAlokilo) -> Result<Self, PaĝaEraro>
    where
        Self: Sized;

    fn ŝalti(&self);

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

    /// Identeca mapo: la malalta 1 MiB, la kerna bildo kaj ĉiuj uzeblaj
    /// regionoj, por ke la nukleo restu atingebla post CR3. La bildo estas
    /// mapita aparte, ĉar iuj praŝarĝiloj raportas ĝin kiel rezervitan.
    fn idente_mapigi(
        &mut self,
        mapo: &MemoraMapo,
        permesoj: PaĝaPermesoj,
        alokilo: &mut KadraAlokilo,
    ) -> Result<(), PaĝaEraro> {
        self.mapigi_intervalon(
            VirtuaAdreso::nova(0),
            VirtuaAdreso::nova(0x10_0000),
            permesoj,
            alokilo,
        )?;
        self.mapigi_intervalon(
            VirtuaAdreso::nova(&raw const __kerna_komenco as u64),
            VirtuaAdreso::nova(&raw const __kerna_fino as u64),
            permesoj,
            alokilo,
        )?;

        for regiono in mapo.regionoj().iter().filter(|regiono| regiono.uzebla()) {
            self.mapigi_intervalon(
                VirtuaAdreso::nova(regiono.komenco),
                VirtuaAdreso::nova(regiono.fino()),
                permesoj,
                alokilo,
            )?;
        }
        Ok(())
    }
}
