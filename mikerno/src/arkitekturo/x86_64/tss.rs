//! La Taska Stata Segmento (TSS) de x86_64.
//!
//! En longa reĝimo la TSS nur provizas `RSP0` — la kernan stakon por transiroj
//! Ring 3 → Ring 0 — kaj la IST-montrilojn. Ĉar `ltr` estas privilegia, la
//! strukturo, ĝia priskribo kaj la ŝarga instrukcio restas nur ĉi tie.

use core::arch::asm;
use core::mem::size_of;

/// GDT-indico 5 (bajtoj 0x28 kaj 0x30), post la uzantaj segmentoj.
const SELEKTILO: u16 = 0x28;
const KERNOSTAKO_GRANDE: usize = 16 * 1024;

#[repr(C, packed)]
struct TaskaStataSegmento {
    _rezervita: u32,
    kernostako_pinto: u64,
    _rsp1: u64,
    _rsp2: u64,
    _rezervita1: u64,
    _ist: [u64; 7],
    _rezervita2: u64,
    _rezervita3: u16,
    iomapbazo: u16,
}

static mut TASKA_STATO: TaskaStataSegmento = TaskaStataSegmento {
    _rezervita: 0,
    kernostako_pinto: 0,
    _rsp1: 0,
    _rsp2: 0,
    _rezervita1: 0,
    _ist: [0; 7],
    _rezervita2: 0,
    _rezervita3: 0,
    // Iomapo ekster la limo malŝaltas la I/O-permesan bitaron.
    iomapbazo: size_of::<TaskaStataSegmento>() as u16,
};

const _: () = assert!(size_of::<TaskaStataSegmento>() == 104);

#[repr(align(16))]
struct Kernostako(#[allow(dead_code)] [u8; KERNOSTAKO_GRANDE]);

static mut KERNOSTAKO: Kernostako = Kernostako([0; KERNOSTAKO_GRANDE]);

// TODO(M12): ĉiu uzanta fadeno bezonas propran kernan stakon;
// ĝisdatigi TSS.RSP0 dum kuntekstŝanĝo. Ĝis tiam: nur unu uzanta fadeno.

/// La 16-bajta longreĝima TSS-priskribo por la GDT.
pub fn priskribo() -> [u64; 2] {
    let bazo = &raw const TASKA_STATO as u64;
    let limo = (size_of::<TaskaStataSegmento>() - 1) as u64;
    let malalta = (limo & 0xFFFF)
        | ((bazo & 0xFFFF) << 16)
        | (((bazo >> 16) & 0xFF) << 32)
        | (0x89u64 << 40)
        | (((limo >> 16) & 0xF) << 48)
        | (((bazo >> 24) & 0xFF) << 56);
    [malalta, bazo >> 32]
}

/// Agordu `RSP0` al la kerna stako kaj ŝargu la taskan registron.
pub fn inicialigi_kernostakon() {
    let pinto = &raw const KERNOSTAKO as u64 + KERNOSTAKO_GRANDE as u64;
    unsafe {
        let tss = &mut *core::ptr::addr_of_mut!(TASKA_STATO);
        tss.kernostako_pinto = pinto;
        asm!(
            "ltr ax",
            in("ax") SELEKTILO,
            options(nostack, preserves_flags),
        );
    }
    crate::skribu!("[tss] TR ŝargita; kernostako pinto {:#x}\n", pinto);
}
