//! Minimuma `fw_cfg`-kliento por `ramfb`.
//!
//! QEMU `virt` metas `fw_cfg` ĉe `0x0902_0000`; la FDT anoncas la saman
//! adreson per la kongruaĵo `qemu,fw-cfg-mmio`. La datumregistro akceptas nur
//! unu-bajtajn alirojn, la selektilo nur 16-bitan skribon kaj la DMA-registro
//! 64-bitan skribon; ĉiuj valoroj estas en granda fina ordo.
//!
//! `ramfb` postulas la DMA-vojon: la bajtaj skriboj al la datumregistro estas
//! senefikaj ekde QEMU 2.4. La agordo de `etc/ramfb` estas 28 bajtoj da granda
//! fina ordo (`addr`, `fourcc`, `flags`, `width`, `height`, `stride`).

use core::arch::asm;
use core::mem::size_of;
use core::ptr::{read_volatile, write_volatile};

use crate::ekrano::FrambufaInformo;
use crate::kunhavigita::Kunhavigita;
use crate::memoro::KADRA_GRANDO;
use crate::memoro::kadro::KadraAlokilo;

/// Fiksa adreso de `fw_cfg` sur QEMU `virt`, se la FDT ne liveras alian.
const DEFAŬLTA: u64 = 0x0902_0000;

static BAZO: Kunhavigita<u64> = Kunhavigita::nova(DEFAŬLTA);

const DATUMO: u64 = 0x00;
const SELEKTILO: u64 = 0x08;
const DMA: u64 = 0x10;

/// La klavo de la dosiera dosierujo (`FW_CFG_FILE_DIR`).
const DOSIERUJO: u16 = 0x0019;
/// `struct FWCfgFile`: `size` (be32), `select` (be16), `reserved` (be16),
/// `name[56]`.
const DOSIERA_ERO: usize = 64;
const MAKSAJ_DOSIEROJ: u32 = 64;
const RAMFB_NOMO: &[u8] = b"etc/ramfb";

const DMA_ERARO: u32 = 0x01;
const DMA_ELEKTI: u32 = 0x08;
const DMA_SKRIBI: u32 = 0x10;

/// La `ramfb`-bildo: 1280×800, XRGB8888. La grando kongruas kun la implicita
/// Limine-reĝimo de la `x86_64`-vojo, do la piksela haketo estas komparebla.
pub const LARĜO: u32 = 1280;
pub const ALTO: u32 = 800;
pub const PAŜO: u32 = LARĜO * 4;
/// `fourcc_code('X', 'R', '2', '4')`: XRGB8888.
const KOLORSPACO_XR24: u32 = 0x3432_5258;

#[repr(C)]
struct DmaEro {
    kontrolo: u32,
    longo: u32,
    adreso: u64,
}

const _: () = assert!(size_of::<DmaEro>() == 16);

#[repr(C, packed)]
struct Agordo {
    adreso: u64,
    koloro: u32,
    flagoj: u32,
    larĝo: u32,
    alto: u32,
    paŝo: u32,
}

const _: () = assert!(size_of::<Agordo>() == 28);

/// Agordu la `fw_cfg`-bazon el la FDT.
pub fn agordi_bazon(bazo: u64) {
    unsafe { *BAZO.akiri() = bazo };
}

/// La uzata `fw_cfg`-bazo.
pub fn bazo() -> u64 {
    unsafe { *BAZO.akiri() }
}

/// Aloku kontiguan RAM-frambufon kaj anoncu ĝin al `ramfb` per `fw_cfg`.
///
/// `None` se la `etc/ramfb`-dosiero mankas, se ne ekzistas sufiĉe da
/// sinsekvaj kadroj aŭ se la DMA-transigo raportas eraron.
pub fn krei_frambufon(alokilo: &mut KadraAlokilo) -> Option<FrambufaInformo> {
    let bazo = bazo();
    let klavo = serĉi_dosieron(bazo, RAMFB_NOMO)?;
    let paĝoj = PAŜO as usize * ALTO as usize / KADRA_GRANDO as usize;
    let kadro = alokilo.aloki_kontiguajn(paĝoj)?;
    let adreso = kadro.adreso();
    let agordo = Agordo {
        adreso: adreso.to_be(),
        koloro: KOLORSPACO_XR24.to_be(),
        flagoj: 0,
        larĝo: LARĜO.to_be(),
        alto: ALTO.to_be(),
        paŝo: PAŜO.to_be(),
    };
    // SAFETY: `Agordo` estas `repr(C)` kaj 28-bajta; la bufero vivas sur la
    // stako dum la sinkrona DMA-transigo.
    let bufro =
        unsafe { core::slice::from_raw_parts(&raw const agordo as *const u8, size_of::<Agordo>()) };
    if !dma_skribi(bazo, klavo, bufro) {
        crate::skribu!("[kerno] ramfb: la fw_cfg-DMA malsukcesis\n");
        // La kadroj restas rezervitaj: la aparato eble ankoraŭ montras ilin.
        return None;
    }
    Some(FrambufaInformo {
        bazo: adreso,
        larĝo: LARĜO,
        alto: ALTO,
        paŝo: PAŜO,
        bitoj_po_pikselo: 32,
        ruĝa: 16,
        verda: 8,
        blua: 0,
    })
}

/// Serĉu la selektan klavon de dosiero en `FW_CFG_FILE_DIR`.
fn serĉi_dosieron(bazo: u64, nomo: &[u8]) -> Option<u16> {
    elekti(bazo, DOSIERUJO);
    let mut nombro = [0u8; 4];
    for bajto in &mut nombro {
        *bajto = legi_bajton(bazo);
    }
    let nombro = u32::from_be_bytes(nombro);
    if nombro == 0 || nombro > MAKSAJ_DOSIEROJ {
        return None;
    }
    for _ in 0..nombro {
        let mut ero = [0u8; DOSIERA_ERO];
        for bajto in &mut ero {
            *bajto = legi_bajton(bazo);
        }
        if &ero[8..8 + nomo.len()] == nomo && ero[8 + nomo.len()] == 0 {
            return Some(u16::from_be_bytes([ero[4], ero[5]]));
        }
    }
    None
}

fn legi_bajton(bazo: u64) -> u8 {
    // SAFETY: la datumregistro de `fw_cfg` estas MMIO-legado sen kromefikoj.
    unsafe { read_volatile((bazo + DATUMO) as *const u8) }
}

fn elekti(bazo: u64, klavo: u16) {
    // SAFETY: la selektilo de `fw_cfg` akceptas nur 16-bitan skribon, en granda
    // fina ordo (la aparato estas `DEVICE_BIG_ENDIAN`).
    unsafe { write_volatile((bazo + SELEKTILO) as *mut u16, klavo.to_be()) };
}

/// Plenumu unu DMA-skribon kaj raportu, ĉu ĝi sukcesis.
fn dma_skribi(bazo: u64, klavo: u16, bufro: &[u8]) -> bool {
    let mut ero = DmaEro {
        kontrolo: (DMA_SKRIBI | DMA_ELEKTI | (u32::from(klavo) << 16)).to_be(),
        longo: (bufro.len() as u32).to_be(),
        adreso: (bufro.as_ptr() as u64).to_be(),
    };
    let ero_adreso = (&raw mut ero as u64).to_be();
    // La priskribo kaj la bufero kuŝas en la identece mapita kerna RAM, do
    // ilia virtuala adreso estas ankaŭ fizika. `dsb` certigas, ke la enhavo
    // videblas antaŭ la DMA-peto; la asm ankaŭ malhelpas al la kompililo
    // ŝovi la skribojn malantaŭ la MMIO-aliron.
    // SAFETY: la DMA-registro akceptas 64-bitan skribon ĉe sia bazo; la
    // priskribo estas `repr(C)`, 16-bajta kaj vivas dum la sinkrona transigo.
    unsafe {
        asm!("dsb sy", options(nostack));
        write_volatile((bazo + DMA) as *mut u64, ero_adreso);
    }
    // QEMU plenumas la transigon sinkrone dum la skribo al la DMA-registro, do
    // la kontrolkampo jam havas la finan valoron (0 aŭ `DMA_ERARO`).
    let kontrolo = u32::from_be(unsafe { read_volatile(&raw const ero.kontrolo) });
    kontrolo & DMA_ERARO == 0
}
