//! Horizonto: la unua uzanta procezo (Ring 3).
//!
//! La kerno liveras ĉi tiun bildon kiel praŝarĝan modulon, muntas ĝin super
//! 4 GiB kaj transiras al `horizonto_start` per `iretq`. La vokprotokolo estas
//! `int 0x80` kun la registra aranĝo el `libmikerno`.

#![no_std]
#![no_main]

mod biblioteko;

use core::panic::PanicInfo;

/// Enirpunkto de la unua uzanta procezo.
#[unsafe(no_mangle)]
pub extern "C" fn horizonto_start() -> ! {
    biblioteko::skribi("horizonto: la unua procezo vekis en Ring 3.\n");
    // Ĉi tie la estonteco inicialigos la uzantajn servojn.
    biblioteko::fini(0)
}

#[panic_handler]
fn paniko(_informo: &PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
