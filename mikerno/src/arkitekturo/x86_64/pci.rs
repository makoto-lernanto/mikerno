//! Minimuma PCI-konfigura spaco por la virtio-blk-dukto (QEMU).
//!
//! `legi` kaj `skribi` uzas la klasikajn pordojn 0xCF8/0xCFC. La serĉo
//! limiĝas al la unua buso; la testaj QEMU-maŝinoj metas la aparaton tie.

use core::arch::asm;

const KONFIGURA_ADRESO: u16 = 0xCF8;
const KONFIGURAJ_DATUMOJ: u16 = 0xCFC;

/// Vendor ID de virtio.
pub const VIRTIO_VENDOR: u16 = 0x1AF4;
/// Moderna virtio-blk (virtio 1.0, nur modernaj kapabloj).
pub const VIRTIO_BLK_MODERNA: u16 = 0x1042;
/// Transira virtio-blk; moderna kapablolisto ankoraŭ povas ekzisti.
pub const VIRTIO_BLK_TRANSIRA: u16 = 0x1001;

const KAPABLO_VIRTIO: u8 = 0x09;
const KAPABLO_KOMUNA: u8 = 1;
const KAPABLO_SCIIGA: u8 = 2;
const KAPABLO_APARATA: u8 = 4;

/// Unu trovita PCI-funkcio.
#[derive(Clone, Copy)]
pub struct Aparato {
    bus: u8,
    slot: u8,
    func: u8,
}

/// Unu virtio-kapablo ene de BAR.
#[derive(Clone, Copy)]
pub struct Kapo {
    pub bar: u8,
    pub offset: u32,
    pub longo: u32,
}

/// La kapabloj necesaj por la modernaj registroj kaj la sciiga pordo.
#[derive(Clone, Copy)]
pub struct Kapoj {
    pub komuna: Kapo,
    pub sciiga: Kapo,
    pub aparata: Kapo,
    pub sciiga_multobligo: u32,
}

fn legi_dvardon(bus: u8, slot: u8, func: u8, offset: u8) -> u32 {
    let adreso = 1u32 << 31
        | ((bus as u32) & 0xff) << 16
        | ((slot as u32) & 0x1f) << 11
        | ((func as u32) & 0x07) << 8
        | (offset as u32) & 0xfc;
    unsafe {
        asm!(
            "out dx, eax",
            in("dx") KONFIGURA_ADRESO,
            in("eax") adreso,
            options(nomem, nostack, preserves_flags),
        );
        let valoro: u32;
        asm!(
            "in eax, dx",
            in("dx") KONFIGURAJ_DATUMOJ,
            out("eax") valoro,
            options(nomem, nostack, preserves_flags),
        );
        valoro
    }
}

fn skribi_dvardon(bus: u8, slot: u8, func: u8, offset: u8, valoro: u32) {
    let adreso = 1u32 << 31
        | ((bus as u32) & 0xff) << 16
        | ((slot as u32) & 0x1f) << 11
        | ((func as u32) & 0x07) << 8
        | (offset as u32) & 0xfc;
    unsafe {
        asm!(
            "out dx, eax",
            in("dx") KONFIGURA_ADRESO,
            in("eax") adreso,
            options(nomem, nostack, preserves_flags),
        );
        asm!(
            "out dx, eax",
            in("dx") KONFIGURAJ_DATUMOJ,
            in("eax") valoro,
            options(nomem, nostack, preserves_flags),
        );
    }
}

fn legi_bajton(bus: u8, slot: u8, func: u8, offset: u8) -> u8 {
    (legi_dvardon(bus, slot, func, offset & 0xfc) >> ((offset & 3) * 8)) as u8
}

fn legi_vorton(bus: u8, slot: u8, func: u8, offset: u8) -> u16 {
    (legi_dvardon(bus, slot, func, offset & 0xfc) >> ((offset & 2) * 8)) as u16
}

fn skribi_vorton(bus: u8, slot: u8, func: u8, offset: u8, valoro: u16) {
    let antaŭe = legi_dvardon(bus, slot, func, offset & 0xfc);
    let sxovo = (offset & 2) * 8;
    let masko = !(0xffffu32 << sxovo);
    skribi_dvardon(
        bus,
        slot,
        func,
        offset & 0xfc,
        (antaŭe & masko) | ((valoro as u32) << sxovo),
    );
}

/// Serĉu la unuan virtio-blk-funkcion sur buso 0.
pub fn serĉi() -> Option<Aparato> {
    for slot in 0..32u8 {
        for func in 0..8u8 {
            let vendor = legi_vorton(0, slot, func, 0);
            if vendor == 0xffff || vendor != VIRTIO_VENDOR {
                continue;
            }
            let aparato = legi_vorton(0, slot, func, 2);
            if aparato == VIRTIO_BLK_MODERNA || aparato == VIRTIO_BLK_TRANSIRA {
                return Some(Aparato { bus: 0, slot, func });
            }
        }
    }
    None
}

/// Ŝaltu memoran kaj busan mastrojn, poste legu la virtio-kapablojn.
pub fn malfermi(aparato: &Aparato) -> Result<Kapoj, &'static str> {
    let komando = legi_vorton(aparato.bus, aparato.slot, aparato.func, 4);
    skribi_vorton(
        aparato.bus,
        aparato.slot,
        aparato.func,
        4,
        komando | (1 << 1) | (1 << 2),
    );

    let status = legi_vorton(aparato.bus, aparato.slot, aparato.func, 6);
    if status & (1 << 4) == 0 {
        return Err("PCI-kapablolisto mankas");
    }

    let mut montrilo = legi_bajton(aparato.bus, aparato.slot, aparato.func, 0x34) & 0xfc;
    let mut komuna = None;
    let mut sciiga = None;
    let mut aparata = None;
    let mut multobligo = 1;

    for _ in 0..48 {
        if montrilo == 0 {
            break;
        }
        let identigilo = legi_bajton(aparato.bus, aparato.slot, aparato.func, montrilo);
        let sekva = legi_bajton(aparato.bus, aparato.slot, aparato.func, montrilo + 1) & 0xfc;
        let longo = legi_bajton(aparato.bus, aparato.slot, aparato.func, montrilo + 2);

        if identigilo == KAPABLO_VIRTIO && longo >= 16 {
            let speco = legi_bajton(aparato.bus, aparato.slot, aparato.func, montrilo + 3);
            let kapo = Kapo {
                bar: legi_bajton(aparato.bus, aparato.slot, aparato.func, montrilo + 4),
                offset: legi_dvardon(aparato.bus, aparato.slot, aparato.func, montrilo + 8),
                longo: legi_dvardon(aparato.bus, aparato.slot, aparato.func, montrilo + 12),
            };
            match speco {
                KAPABLO_KOMUNA => komuna = Some(kapo),
                KAPABLO_SCIIGA => {
                    sciiga = Some(kapo);
                    if longo >= 20 {
                        multobligo =
                            legi_dvardon(aparato.bus, aparato.slot, aparato.func, montrilo + 16);
                    }
                }
                KAPABLO_APARATA => aparata = Some(kapo),
                _ => {}
            }
        }

        if sekva == 0 {
            break;
        }
        montrilo = sekva;
    }

    Ok(Kapoj {
        komuna: komuna.ok_or("komuna virtio-kapablo mankas")?,
        sciiga: sciiga.ok_or("sciiga virtio-kapablo mankas")?,
        aparata: aparata.ok_or("aparata virtio-kapablo mankas")?,
        sciiga_multobligo: multobligo.max(1),
    })
}

/// La fizika bazo de BAR; nur MMIO-BAR-oj estas akceptataj.
pub fn baro_bazo(aparato: &Aparato, bar: u8) -> Result<u64, &'static str> {
    if bar > 5 {
        return Err("nevalida BAR-numero");
    }
    let offset = 0x10 + bar * 4;
    let malalta = legi_dvardon(aparato.bus, aparato.slot, aparato.func, offset);
    if malalta & 1 != 0 {
        return Err("I/O-BAR ne estas subtenata");
    }
    let mut bazo = (malalta & !0xf) as u64;
    if (malalta >> 1) & 3 == 2 {
        if bar >= 5 {
            return Err("64-bita BAR transiras la limon");
        }
        let alta = legi_dvardon(aparato.bus, aparato.slot, aparato.func, offset + 4);
        bazo |= (alta as u64) << 32;
    }
    Ok(bazo)
}
