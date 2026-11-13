//! Minimuma procesora kontrolo: haltigo, `RFLAGS.IF`, `CR2` kaj CPUID-trajtoj.

use core::arch::asm;

/// La procesoraj trajtoj, kiujn la kerna firmigo bezonas.
#[derive(Clone, Copy)]
pub struct Trajtoj {
    pub smep: bool,
    pub smap: bool,
}

/// Legu `SMEP` kaj `SMAP` el CPUID (folio 7, subfolio 0).
///
/// `SMEP` estas bito 7 kaj `SMAP` bito 20 de `EBX`; `CPUID.7` ekzistas nur se
/// la maksimuma baza folio atingas 7, do tio estas kontrolita unue.
pub fn trajtoj() -> Trajtoj {
    let (maks, _, _, _) = cpuid(0, 0);
    if maks < 7 {
        return Trajtoj {
            smep: false,
            smap: false,
        };
    }
    let (_, ebx, _, _) = cpuid(7, 0);
    Trajtoj {
        smep: ebx & (1 << 7) != 0,
        smap: ebx & (1 << 20) != 0,
    }
}

fn cpuid(folio: u32, subfolio: u32) -> (u32, u32, u32, u32) {
    let (mut eax, mut ebx, mut ecx, mut edx) = (folio, 0, subfolio, 0);
    // SAFETY: `cpuid` nur legas procesorajn identigilojn. `RBX` estas
    // konservata mane, ĉar LLVM rezervas ĝin kaj malakceptas ĝin kiel
    // operandon.
    unsafe {
        asm!(
            "push rbx",
            "cpuid",
            "mov {eligo:e}, ebx",
            "pop rbx",
            inout("eax") eax,
            inout("ecx") ecx,
            out("edx") edx,
            eligo = out(reg) ebx,
            options(nomem, preserves_flags),
        );
    }
    (eax, ebx, ecx, edx)
}

pub fn halti() -> ! {
    loop {
        unsafe { asm!("hlt", options(nomem, nostack, preserves_flags)) };
    }
}

/// Atendu unu interrompon; la CPU vekiĝas ĉe la sekva `hlt`-okazo.
pub fn atendi_interrompon() {
    unsafe { asm!("hlt", options(nomem, nostack, preserves_flags)) };
}

pub fn ebligi_interrompojn() {
    unsafe { asm!("sti", options(nomem, nostack)) };
}

pub fn malebligi_interrompojn() {
    unsafe { asm!("cli", options(nomem, nostack)) };
}

pub fn interrompoj_ebligitaj() -> bool {
    let flagoj: u64;
    unsafe {
        asm!(
            "pushfq",
            "pop {}",
            out(reg) flagoj,
            options(preserves_flags),
        );
    }
    flagoj & (1 << 9) != 0
}

/// La adreso, kiu kaŭzis la lastan paĝan eraron (`CR2`).
pub fn fuŝa_adreso() -> u64 {
    let adreso: u64;
    unsafe {
        asm!(
            "mov {}, cr2",
            out(reg) adreso,
            options(nomem, nostack, preserves_flags),
        );
    }
    adreso
}
