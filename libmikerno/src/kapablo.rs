//! Finpunktoj, permesoj kaj kapabloj.
//!
//! La mesaĝo (`Mesago`) jam venas el ĉi tiu crate; la kapabloj kaj permesoj
//! ankoraŭ spegulas `mikerno::kerno::ipc` kaj migros en M13.

/// Bitara aro de permesoj por finpunkto.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Permesoj(u8);

impl Permesoj {
    pub const SENDI: Self = Self(1);
    pub const RICEVI: Self = Self(2);
    pub const ĈIUJ: Self = Self(Self::SENDI.0 | Self::RICEVI.0);

    pub const fn posedas(&self, permeso: Self) -> bool {
        self.0 & permeso.0 == permeso.0
    }
}

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
