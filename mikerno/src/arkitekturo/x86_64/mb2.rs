//! Multiboot2-start-informo (Limine, ISO-starto).

use core::ptr;
use core::slice;

use crate::ekrano::FrambufaInformo;
use crate::kunhavigita::Kunhavigita;
use crate::memoro::{
    EniraEraro, EniraInformo, MAKSAJ_MODULOJ, MAKSAJ_REGIONOJ, MemoraMapo, MemoraRegiono, Modulo,
    RegionaTipo,
};

pub const MAGIO: u32 = 0x36d7_6289;

const TIPO_FINO: u32 = 0;
const TIPO_MODULO: u32 = 3;
const TIPO_MEMORA_MAPO: u32 = 6;
const TIPO_FRAMEBUFO: u32 = 8;
const KAPA_GRANDO: usize = 8;
const MAPERA_GRANDO: usize = 24;
const MODULA_KAPA_GRANDO: usize = 16;
const REZERVOJ: usize = 1 + MAKSAJ_MODULOJ;

static MAPO: Kunhavigita<MemoraMapo> = Kunhavigita::nova(MemoraMapo::nova());
static REZERVOJ_AREO: Kunhavigita<[MemoraRegiono; REZERVOJ]> =
    Kunhavigita::nova([MemoraRegiono::MALPLENA; REZERVOJ]);
static REZERVOJ_NOMBRO: Kunhavigita<usize> = Kunhavigita::nova(0);
static FRAMEBUFO: Kunhavigita<Option<FrambufaInformo>> = Kunhavigita::nova(None);
static MODULOJ: Kunhavigita<[Modulo; MAKSAJ_MODULOJ]> = Kunhavigita::nova(
    [Modulo {
        komenco: 0,
        longo: 0,
    }; MAKSAJ_MODULOJ],
);
static MODULA_NOMBRO: Kunhavigita<usize> = Kunhavigita::nova(0);

pub fn analizi(informo: *const u8) -> Result<EniraInformo, EniraEraro> {
    if informo.is_null() {
        return Err(EniraEraro::NevalidaStarto);
    }
    let tuta = unsafe { legi_u32(informo) } as usize;
    if tuta < KAPA_GRANDO || !tuta.is_multiple_of(8) || unsafe { legi_u32(informo.add(4)) } != 0 {
        return Err(EniraEraro::NevalidaMagio);
    }

    let mapo = unsafe { &mut *MAPO.akiri() };
    *mapo = MemoraMapo::nova();
    let framebufo = unsafe { &mut *FRAMEBUFO.akiri() };
    *framebufo = None;
    unsafe { *MODULA_NOMBRO.akiri() = 0 };

    let fino = informo as usize + tuta;
    let mut etikedo = unsafe { informo.add(KAPA_GRANDO) };
    let mut vidis_mapon = false;
    while (etikedo as usize) + KAPA_GRANDO <= fino {
        let tipo = unsafe { legi_u32(etikedo) };
        let grando = unsafe { legi_u32(etikedo.add(4)) } as usize;
        if tipo == TIPO_FINO {
            break;
        }
        if grando < KAPA_GRANDO || (etikedo as usize) + grando > fino {
            return Err(EniraEraro::NevalidaMagio);
        }
        if tipo == TIPO_MEMORA_MAPO {
            legi_mapon(etikedo, grando, mapo)?;
            vidis_mapon = true;
        } else if tipo == TIPO_FRAMEBUFO {
            legi_frambufoon(etikedo, grando, framebufo);
        } else if tipo == TIPO_MODULO {
            legi_modulon(etikedo, grando)?;
        }
        etikedo = (etikedo as usize + grando).next_multiple_of(8) as *const u8;
    }
    if !vidis_mapon {
        return Err(EniraEraro::NeniuMemoraMapo);
    }

    let rezervoj = unsafe { &mut *REZERVOJ_AREO.akiri() };
    let modulo_nombro = unsafe { *MODULA_NOMBRO.akiri() };
    let moduloj = unsafe { &*MODULOJ.akiri() };
    let modulo_eroj = &moduloj[..modulo_nombro];
    rezervoj[0] = MemoraRegiono {
        komenco: informo as u64,
        longo: tuta as u64,
        tipo: RegionaTipo::Rezervita,
    };
    for (indekso, modulo) in modulo_eroj.iter().enumerate() {
        rezervoj[indekso + 1] = MemoraRegiono {
            komenco: modulo.komenco,
            longo: modulo.longo,
            tipo: RegionaTipo::Rezervita,
        };
    }
    unsafe { *REZERVOJ_NOMBRO.akiri() = 1 + modulo_nombro };

    Ok(EniraInformo {
        memora_mapo: unsafe { &*MAPO.akiri() },
        rezervindaj: unsafe { slice::from_raw_parts(rezervoj.as_ptr(), 1 + modulo_nombro) },
        frambufo: *framebufo,
        moduloj: modulo_eroj,
    })
}

/// Multiboot2-modulo (tipo 3): `mod_start`, `mod_end`, `cmdline`, `reserved`.
fn legi_modulon(etikedo: *const u8, grando: usize) -> Result<(), EniraEraro> {
    if grando < MODULA_KAPA_GRANDO {
        return Err(EniraEraro::NevalidaModulo);
    }
    let komenco = unsafe { legi_u32(etikedo.add(8)) } as u64;
    let fino = unsafe { legi_u32(etikedo.add(12)) } as u64;
    if fino <= komenco {
        return Err(EniraEraro::NevalidaModulo);
    }
    let nombro = unsafe { &mut *MODULA_NOMBRO.akiri() };
    if *nombro == MAKSAJ_MODULOJ {
        return Err(EniraEraro::TroMultajModuloj);
    }
    unsafe {
        (*MODULOJ.akiri())[*nombro] = Modulo {
            komenco,
            longo: fino - komenco,
        };
    }
    *nombro += 1;
    Ok(())
}

fn legi_frambufoon(etikedo: *const u8, grando: usize, celo: &mut Option<FrambufaInformo>) {
    if grando < 32 {
        return;
    }
    let (mut ruĝa, mut verda, mut blua) = (16, 8, 0);
    if unsafe { *etikedo.add(29) } == 1 && grando >= 38 {
        ruĝa = unsafe { *etikedo.add(32) };
        verda = unsafe { *etikedo.add(34) };
        blua = unsafe { *etikedo.add(36) };
    }
    *celo = Some(FrambufaInformo {
        bazo: unsafe { legi_u64(etikedo.add(8)) },
        paŝo: unsafe { legi_u32(etikedo.add(16)) },
        larĝo: unsafe { legi_u32(etikedo.add(20)) },
        alto: unsafe { legi_u32(etikedo.add(24)) },
        bitoj_po_pikselo: unsafe { *etikedo.add(28) },
        ruĝa,
        verda,
        blua,
    });
}

fn legi_mapon(etikedo: *const u8, grando: usize, mapo: &mut MemoraMapo) -> Result<(), EniraEraro> {
    if grando < 16 {
        return Err(EniraEraro::NeniuMemoraMapo);
    }
    let erogrando = unsafe { legi_u32(etikedo.add(8)) } as usize;
    if erogrando < MAPERA_GRANDO {
        return Err(EniraEraro::NeniuMemoraMapo);
    }
    let nombro = (grando - 16) / erogrando;
    if nombro > MAKSAJ_REGIONOJ {
        return Err(EniraEraro::TroMultajRegionoj);
    }
    for indekso in 0..nombro {
        let ero = unsafe { etikedo.add(16 + indekso * erogrando) };
        mapo.aldoni(MemoraRegiono {
            komenco: unsafe { legi_u64(ero) },
            longo: unsafe { legi_u64(ero.add(8)) },
            tipo: mapi_tipon(unsafe { legi_u32(ero.add(16)) }),
        });
    }
    Ok(())
}

fn mapi_tipon(tipo: u32) -> RegionaTipo {
    match tipo {
        1 => RegionaTipo::Uzebla,
        2 => RegionaTipo::Rezervita,
        3 => RegionaTipo::Acpi,
        4 => RegionaTipo::Nvs,
        5 => RegionaTipo::Neuzebla,
        _ => RegionaTipo::Nekonata,
    }
}

unsafe fn legi_u32(adreso: *const u8) -> u32 {
    unsafe { ptr::read_unaligned(adreso as *const u32) }
}

unsafe fn legi_u64(adreso: *const u8) -> u64 {
    unsafe { ptr::read_unaligned(adreso as *const u64) }
}
