//! Horizonto: la unua uzanta procezo (Ring 3).
//!
//! La kerno liveras ĉi tiun bildon kiel praŝarĝan modulon, muntas ĝin super
//! 4 GiB kaj transiras al `horizonto_start` per `iretq`. La vokprotokolo estas
//! `int 0x80` kun la registra aranĝo el `libmikerno`.

#![no_std]
#![no_main]

mod biblioteko;

use core::panic::PanicInfo;

use libmikerno::starto::REĜIMO_KRUI;
use libmikerno::syscall::{SYS_SKRIBI, invoki};

const SALUTO: &str = "horizonto: la unua procezo vekis en Ring 3.\n";

/// Enirpunkto de la unua uzanta procezo. `reĝimo` venas en `rdi` el la kerna
/// `iretq`-transiro.
#[unsafe(no_mangle)]
pub extern "C" fn horizonto_start(reĝimo: u64) -> ! {
    biblioteko::skribi(SALUTO);
    testi_negativajn();
    if reĝimo == REĜIMO_KRUI {
        provoki_paĝan_eraron();
    }
    // Ĉi tie la estonteco inicialigos la uzantajn servojn.
    biblioteko::fini(0)
}

/// Kontrolu, ke la kerno rifuzas malvalidajn Ring 3-argumentojn anstataŭ
/// dereferenci ilin.
fn testi_negativajn() {
    // SAFETY: la argumentoj estas intence malvalidaj; la kerna dispetĉilo
    // validigas montrilojn kaj longojn antaŭ ol legi aŭ skribi ion ajn.
    let (nekonata, nula, kerna, troa) = unsafe {
        (
            invoki(0x7FFF, [0; 4]),
            invoki(SYS_SKRIBI, [0, 8, 0, 0]),
            invoki(SYS_SKRIBI, [0x1000, 8, 0, 0]),
            invoki(SYS_SKRIBI, [SALUTO.as_ptr() as usize, 4097, 0, 0]),
        )
    };
    if nekonata == -2 && nula == -1 && kerna == -1 && troa == -1 {
        biblioteko::skribi("[ring3] negativaj: nekonata=-2, nula=-1, kerna=-1, troa=-1\n");
    } else {
        biblioteko::skribi("[ring3] negativaj: MALBONE\n");
    }
}

/// Legu intence nemapitan adreson. La kerna `#PF`-traktilo rekonas Ring 3,
/// mortigas la procezon kaj daŭrigas; la voko do normale ne revenas.
fn provoki_paĝan_eraron() -> ! {
    const NEMAPITA: u64 = 0xDEAD_BEEF;
    biblioteko::skribi("[ring3] provokas paĝan eraron ĉe 0xdeadbeef\n");
    let valoro: u64;
    // SAFETY: la adreso estas intence nemapita; la celo estas kaŭzi `#PF` el
    // Ring 3, kiun la kerno kaptas kaj mortigas la procezon.
    unsafe {
        core::arch::asm!(
            "mov {}, [{}]",
            out(reg) valoro,
            in(reg) NEMAPITA,
            options(nostack, preserves_flags, readonly),
        );
    }
    core::hint::black_box(valoro);
    biblioteko::skribi("[ring3] eraro: la legado ne kaŭzis paĝan eraron\n");
    biblioteko::fini(1)
}

#[panic_handler]
fn paniko(_informo: &PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
