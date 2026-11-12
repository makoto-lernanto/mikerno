//! Horizonto: la unua uzanta procezo (Ring 3).
//!
//! La kerno liveras ĉi tiun bildon kiel praŝarĝan modulon, muntas ĝin super
//! 4 GiB kaj transiras al `horizonto_start` per `iretq`. La vokprotokolo estas
//! `int 0x80` kun la registra aranĝo el `libmikerno`.

#![no_std]
#![no_main]

mod biblioteko;

use core::panic::PanicInfo;

use libmikerno::syscall::{SYS_SKRIBI, invoki};

const SALUTO: &str = "horizonto: la unua procezo vekis en Ring 3.\n";

/// Enirpunkto de la unua uzanta procezo.
#[unsafe(no_mangle)]
pub extern "C" fn horizonto_start() -> ! {
    biblioteko::skribi(SALUTO);
    testi_negativajn();
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

#[panic_handler]
fn paniko(_informo: &PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
