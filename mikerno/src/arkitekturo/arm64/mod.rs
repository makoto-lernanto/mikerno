//! arm64-malantaŭo: A1-skeleto (seria konzolo, FDT, identeca MMU, PSCI).

pub mod eniro;
pub mod fdt;
pub mod pagxado;
pub mod procesoro;
pub mod seria;

/// La unua Rust-funkcio post la asembla eniro; `dtb` venas el la praŝarĝilo.
#[unsafe(no_mangle)]
pub extern "C" fn kerna_eniro_arm64(dtb: *const u8) -> ! {
    seria::pravalorizi(seria::DEFAŬLTA);

    let fdt = fdt::Fdt::nova(dtb);
    let seria_adreso = fdt
        .as_ref()
        .and_then(|fdt| fdt.seria_adreso())
        .map(|adreso| adreso as usize)
        .unwrap_or(seria::DEFAŬLTA);
    if seria_adreso != seria::DEFAŬLTA {
        seria::pravalorizi(seria_adreso);
    }

    crate::skribu!(
        "[MiKerno] Saluton, mi estas MiKerno v{}!\n",
        env!("CARGO_PKG_VERSION")
    );

    let mut regionoj = 0;
    let mut uzebla = 0u64;
    if let Some(fdt) = fdt.as_ref() {
        regionoj = fdt.memoro().len();
        uzebla = fdt.memoro().iter().map(|regiono| regiono.longo).sum();
        if let Some((komenco, fino)) = fdt.initrd() {
            crate::skribu!("[kerno] initrd: 0x{:x}..0x{:x}\n", komenco, fino);
        }
    } else {
        crate::skribu!("[kerno] averto: FDT ne estas valida; uzata estas la fiksa PL011\n");
    }
    crate::skribu!(
        "[kerno] memoro: {} regionoj, {} MiB uzeblaj\n",
        regionoj,
        uzebla / (1024 * 1024)
    );

    pagxado::pravalorizi();
    crate::skribu!("[kerno] seria konzolo: PL011 @ 0x{:x}\n", seria_adreso);

    procesoro::halti()
}
