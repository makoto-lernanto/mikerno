//! Xen-PVH start-informo: memora mapo kaj rezervendaj regionoj.

use core::mem::size_of;
use core::slice;

use crate::kunhavigita::Kunhavigita;
use crate::memoro::{
    EniraEraro, EniraInformo, KADRA_GRANDO, MAKSAJ_REGIONOJ, MemoraMapo, MemoraRegiono, RegionaTipo,
};

const MAGIO: u32 = 0x336e_c578;
const REZERVOJ: usize = 8;

#[repr(C)]
#[allow(dead_code)]
struct HvmStartInfo {
    magio: u32,
    versio: u32,
    flagoj: u32,
    nombro_da_moduloj: u32,
    modlista_paddr: u64,
    komandlinia_paddr: u64,
    rsdp_paddr: u64,
    memmap_paddr: u64,
    memmap_entoj: u32,
    rezervita: u32,
}

#[repr(C)]
struct HvmMemmapEro {
    adreso: u64,
    grando: u64,
    tipo: u32,
    _rezervita: u32,
}

static MAPO: Kunhavigita<MemoraMapo> = Kunhavigita::nova(MemoraMapo::nova());
static REZERVOJ_AREO: Kunhavigita<[MemoraRegiono; REZERVOJ]> =
    Kunhavigita::nova([MemoraRegiono::MALPLENA; REZERVOJ]);
static REZERVOJ_NOMBRO: Kunhavigita<usize> = Kunhavigita::nova(0);

pub fn analizi(starta_informo: *const u8) -> Result<EniraInformo, EniraEraro> {
    if starta_informo.is_null() {
        return Err(EniraEraro::NevalidaStarto);
    }

    let starta = unsafe { &*(starta_informo as *const HvmStartInfo) };
    if starta.magio != MAGIO {
        return Err(EniraEraro::NevalidaMagio);
    }
    if starta.versio < 1 {
        return Err(EniraEraro::NesubtenataVersio);
    }
    if starta.memmap_paddr == 0 || starta.memmap_entoj == 0 {
        return Err(EniraEraro::NeniuMemoraMapo);
    }
    if starta.memmap_entoj as usize > MAKSAJ_REGIONOJ {
        return Err(EniraEraro::TroMultajRegionoj);
    }

    let mapo = MAPO.akiri();
    let rezervoj = REZERVOJ_AREO.akiri();
    let nombro = REZERVOJ_NOMBRO.akiri();
    *mapo = MemoraMapo::nova();
    *nombro = 0;

    let memmapa_grando = starta.memmap_entoj as u64 * size_of::<HvmMemmapEro>() as u64;
    let eroj = unsafe {
        slice::from_raw_parts(
            starta.memmap_paddr as *const HvmMemmapEro,
            starta.memmap_entoj as usize,
        )
    };
    for ero in eroj {
        mapo.aldoni(MemoraRegiono {
            komenco: ero.adreso,
            longo: ero.grando,
            tipo: mapi_tipon(ero.tipo),
        });
    }

    aldoni_rezervon(
        rezervoj,
        nombro,
        starta_informo as u64,
        size_of::<HvmStartInfo>() as u64,
    )?;
    aldoni_rezervon(rezervoj, nombro, starta.memmap_paddr, memmapa_grando)?;
    if starta.rsdp_paddr != 0 {
        aldoni_rezervon(rezervoj, nombro, starta.rsdp_paddr, KADRA_GRANDO)?;
    }

    let memora_mapo = &*MAPO.akiri();
    let rezervindaj = unsafe { slice::from_raw_parts(REZERVOJ_AREO.akiri().as_ptr(), *nombro) };

    Ok(EniraInformo {
        memora_mapo,
        rezervindaj,
        framebuffer: None,
    })
}

fn aldoni_rezervon(
    rezervoj: &mut [MemoraRegiono; REZERVOJ],
    nombro: &mut usize,
    komenco: u64,
    longo: u64,
) -> Result<(), EniraEraro> {
    if longo == 0 {
        return Ok(());
    }
    if *nombro == REZERVOJ {
        return Err(EniraEraro::TroMultajRegionoj);
    }
    rezervoj[*nombro] = MemoraRegiono {
        komenco,
        longo,
        tipo: RegionaTipo::Rezervita,
    };
    *nombro += 1;
    Ok(())
}

fn mapi_tipon(tipo: u32) -> RegionaTipo {
    match tipo {
        1 => RegionaTipo::Uzebla,
        2 => RegionaTipo::Rezervita,
        3 => RegionaTipo::Acpi,
        4 => RegionaTipo::Nvs,
        5 => RegionaTipo::Neuzebla,
        6 => RegionaTipo::Malŝaltita,
        7 => RegionaTipo::Pmem,
        _ => RegionaTipo::Nekonata,
    }
}
