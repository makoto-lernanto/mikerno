//! Horizonto: la unua uzanta procezo (Ring 3).
//!
//! La bildo ankoraŭ ne estas enŝovita en la kernan bildon; ĝi fiksas la
//! enirpunkton, la vokprotokolon kaj la vivociklon por la venonta Ring 3-ŝalto.

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
