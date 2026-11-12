#![no_std]
#![no_main]

mod arkitekturo;
mod ekrano;
mod interrompo;
mod kerno;
mod kunhavigita;
mod memoro;

use core::panic::PanicInfo;

/// Skribu formatitan tekston al la arkitektura konzolo.
#[macro_export]
macro_rules! skribu {
    ($($argumento:tt)*) => {
        $crate::arkitekturo::skribi_formaton(format_args!($($argumento)*))
    };
}

/// Kerna enirpunkto, vokata de la praŝarĝa ŝelko en 64-bita reĝimo kun
/// interrompoj malŝaltitaj kaj la komenca GDT jam instalita.
#[unsafe(no_mangle)]
pub extern "C" fn kerna_eniro(starta_informo: *const u8, magio: u32) -> ! {
    arkitekturo::pravalorizi();

    match arkitekturo::analizi_eniron(starta_informo, magio) {
        Ok(informo) => kerno::ĉefa(informo),
        Err(eraro) => skribu!("\n[kerno] praŝarĝa eraro: {}\n", eraro),
    }

    arkitekturo::halti()
}

#[panic_handler]
fn paniko(informo: &PanicInfo) -> ! {
    skribu!("\n[kerno] paniko: {}\n", informo);
    arkitekturo::halti()
}
