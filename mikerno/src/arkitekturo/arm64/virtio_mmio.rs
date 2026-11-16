//! Virtio-blk per la MMIO-transporto de la QEMU-maŝino `virt` (arm64).

use crate::arkitekturo::komuna::VirtioTransporto;
use crate::memoro::kadro::KadraAlokilo;
use crate::memoro::virtuala::{AdresaSpaco, PaĝaPermesoj, VirtuaAdreso};

/// La unua fenestro de la `virt`-maŝino; 32 aparatoj po 0x200 bajtoj.
const MMIO_BAZO: u64 = 0x0a00_0000;
const MMIO_PAŜO: u64 = 0x200;
const MMIO_APARATOJ: u64 = 32;

const MAGIO: u32 = 0x7472_6976;
const VERSIO_MODERNA: u32 = 2;
const APARATO_BLOKO: u32 = 2;
const VENDOR_QEMU: u32 = 0x554d_4551;

fn legi_u32(bazo: u64, offset: u64) -> u32 {
    unsafe { core::ptr::read_volatile((bazo + offset) as *const u32) }
}

/// Trovu la unuan virtio-blk-aparaton en la MMIO-fenestro.
pub fn transporto(
    spaco: &mut dyn AdresaSpaco,
    alokilo: &mut KadraAlokilo,
) -> Result<VirtioTransporto, &'static str> {
    let fino = MMIO_BAZO + MMIO_APARATOJ * MMIO_PAŜO;
    spaco
        .mapigi_intervalon(
            VirtuaAdreso::nova(MMIO_BAZO),
            VirtuaAdreso::nova(fino),
            PaĝaPermesoj::SKRIBA_SENKAŜE,
            alokilo,
        )
        .map_err(|_| "virtio-MMIO-mapado malsukcesis")?;

    for indekso in 0..MMIO_APARATOJ {
        let bazo = MMIO_BAZO + indekso * MMIO_PAŜO;
        if legi_u32(bazo, 0x00) != MAGIO
            || legi_u32(bazo, 0x04) != VERSIO_MODERNA
            || legi_u32(bazo, 0x08) != APARATO_BLOKO
            || legi_u32(bazo, 0x0c) != VENDOR_QEMU
        {
            continue;
        }
        return Ok(VirtioTransporto::nova_mmio(bazo as *mut u8));
    }

    Err("virtio-blk ne trovita en la MMIO-fenestro")
}
