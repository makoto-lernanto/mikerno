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

/// Unua kerna enirpunkto, vokata de la praŝarĝa ŝelko en 64-bita reĝimo kun
/// interrompoj malŝaltitaj kaj la komenca GDT jam instalita.
///
/// Ĝi provas la KASLR-kopion kaj salton; se tio ne eblas, ĝi mem vokas la duan
/// enirpunkton ĉe la liga bazo. Ĝi neniam revenas al la ŝelko.
#[unsafe(no_mangle)]
pub extern "C" fn kerna_eniro(starta_informo: *const u8, magio: u32) -> ! {
    arkitekturo::hazardigi(starta_informo, magio)
}

/// Dua kerna enirpunkto: la bildo jam loĝas ĉe `bazo` (la liga bazo, se KASLR
/// malsukcesis). `origino` estas la adreso, de kiu la bildo estis kopiita.
#[unsafe(no_mangle)]
pub extern "C" fn kerna_eniro_kopio(
    starta_informo: *const u8,
    magio: u32,
    bazo: u64,
    origino: u64,
) -> ! {
    arkitekturo::pravalorizi();

    match arkitekturo::analizi_eniron(starta_informo, magio) {
        Ok(informo) => kerno::ĉefa(informo, bazo, origino),
        Err(eraro) => skribu!("\n[kerno] praŝarĝa eraro: {}\n", eraro),
    }

    arkitekturo::halti()
}

#[panic_handler]
fn paniko(informo: &PanicInfo) -> ! {
    skribu!("\n[kerno] paniko: {}\n", informo);
    arkitekturo::halti()
}
