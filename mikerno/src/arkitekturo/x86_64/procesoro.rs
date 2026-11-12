//! Minimuma procesora kontrolo: haltigo, `RFLAGS.IF` kaj `CR2`.

use core::arch::asm;

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
