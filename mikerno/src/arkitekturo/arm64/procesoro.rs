//! Procesora kontrolo de la `arm64`-skeleto: `WFI` kaj PSCI-haltigo.

use core::arch::asm;

/// PSCI `SYSTEM_OFF` (32-bita funkcia identigilo).
const PSCI_SYSTEM_OFF: u64 = 0x8400_0008;

/// Haltigu la maŝinon per PSCI `SYSTEM_OFF`; se la hipervizitilo rifuzas,
/// restu en `WFI`-buklo.
pub fn halti() -> ! {
    // SAFETY: `hvc #0` kun PSCI-funkcia identigilo estas la dokumentita vojo;
    // QEMU `virt` liveras la PSCI-servon per la sama kanalo.
    unsafe {
        asm!(
            "hvc #0",
            in("x0") PSCI_SYSTEM_OFF,
            options(nomem, nostack),
        );
    }
    loop {
        // SAFETY: `wfi` nur dormigas la procesoron ĝis interrompo.
        unsafe { asm!("wfi", options(nomem, nostack)) };
    }
}
