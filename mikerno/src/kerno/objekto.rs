//! Kernaj objektoj: fadeno, ĝia stato kaj la konservita kunteksto.

use crate::memoro::virtuala::VirtuaAdreso;

use super::procezo::ProcezaId;

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

/// La aranĝo estas fiksa (`#[repr(C)]`): la muntita ŝanĝo uzas fiksajn
/// deŝovojn.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Kunteksto {
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rbp: u64,
    pub rsp: u64,
    pub rip: u64,
    pub rflags: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
}

impl Kunteksto {
    pub const fn nova() -> Self {
        Self {
            rax: 0,
            rbx: 0,
            rcx: 0,
            rdx: 0,
            rsi: 0,
            rdi: 0,
            rbp: 0,
            rsp: 0,
            rip: 0,
            rflags: 0,
            r8: 0,
            r9: 0,
            r10: 0,
            r11: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,
        }
    }
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
