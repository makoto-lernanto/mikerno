//! Virtio-blk per la moderna PCI-transporto (x86_64, QEMU).

use crate::arkitekturo::komuna::VirtioTransporto;
use crate::memoro::kadro::KadraAlokilo;
use crate::memoro::virtuala::{AdresaSpaco, PaĝaPermesoj, VirtuaAdreso};

use super::pci;

fn mapigi_kapon(
    spaco: &mut dyn AdresaSpaco,
    alokilo: &mut KadraAlokilo,
    bazo: u64,
    kapo: &pci::Kapo,
) -> Result<(), &'static str> {
    let komenco = bazo + kapo.offset as u64;
    let fino = komenco + kapo.longo.max(1) as u64;
    spaco
        .mapigi_intervalon(
            VirtuaAdreso::nova(komenco),
            VirtuaAdreso::nova(fino),
            PaĝaPermesoj::SKRIBA_SENKAŜE,
            alokilo,
        )
        .map_err(|_| "virtio-MMIO-mapado malsukcesis")
}

/// Trovu kaj mapu unu virtio-blk-aparaton; redonu ĝiajn registrajn adresojn.
pub fn transporto(
    spaco: &mut dyn AdresaSpaco,
    alokilo: &mut KadraAlokilo,
) -> Result<VirtioTransporto, &'static str> {
    let aparato = pci::serĉi().ok_or("virtio-blk ne trovita sur PCI-buso 0")?;
    let kapoj = pci::malfermi(&aparato)?;

    let komuna_bazo = pci::baro_bazo(&aparato, kapoj.komuna.bar)?;
    let sciiga_bazo = pci::baro_bazo(&aparato, kapoj.sciiga.bar)?;
    let aparata_bazo = pci::baro_bazo(&aparato, kapoj.aparata.bar)?;

    mapigi_kapon(spaco, alokilo, komuna_bazo, &kapoj.komuna)?;
    mapigi_kapon(spaco, alokilo, sciiga_bazo, &kapoj.sciiga)?;
    mapigi_kapon(spaco, alokilo, aparata_bazo, &kapoj.aparata)?;

    Ok(VirtioTransporto::nova_pcia(
        (komuna_bazo + kapoj.komuna.offset as u64) as *mut u8,
        (sciiga_bazo + kapoj.sciiga.offset as u64) as *mut u8,
        (aparata_bazo + kapoj.aparata.offset as u64) as *mut u8,
        kapoj.sciiga_multobligo,
    ))
}
