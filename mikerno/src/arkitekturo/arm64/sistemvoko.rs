//! `svc #0`: la registra ABI kaj la elvoko el `EL1` por la Ring 0-memtesto.
//!
//! La vektora ŝelko (`interrompo`) jam dispetĉas `SVC` al la komuna
//! `kerno::sistemvoko::voki`; ĉi tiu modulo nur instalas la tabelon (denove,
//! idempote) kaj liveras la asemblan vokon uzatan de la memtesto.

/// Instalu la vektoran tabelon, kiu enhavas la `SVC`-dispetĉon.
pub fn instali() {
    super::interrompo::pravalorizi();
}

/// Voku la kernon el `EL1` per `svc #0`.
///
/// `x8` portas la numeron, `x0`–`x3` la argumentojn, `x0` la rezulton.
pub fn invoki(numero: usize, argumentoj: [u64; 4]) -> u64 {
    let rezulto: u64;
    // SAFETY: `svc #0` eniras la vektoran ŝelkon instalitan de `instali`; la
    // registra ABI kongruas kun `libmikerno::syscall`. La voko ne tuŝas la
    // stakon (`nostack`).
    unsafe {
        core::arch::asm!(
            "svc #0",
            inlateout("x0") argumentoj[0] => rezulto,
            in("x1") argumentoj[1],
            in("x2") argumentoj[2],
            in("x3") argumentoj[3],
            in("x8") numero,
            options(nostack),
        );
    }
    rezulto
}
