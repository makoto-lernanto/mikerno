//! Sistemvokaj numeroj kaj maldika `int 0x80`-envolvaĵo.
//!
//! La kerno ankoraŭ ne instalas la DPL3-pordon; ĉi tiu modulo fiksas la
//! interkonsenton, kiun `horizonto` uzos post la Ring 3-ŝalto.
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
