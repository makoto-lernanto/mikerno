//! Finpunktoj, permesoj, kapabloj kaj teniloj.
//!
//! La mesaĝo (`Mesago`) jam venas el ĉi tiu crate; ankaŭ la permesoj, kapabloj
//! kaj la 32-bita `Tenilo` estas komuna ABI: la kerno re-exportas ilin sen
//! propra kopio, do la aranĝo ne povas malkongruiĝi.

/// Bitara aro de permesoj por finpunkto.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Permesoj(u8);

impl Permesoj {
    /// Neniu permeso; `posedas(NENIU)` ĉiam veras. Uzata por legi kapablon sen
    /// postuli rajton (ekz. `FERMI` aŭ la fonto de `DONI`).
    pub const NENIU: Self = Self(0);
    pub const SENDI: Self = Self(1);
    pub const RICEVI: Self = Self(2);
    /// Revoki la finan punkton entute; ĝi estas aparta de la sendaj kaj
    /// ricevaj permesoj, do `ĈIUJ` restas `0b011`.
    pub const REVOKI: Self = Self(4);
    /// La sendaj, ricevaj kaj revokaj permesoj kune.
    pub const PLENAJ: Self = Self(Self::SENDI.0 | Self::RICEVI.0 | Self::REVOKI.0);
    /// La kutima paro `SENDI | RICEVI`; ĝi ne inkluzivas `REVOKI`.
    pub const ĈIUJ: Self = Self(Self::SENDI.0 | Self::RICEVI.0);

    pub const fn posedas(&self, permeso: Self) -> bool {
        self.0 & permeso.0 == permeso.0
    }

    /// La krudaj permes-bitoj, uzataj sur la sistemvoka limo.
    pub const fn bitoj(&self) -> u8 {
        self.0
    }

    /// Kunmetu permesojn el krudaj bitoj; `None` se iu bito estas nekonata.
    pub const fn el_bitoj(bitoj: u8) -> Option<Self> {
        if bitoj & !Self::PLENAJ.0 != 0 {
            return None;
        }
        Some(Self(bitoj))
    }
}

/// 32-bita kapabla tenilo: 8 bitoj da indekso kaj 24 bitoj da generacio.
///
/// La generacio komenciĝas ĉe `1`, do tenilo `0` (indekso `0`, generacio `0`)
/// estas ĉiam malvalida. `nova` kaj `el_nombro` rifuzas indekson super
/// `MAKSA_INDEKSO` kaj generacion `0`; kiam generacio eluziĝas, la kerno
/// emeritigas la fendon anstataŭ rekomenci de `1` (Fazo 2).
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Tenilo(u32);

impl Tenilo {
    /// La nevalida tenilo: indekso `0` kaj generacio `0`.
    pub const MALVALIDA: Self = Self(0);
    /// La plej alta laŭleĝa fenda indekso (16 fendoj: `0..=15`).
    pub const MAKSA_INDEKSO: u8 = 15;
    /// La plej alta laŭleĝa generacio (24 bitoj).
    pub const MAKSA_GENERACIO: u32 = 0x00FF_FFFF;

    /// Kunmetu tenilon el indekso kaj generacio, aŭ `None` se iu el ili ne
    /// eniras sian kampon.
    pub const fn nova(indekso: u8, generacio: u32) -> Option<Self> {
        if indekso > Self::MAKSA_INDEKSO || generacio == 0 || generacio > Self::MAKSA_GENERACIO {
            return None;
        }
        Some(Self((generacio << 8) | indekso as u32))
    }

    /// Malpakigu krudan 32-bitan valoron, kontrolante la indekson kaj la
    /// generacion antaŭ ol redoni ĝin.
    pub const fn el_nombro(valoro: u32) -> Option<Self> {
        if (valoro & 0xFF) > Self::MAKSA_INDEKSO as u32 || valoro >> 8 == 0 {
            return None;
        }
        Some(Self(valoro))
    }

    /// La kruda 32-bita valoro, uzata sur la sistemvoka limo.
    pub const fn nombro(&self) -> u32 {
        self.0
    }

    /// La fenda indekso (`0..=15` por valida tenilo).
    pub const fn indekso(&self) -> u8 {
        (self.0 & 0xFF) as u8
    }

    /// La generacio (`1..=0x00FF_FFFF` por valida tenilo).
    pub const fn generacio(&self) -> u32 {
        self.0 >> 8
    }
}

/// La defaŭlta kapablo de ĉiu procezo: fendo `0`, unua generacio (`0x100`).
///
/// La kerno instalas ĝin ĉe la proceza registrado (`registri_fadenon`), do
/// `horizonto` povas uzi sian propran finpunkton sen antaŭa `KREI_FINPUNKTON`.
/// Ĉi tio estas la transira kapabla distribuo antaŭ `fork`/`exec` (M16), ne
/// ĝenerala API.
pub const BOOTSTRAP_TENILO: Tenilo = Tenilo(1 << 8);

/// Nomita finpunkto, de kiu oni derivas kapablon.
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct FinaPunkto {
    pub id: u64,
    pub permesoj: Permesoj,
}

impl FinaPunkto {
    pub const fn nova(id: u64, permesoj: Permesoj) -> Self {
        Self { id, permesoj }
    }

    pub const fn kapablo(&self) -> Kapablo {
        Kapablo {
            objekto_id: self.id,
            permesoj: self.permesoj,
        }
    }
}

/// Nepraĵo por uzi finpunkton: identigilo plus la permesoj de la portanto.
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Kapablo {
    pub objekto_id: u64,
    pub permesoj: Permesoj,
}
