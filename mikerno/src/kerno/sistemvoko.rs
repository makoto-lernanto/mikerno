//! Sistemvokoj: la `int 0x80`-interkonsento de la kerno.
//!
//! Registra ABI: `rax` = numero, `rdi`/`rsi`/`rdx`/`r10` = argumentoj,
//! `rax` = rezulto. La enira ŝelko jam konservis la registrojn en
//! `InterrompaKadro`, do la dispetĉilo nur legas kaj skribas la kadron.

use crate::interrompo::{InterrompaKadro, InterrompaTraktilo, SISTEMVOKA_VEKTORO};

pub const NUMERO_SENDI: u64 = 1;
pub const NUMERO_RICEVI: u64 = 2;
pub const NUMERO_SKRIBI: u64 = 3;
pub const NUMERO_FINI: u64 = 4;
pub const NUMERO_CEDI: u64 = 5;

/// `-ENOSYS`: la numero estas konata, sed la voko ankoraŭ ne efektiviĝis.
const NE_EFEKTIVIGITA: i64 = -38;
/// `-EINVAL`: la numero ne apartenas al la interkonsento.
const NEKONATA_NUMERO: i64 = -22;

struct SistemvokaTraktilo;

impl InterrompaTraktilo for SistemvokaTraktilo {
    fn trakta(&self, kadro: &mut InterrompaKadro) {
        let numero = kadro.rax;
        let argumentoj = [kadro.rdi, kadro.rsi, kadro.rdx, kadro.r10];
        kadro.rax = dispetchi(numero, argumentoj) as u64;
    }
}

fn dispetchi(numero: u64, _argumentoj: [u64; 4]) -> i64 {
    match numero {
        NUMERO_SENDI | NUMERO_RICEVI | NUMERO_SKRIBI | NUMERO_FINI | NUMERO_CEDI => NE_EFEKTIVIGITA,
        _ => NEKONATA_NUMERO,
    }
}

static TRAKTILO: SistemvokaTraktilo = SistemvokaTraktilo;

pub fn pravalorizi() {
    crate::interrompo::registri(SISTEMVOKA_VEKTORO, &TRAKTILO);
    crate::skribu!("[sistemvoko] pordego 0x80 (DPL3) instalita\n");
}

/// Voku la pordegon ankoraŭ el Ring 0, por pruvi la kadran vojon.
pub fn testi_pordegon() {
    let konata = invoki(NUMERO_SKRIBI, [0; 4]);
    let nekonata = invoki(0x7FFF, [0; 4]);
    crate::skribu!(
        "[sistemvoko] memtesto: konata={}, nekonata={}\n",
        konata,
        nekonata
    );
}

fn invoki(numero: u64, argumentoj: [u64; 4]) -> i64 {
    let rezulto: i64;
    unsafe {
        core::arch::asm!(
            "int 0x80",
            inlateout("rax") numero as i64 => rezulto,
            in("rdi") argumentoj[0],
            in("rsi") argumentoj[1],
            in("rdx") argumentoj[2],
            in("r10") argumentoj[3],
            options(nostack),
        );
    }
    rezulto
}
