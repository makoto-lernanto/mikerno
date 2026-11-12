//! Minimuma ebena tutmonda priskriba tabelo (GDT).
//!
//! La aranĝo estas elektita tiel, ke ĝi taŭgas kaj por `iretq` kaj por la
//! estonta `syscall`/`sysret`:
//!
//! - `0x08` — kerna kodo (DPL0)
//! - `0x10` — kernaj datumoj (DPL0)
//! - `0x18` — uzantaj datumoj (DPL3)
//! - `0x20` — uzanta kodo (DPL3)
//! - `0x28` — taska stata segmento (TSS), 16 bajtoj (`0x28` kaj `0x30`)
//!
//! La TSS-priskribo estas 16 bajtoj en longa reĝimo, do ĝi okupas du GDT-erojn.

use core::arch::asm;
use core::mem::size_of;

use super::tss;

#[repr(C, packed)]
struct Priskribo {
    limo: u16,
    bazo: u64,
}

#[repr(C, packed)]
struct TutmondaPriskribaTabelo {
    nulo: u64,
    kodo: u64,
    datumoj: u64,
    uzanta_datumoj: u64,
    uzanta_kodo: u64,
    tss: [u64; 2],
    priskribo: Priskribo,
}

const _: () = assert!(size_of::<TutmondaPriskribaTabelo>() == 7 * 8 + size_of::<Priskribo>());

static mut TUTMONDA_TABELO: TutmondaPriskribaTabelo = TutmondaPriskribaTabelo {
    nulo: 0,
    kodo: 0,
    datumoj: 0,
    uzanta_datumoj: 0,
    uzanta_kodo: 0,
    tss: [0; 2],
    priskribo: Priskribo { limo: 0, bazo: 0 },
};

pub fn instali() {
    unsafe {
        let gdt = &mut *core::ptr::addr_of_mut!(TUTMONDA_TABELO);

        // Ĉeesta | DPL0 | Kodo | Ne-konforma | Legi/Plenumi | L=1.
        gdt.kodo = 0x00AF_9A00_0000_0000;
        // Ĉeesta | DPL0 | Datumoj | Legi/Skribi.
        gdt.datumoj = 0x00CF_9200_0000_0000;
        // Ĉeesta | DPL3 | Datumoj | Legi/Skribi.
        gdt.uzanta_datumoj = 0x00CF_F200_0000_0000;
        // Ĉeesta | DPL3 | Kodo | Ne-konforma | Legi/Plenumi | L=1.
        gdt.uzanta_kodo = 0x00AF_FA00_0000_0000;
        gdt.tss = tss::priskribo();
        gdt.priskribo.limo =
            (size_of::<TutmondaPriskribaTabelo>() - size_of::<Priskribo>() - 1) as u16;
        gdt.priskribo.bazo = &raw const TUTMONDA_TABELO as u64;

        asm!(
            "lgdt [{ptr}]",
            // Reŝargu CS per malproksima reveno.
            "push 0x08",
            "lea rax, [rip + 2f]",
            "push rax",
            "retfq",
            "2:",
            "mov ax, 0x10",
            "mov ds, ax",
            "mov es, ax",
            "mov fs, ax",
            "mov gs, ax",
            "mov ss, ax",
            ptr = in(reg) &raw const gdt.priskribo,
            out("rax") _,
            options(nostack),
        );
    }
}
