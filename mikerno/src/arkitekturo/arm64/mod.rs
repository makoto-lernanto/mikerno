//! arm64-malantaŭo: vektora tabelo, GICv2, ĝenerala temporizilo, la
//! kuntekstŝanĝo de la planisto kaj la ligo al la komuna kerno.

use crate::kunhavigita::Kunhavigita;
use crate::memoro::{EniraInformo, MAKSAJ_MODULOJ, MemoraMapo, MemoraRegiono, Modulo, RegionaTipo};

pub mod eniro;
pub mod fdt;
pub mod gic;
pub mod interrompo;
pub mod pagxado;
pub mod planado;
pub mod procesoro;
pub mod seria;
pub mod sistemvoko;
pub mod temporizilo;
pub mod uzanta;

use crate::arkitekturo::komuna::{FirmigaRaporto, FirmigaTrajto};

/// Instalu la `SVC`-dispetĉon (la vektora tabelo).
pub fn instali_sistemvokon() {
    sistemvoko::instali();
}

/// La firmigaj trajtoj de `arm64`, por la starta raporto.
pub fn firmiga_raporto() -> FirmigaRaporto {
    let mut raporto = FirmigaRaporto::nova();
    raporto.aldonu(FirmigaTrajto {
        nomo: "PXN",
        subtenata: true,
        aktiva: true,
        priskribo: "EL1 ne plenumas uzantajn paĝojn",
    });
    raporto.aldonu(FirmigaTrajto {
        nomo: "UXN",
        subtenata: true,
        aktiva: true,
        priskribo: "EL0 ne plenumas datumajn paĝojn",
    });
    raporto.aldonu(FirmigaTrajto {
        nomo: "PAN",
        subtenata: false,
        aktiva: false,
        priskribo: "ne uzata en A3 (`SCTLR_EL1.SPAN = 0`)",
    });
    raporto
}

/// Fenestro por la kerna aliro al uzantaj bufroj.
///
/// `arm64` de A3 ne ŝaltas `PAN`-on (`SCTLR_EL1.SPAN = 0`), do la kerno rajtas
/// legi uzantajn paĝojn rekte kaj la gardo estas senefika.
pub struct StacGardo;

impl StacGardo {
    pub fn nova() -> Self {
        Self
    }
}

pub fn smap_aktiva() -> bool {
    false
}

pub fn smep_aktiva() -> bool {
    false
}

/// Praŝarĝaj strukturoj, kiuj restas rezervitaj: la DTB kaj la `initrd`.
const MAKSAJ_REZERVOJ: usize = 2;

static MAPO: Kunhavigita<MemoraMapo> = Kunhavigita::nova(MemoraMapo::nova());
static REZERVOJ: Kunhavigita<[MemoraRegiono; MAKSAJ_REZERVOJ]> =
    Kunhavigita::nova([MemoraRegiono::MALPLENA; MAKSAJ_REZERVOJ]);
static REZERVOJ_NOMBRO: Kunhavigita<usize> = Kunhavigita::nova(0);
static MODULOJ: Kunhavigita<[Modulo; MAKSAJ_MODULOJ]> = Kunhavigita::nova(
    [Modulo {
        komenco: 0,
        longo: 0,
    }; MAKSAJ_MODULOJ],
);
static MODULA_NOMBRO: Kunhavigita<usize> = Kunhavigita::nova(0);

/// Konstruu la arkitekture sendependan eniran informon el la FDT. La `initrd`
/// estas samtempe modula bildo kaj rezervita regiono: la nukleo legos la ELF-on
/// el ĝi ĝis la fino de la scenaro.
fn prepari_eniron(dtb: *const u8, fdt: &fdt::Fdt) -> EniraInformo {
    let mapo = MAPO.akiri();
    *mapo = MemoraMapo::nova();
    for regiono in fdt.memoro() {
        mapo.aldoni(MemoraRegiono {
            komenco: regiono.komenco,
            longo: regiono.longo,
            tipo: RegionaTipo::Uzebla,
        });
    }

    // SAFETY: `Fdt::nova` jam validigis la kapon, do `totalsize` estas legebla.
    let dtb_longo = u32::from_be(unsafe { (dtb.add(4) as *const u32).read_unaligned() }) as u64;
    let rezervoj = REZERVOJ.akiri();
    rezervoj[0] = MemoraRegiono {
        komenco: dtb as u64,
        longo: dtb_longo,
        tipo: RegionaTipo::Rezervita,
    };
    let mut rezervoj_nombro = 1;

    *MODULA_NOMBRO.akiri() = 0;
    if let Some((komenco, fino)) = fdt.initrd() {
        rezervoj[rezervoj_nombro] = MemoraRegiono {
            komenco,
            longo: fino - komenco,
            tipo: RegionaTipo::Rezervita,
        };
        rezervoj_nombro += 1;
        MODULOJ.akiri()[0] = Modulo {
            komenco,
            longo: fino - komenco,
        };
        *MODULA_NOMBRO.akiri() = 1;
    }
    *REZERVOJ_NOMBRO.akiri() = rezervoj_nombro;

    let rezervoj = REZERVOJ.akiri();
    let modulo_nombro = *MODULA_NOMBRO.akiri();
    EniraInformo {
        memora_mapo: &*MAPO.akiri(),
        rezervindaj: unsafe { core::slice::from_raw_parts(rezervoj.as_ptr(), rezervoj_nombro) },
        moduloj: unsafe { core::slice::from_raw_parts(MODULOJ.akiri().as_ptr(), modulo_nombro) },
    }
}

/// La unua Rust-funkcio post la asembla eniro; `dtb` venas el la praŝarĝilo.
#[unsafe(no_mangle)]
pub extern "C" fn kerna_eniro_arm64(dtb: *const u8) -> ! {
    seria::pravalorizi(seria::DEFAŬLTA);

    let Some(fdt) = fdt::Fdt::nova(dtb) else {
        crate::skribu!("[kerno] averto: FDT ne estas valida; uzata estas la fiksa PL011\n");
        procesoro::halti()
    };
    if let Some(adreso) = fdt.seria_adreso() {
        seria::pravalorizi(adreso as usize);
    }

    let informo = prepari_eniron(dtb, &fdt);
    if let Some((komenco, fino)) = fdt.initrd() {
        crate::skribu!("[kerno] initrd: 0x{:x}..0x{:x}\n", komenco, fino);
    }

    pagxado::pravalorizi();
    interrompo::pravalorizi();
    gic::pravalorizi();

    crate::kerno::ĉefa(informo, pagxado::KERNA_BAZO, pagxado::KERNA_BAZO);
    procesoro::halti()
}
