//! Sistemvokaj numeroj kaj maldika `int 0x80`-envolvaĵo.
//!
//! La DPL3-pordego estas instalita, do la samaj envolvaĵoj funkcias el Ring 0
//! kaj Ring 3.
//!
//! Registra ABI: `rax` = numero, `rdi`/`rsi`/`rdx`/`r10` = argumentoj,
//! `rax` = rezulto. La kerno konservas la ceterajn ĝeneralajn registreojn.

use crate::kapablo::Kapablo;
use crate::mesago::Mesago;

pub const SYS_SENDI: usize = 1;
pub const SYS_RICEVI: usize = 2;
pub const SYS_SKRIBI: usize = 3;
pub const SYS_FINI: usize = 4;
pub const SYS_CEDI: usize = 5;

/// La sekvaj numeroj estas rezervitaj per ABI-versio `2`; en la unua fazo de
/// M14 la kerno ankoraŭ respondas `-2` (nekonata voko).
pub const SYS_KREI_FINPUNKTON: usize = 6;
pub const SYS_DONI: usize = 7;
pub const SYS_REVOKI: usize = 8;
pub const SYS_RICEVI_TEMPO: usize = 9;
pub const SYS_SENDI_TEMPO: usize = 10;
pub const SYS_FERMI: usize = 11;

/// `-9`: la tenilo ne validas, la generacio malakordas aŭ la kapablo estas
/// revokita (`EBADF`).
pub const NEVALIDA_TENILO: isize = -9;
/// `-11`: nebloka vojo trovis malplenan vicon; la vokanto rajtas reprovi
/// (`EAGAIN`).
pub const PROVU_DENOVE: isize = -11;
/// `-28`: la finpunkta aŭ kapabla tabelo estas plena (`ENOSPC`).
pub const TABELO_PLENA: isize = -28;
/// `-32`: la finpunkto estis revokita aŭ ĝia posedanto finiĝis (`EPIPE`).
pub const FINPUNKTO_REVOKITA: isize = -32;
/// `-90`: la mesaĝa longaro superas `MAKSA_MESAGA_LONGO` (`EMSGSIZE`).
pub const TRO_LONGA_MESAGO: isize = -90;
/// `-110`: la tempolimo pasis antaŭ ol mesaĝo alvenis (`ETIMEDOUT`).
pub const TEMPOLIMO: isize = -110;

/// Voku la kernon; la signifo de la argumentoj dependas de `numero`.
///
/// # Safety
/// La kerno interpretas la argumentojn laŭ la koncerna voko, ofte kiel
/// uzantajn montrilojn. La vokanto respondecas, ke ili restas validaj dum la
/// voko.
#[cfg(target_arch = "x86_64")]
pub unsafe fn invoki(numero: usize, argumentoj: [usize; 4]) -> isize {
    let rezulto: isize;
    unsafe {
        core::arch::asm!(
            "int 0x80",
            inlateout("rax") numero as isize => rezulto,
            in("rdi") argumentoj[0],
            in("rsi") argumentoj[1],
            in("rdx") argumentoj[2],
            in("r10") argumentoj[3],
            options(nostack),
        );
    }
    rezulto
}

/// Sur aliaj arkitekturoj la sistemo ankoraŭ ne ekzistas.
#[cfg(not(target_arch = "x86_64"))]
pub unsafe fn invoki(_numero: usize, _argumentoj: [usize; 4]) -> isize {
    -1
}

/// Sendu mesaĝon tra la kapablo.
pub fn sendi(kapablo: Kapablo, mesago: &Mesago) -> isize {
    unsafe {
        invoki(
            SYS_SENDI,
            [
                kapablo.objekto_id as usize,
                mesago as *const Mesago as usize,
                0,
                0,
            ],
        )
    }
}

/// Ricevu mesaĝon tra la kapablo en `celo`.
pub fn ricevi(kapablo: Kapablo, celo: &mut Mesago) -> isize {
    unsafe {
        invoki(
            SYS_RICEVI,
            [
                kapablo.objekto_id as usize,
                celo as *mut Mesago as usize,
                0,
                0,
            ],
        )
    }
}

/// Skribu tekston al la kerna konzolo.
pub fn skribi(teksto: &str) -> isize {
    unsafe { invoki(SYS_SKRIBI, [teksto.as_ptr() as usize, teksto.len(), 0, 0]) }
}

/// Finu la nunan procezon kun la donita elirkodo.
pub fn fini(kodo: usize) -> isize {
    unsafe { invoki(SYS_FINI, [kodo, 0, 0, 0]) }
}

/// Cedu la procesoron al la planisto.
pub fn cedi() -> isize {
    unsafe { invoki(SYS_CEDI, [0; 4]) }
}
