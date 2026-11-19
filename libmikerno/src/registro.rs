//! Tipaj registroj, kiujn sistemvokoj kopiis inter Ring 0 kaj Ring 3.
//!
//! La aranĝo estas `#[repr(C)]` kaj ne portas internajmontrilojn, do ambaŭ
//! flankoj rajtas kopii ĝin sen konverto. La longoj estas fiksaj; nomoj estas
//! UTF-8 kun fina NUL en la neuzata parto.

/// Maksimuma nombro da registroj en unu voko.
pub const MAKS_REGISTROJ: usize = 16;
/// Longo de dosiernomo en `Dosierero`/`ProcezoRekordo` (sen fina NUL: 59).
pub const NOMA_LONGO: usize = 60;
/// Longo de la permesa priskribo en `ProcezoRekordo`.
pub const PERMESOJ_LONGO: usize = 16;

/// Dosiera tipo, kiel en SNDS.
pub const TIPO_DOSIERO: u8 = 1;
/// Dosieruja tipo, kiel en SNDS.
pub const TIPO_DOSIERUJO: u8 = 2;

/// Proceza stato, kiel en la kerna proceza tabelo.
pub const STATO_PRETA: u8 = 1;
/// Procezo nun plenumas aŭ atendas en la preta vico.
pub const STATO_KURANTA: u8 = 2;
/// Procezo jam finis; la ero ne plu aperas en la tabelo.
pub const STATO_MORTINTA: u8 = 3;

/// Unu linio de `listi`: objekto, ne teksta linio.
///
/// `inodo` estas la SNDS-inoda indekso. Ĝi estas uzata kiel tenilo ĝis la
/// dosiersistemo ekhavos verajn kapabilajn dosier-tenilojn.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Dosierero {
    /// Dosiernomo; UTF-8 kun fina NUL.
    pub nomo: [u8; NOMA_LONGO],
    /// `TIPO_DOSIERO` aŭ `TIPO_DOSIERUJO`.
    pub tipo: u8,
    /// Noto en bajto, por eviti interna plenigo antaŭ `grandeco`.
    pub rezervita0: u8,
    /// `grando` de la unuo; dosierujoj havas la memgrandon de la dosierujo.
    pub grandeco: u32,
    /// SNDS-inoda indekso; nun uzata kiel `tenilo`-valoro.
    pub inodo: u32,
}

/// Unu linio de `procezoj`: objekto, ne teksta linio.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ProcezoRekordo {
    /// Proceza identigilo (la proceza tabelo), ne la fadena identigilo.
    pub pid: u64,
    /// `STATO_PRETA`, `STATO_KURANTA` aŭ `STATO_MORTINTA`.
    pub stato: u8,
    /// Interna plenigo, fikse `0`.
    pub rezervita0: [u8; 7],
    /// Uzo de la kupo en bajtoj (`kupo_fino - UZANTA_KUPO_BAZO`).
    pub memoro: u64,
    /// Permesa priskribo; `NUL` ĉe la unua bajto signifas «neniu priskribo».
    pub permesoj: [u8; PERMESOJ_LONGO],
    /// Proceza nomo; UTF-8 kun fina NUL.
    pub nomo: [u8; NOMA_LONGO],
}

impl Dosierero {
    pub const fn nova() -> Self {
        Self {
            nomo: [0; NOMA_LONGO],
            tipo: 0,
            rezervita0: 0,
            grandeco: 0,
            inodo: 0,
        }
    }
}

impl ProcezoRekordo {
    pub const fn nova() -> Self {
        Self {
            pid: 0,
            stato: 0,
            rezervita0: [0; 7],
            memoro: 0,
            permesoj: [0; PERMESOJ_LONGO],
            nomo: [0; NOMA_LONGO],
        }
    }
}

const _: () = {
    assert!(core::mem::size_of::<Dosierero>() == 72);
    assert!(core::mem::size_of::<ProcezoRekordo>() == 104);
};
