//! Sistemvokaj numeroj kaj maldikaj envolvaĵoj.
//!
//! Sur `x86_64` la voko trapasas la DPL3-pordegon `int 0x80`; sur `arm64` ĝi
//! uzas `svc #0`. La sama bildo funkcias el Ring 0 kaj Ring 3.
//!
//! Registra ABI sur `x86_64`: `rax` = numero, `rdi`/`rsi`/`rdx`/`r10` =
//! argumentoj, `rax` = rezulto. Sur `arm64`: `x8` = numero, `x0`–`x3` =
//! argumentoj, `x0` = rezulto. La kerno konservas la ceterajn ĝeneralajn
//! registreojn.

use crate::kapablo::{Permesoj, Tenilo};
use crate::mesago::Mesago;

pub const SYS_SENDI: usize = 1;
pub const SYS_RICEVI: usize = 2;
pub const SYS_SKRIBI: usize = 3;
pub const SYS_FINI: usize = 4;
pub const SYS_CEDI: usize = 5;

/// La sekvaj numeroj apartenas al ABI-versio `2`. `KREI_FINPUNKTON`, `DONI`,
/// `REVOKI` kaj `FERMI` estas efektivaj ekde la Fazoj 2–4 de M14; la tempaj
/// vokoj ankoraŭ respondas `-2` (nekonata voko) ĝis la Fazoj 5–6.
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

/// Sur `arm64` la voko trapasas `svc #0`; la numero estas en `x8`.
///
/// # Safety
/// La kerno interpretas la argumentojn laŭ la koncerna voko, ofte kiel
/// uzantajn montrilojn. La vokanto respondecas, ke ili restas validaj dum la
/// voko.
#[cfg(target_arch = "aarch64")]
pub unsafe fn invoki(numero: usize, argumentoj: [usize; 4]) -> isize {
    let rezulto: isize;
    unsafe {
        core::arch::asm!(
            "svc #0",
            inlateout("x0") argumentoj[0] as isize => rezulto,
            in("x1") argumentoj[1],
            in("x2") argumentoj[2],
            in("x3") argumentoj[3],
            in("x8") numero,
            options(nostack),
        );
    }
    rezulto
}

/// Sur aliaj arkitekturoj la sistemo ankoraŭ ne ekzistas.
#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
pub unsafe fn invoki(_numero: usize, _argumentoj: [usize; 4]) -> isize {
    -1
}

/// Sendu mesaĝon al la finpunkto de la kapablo `tenilo`.
pub fn sendi(tenilo: Tenilo, mesago: &Mesago) -> isize {
    unsafe {
        invoki(
            SYS_SENDI,
            [
                tenilo.nombro() as usize,
                mesago as *const Mesago as usize,
                0,
                0,
            ],
        )
    }
}

/// Ricevu mesaĝon el la finpunkto de la kapablo `tenilo` en `celo`.
pub fn ricevi(tenilo: Tenilo, celo: &mut Mesago) -> isize {
    unsafe {
        invoki(
            SYS_RICEVI,
            [tenilo.nombro() as usize, celo as *mut Mesago as usize, 0, 0],
        )
    }
}

/// Donu la kapablon `fonto` al la procezo `celo`, kun la permesoj `permesoj`.
///
/// La petitaj permesoj devas esti subaro de la fontaj; alie la voko malakceptas
/// per `-1`. Sukceso redonas la tenilon en la celo-tabelo.
pub fn doni(fonto: Tenilo, celo: u64, permesoj: Permesoj) -> isize {
    unsafe {
        invoki(
            SYS_DONI,
            [
                fonto.nombro() as usize,
                celo as usize,
                permesoj.bitoj() as usize,
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

/// Kreu finpunkton kun la donitaj permesoj; sukceso redonas la krudan
/// `Tenilo`-nombron (pozitiva), alie negativan eraron.
pub fn krei_finpunkton(permesoj: Permesoj) -> isize {
    unsafe { invoki(SYS_KREI_FINPUNKTON, [permesoj.bitoj() as usize, 0, 0, 0]) }
}

/// Fermu la lokan tenilon de finpunkto; `0` aŭ negativa eraro.
pub fn fermi(tenilo: Tenilo) -> isize {
    unsafe { invoki(SYS_FERMI, [tenilo.nombro() as usize, 0, 0, 0]) }
}

/// Revoku la tutan finpunkton: ĉiuj teniloj malvalidiĝas kaj blokita
/// ricevanto vekiĝas per `FINPUNKTO_REVOKITA`. `0` aŭ negativa eraro; la dua
/// argumento estas rezervita kaj devas esti `0`.
pub fn revoki(tenilo: Tenilo) -> isize {
    unsafe { invoki(SYS_REVOKI, [tenilo.nombro() as usize, 0, 0, 0]) }
}
