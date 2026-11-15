//! Kernaj objektoj: fadeno, ĝia stato kaj la konservita kunteksto.

use crate::arkitekturo::Kunteksto;
use crate::memoro::virtuala::VirtuaAdreso;

pub type ProcezaId = u64;
pub type FadenoId = u64;
pub type FadenoEnirpunkto = fn(FadenoId);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum FadenoStato {
    Preta,
    Kuranta,
    Blokitaj,
    Mortinta,
}

#[derive(Clone, Copy)]
pub struct Fadeno {
    pub id: FadenoId,
    /// La procezo, al kiu la fadeno apartenas; `0` estas la nukleo mem.
    pub procezo: ProcezaId,
    pub stako_pinto: VirtuaAdreso,
    pub stato: FadenoStato,
    pub konteksto: Kunteksto,
    pub enirpunkto: Option<FadenoEnirpunkto>,
}

impl Fadeno {
    pub const fn nova(
        id: FadenoId,
        procezo: ProcezaId,
        stako_pinto: VirtuaAdreso,
        enirpunkto: FadenoEnirpunkto,
    ) -> Self {
        Self {
            id,
            procezo,
            stako_pinto,
            stato: FadenoStato::Preta,
            konteksto: Kunteksto::nova(),
            enirpunkto: Some(enirpunkto),
        }
    }

    /// Id 0 portas la praŝarĝan kuntekston, al kiu la planisto revenas.
    pub const fn ripoza() -> Self {
        Self {
            id: 0,
            procezo: 0,
            stako_pinto: VirtuaAdreso::nova(0),
            stato: FadenoStato::Kuranta,
            konteksto: Kunteksto::nova(),
            enirpunkto: None,
        }
    }
}
