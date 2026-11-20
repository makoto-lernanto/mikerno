//! Arkitekture sendependaj virtualaj adresoj, permesoj kaj la limoj de la
//! uzanta fenestro.

use core::fmt;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct VirtuaAdreso(u64);

impl VirtuaAdreso {
    pub const fn nova(adreso: u64) -> Self {
        Self(adreso)
    }

    pub const fn adreso(&self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
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
/// La bazo de la uzanta kupo (datumsegmento), 16 MiB super la stakpinto.
pub const UZANTA_KUPO_BAZO: u64 = UZANTA_STAKO_SUPRO + 16 * 1024 * 1024;
/// La supra limo de la uzanta kupo: ĝi finiĝas antaŭ la sekreta paĝo.
pub const UZANTA_KUPO_FINO: u64 = 0x0000_0002_0000_0000;

/// Rajtoj de unu paĝo. Legeblo ĉiam ĉeestas: la x86_64-backendo ne subtenas
/// tute nelegeblajn paĝojn.
#[derive(Clone, Copy)]
pub struct PaĝaPermesoj {
    pub skribebla: bool,
    pub plenumebla: bool,
    /// `false` por frambufa MMIO: la paĝo estas nekaŝebla (PCD).
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

/// Kontrolu, ke `[adreso, adreso + longo)` kuŝas tute en la uzanta fenestro
/// sen ciferpremo. `longo == 0` estas ĉiam valida.
pub fn kontroli_fenestron(adreso: u64, longo: usize) -> Result<(), PaĝaEraro> {
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

#[cfg(test)]
mod testoj {
    use super::*;
    use crate::memoro::KADRA_GRANDO;

    #[test]
    fn adreso_konservas_la_valoron() {
        assert_eq!(VirtuaAdreso::nova(0x1234).adreso(), 0x1234);
        assert_eq!(FizikaAdreso::nova(0x4000).adreso(), 0x4000);
    }

    #[test]
    fn virtua_ordo_estas_nombra() {
        assert!(VirtuaAdreso::nova(1) < VirtuaAdreso::nova(2));
        assert_eq!(VirtuaAdreso::nova(7), VirtuaAdreso::nova(7));
        assert_ne!(FizikaAdreso::nova(7), FizikaAdreso::nova(8));
    }

    #[test]
    fn fenestro_akceptas_internan_intervalon() {
        assert_eq!(kontroli_fenestron(UZANTA_BAZO + 0x1000, 0x1000), Ok(()));
    }

    #[test]
    fn fenestro_akceptas_ekzaktan_finon() {
        assert_eq!(
            kontroli_fenestron(UZANTA_FINO - KADRA_GRANDO, KADRA_GRANDO as usize),
            Ok(())
        );
    }

    #[test]
    fn fenestro_rifuzas_sub_la_bazon() {
        assert_eq!(
            kontroli_fenestron(UZANTA_BAZO - 1, 1),
            Err(PaĝaEraro::NevalidaAdreso)
        );
    }

    #[test]
    fn fenestro_rifuzas_super_la_finon() {
        assert_eq!(
            kontroli_fenestron(UZANTA_FINO, 1),
            Err(PaĝaEraro::NevalidaAdreso)
        );
        assert_eq!(
            kontroli_fenestron(UZANTA_BAZO, (UZANTA_FINO - UZANTA_BAZO) as usize + 1),
            Err(PaĝaEraro::NevalidaAdreso)
        );
    }

    #[test]
    fn fenestro_rifuzas_ciferpremon() {
        assert_eq!(
            kontroli_fenestron(u64::MAX - 8, 16),
            Err(PaĝaEraro::NevalidaAdreso)
        );
    }

    #[test]
    fn fenestro_akceptas_nulan_longon() {
        assert_eq!(kontroli_fenestron(UZANTA_FINO + 1, 0), Ok(()));
        assert_eq!(kontroli_fenestron(0, 0), Ok(()));
    }

    #[test]
    fn permesoj_havas_mallongan_formon() {
        assert_eq!(PaĝaPermesoj::KERNA_LEGA.to_string(), "R NX");
        assert_eq!(PaĝaPermesoj::KERNA_DATUMOJ.to_string(), "RW NX");
        assert_eq!(PaĝaPermesoj::KERNA_KODO.to_string(), "RX");
        assert_eq!(PaĝaPermesoj::UZANTA_KODO.to_string(), "RX U");
    }
}
