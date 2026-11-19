//! Saluto: eta uzanta bildo por la komando `lanĉi` de Ligo.
//!
//! La kerno lanĉas ĝin en Ring 3 kun `REĜIMO_EXEC`; ĝi salutas, raportas sian
//! procezan identigilon kaj finiĝas. Ĝi pruvas, ke `SYS_LANĜI` kreas novan
//! procezon el ELF-dosiero en SNDS sen tuŝi la ŝelon.

#![no_std]
#![no_main]

use core::panic::PanicInfo;

use libmikerno::syscall::{fini, skribi};

/// Enirpunkto: `reĝimo` venas en `rdi` kaj `identigilo` en `rsi` el la kerna
/// transiro (la sama interkonsento kiel `horizonto_komenco`).
#[unsafe(no_mangle)]
pub extern "C" fn saluto_komenco(_reĝimo: u64, identigilo: u64) -> ! {
    skribi("[saluto] saluton el la procezo ");
    skribi_id(identigilo);
    skribi("; ĝis revido\n");
    let _ = fini(0);
    loop {
        core::hint::spin_loop();
    }
}

fn skribi_id(valoro: u64) {
    let mut ciferoj = [0u8; 20];
    let mut longo = 0;
    let mut restanta = valoro;
    if restanta == 0 {
        ciferoj[0] = b'0';
        longo = 1;
    }
    while restanta > 0 {
        ciferoj[longo] = b'0' + (restanta % 10) as u8;
        longo += 1;
        restanta /= 10;
    }
    let mut teksto = [0u8; 20];
    for (indekso, bajto) in ciferoj[..longo].iter().rev().enumerate() {
        teksto[indekso] = *bajto;
    }
    if let Ok(teksto) = core::str::from_utf8(&teksto[..longo]) {
        skribi(teksto);
    }
}

#[panic_handler]
fn paniko(_informo: &PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
