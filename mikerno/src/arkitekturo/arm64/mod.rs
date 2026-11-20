//! arm64-malantaŭo: vektora tabelo, GICv2, ĝenerala temporizilo, la
//! kuntekstŝanĝo de la planisto kaj la ligo al la komuna kerno.

use crate::kunhavigita::Kunhavigita;
use crate::memoro::{EniraInformo, MAKSAJ_MODULOJ, MemoraMapo, MemoraRegiono, Modulo, RegionaTipo};

pub mod eniro;
pub mod fdt;
pub mod fw_cfg;
pub mod gic;
pub mod interrompo;
pub mod pagado;
pub mod planado;
pub mod procesoro;
pub mod seria;
pub mod sistemvoko;
pub mod temporizilo;
pub mod uzanta;
pub mod virtio_mmio;

use crate::arkitekturo::komuna::{FirmigaRaporto, FirmigaTrajto};
use crate::memoro::KADRA_GRANDO;
use crate::memoro::kadro::KadraAlokilo;
use crate::memoro::virtuala::{AdresaSpaco, PaĝaPermesoj, PaĝaTablo, UZANTA_BAZO, VirtuaAdreso};

/// Instalu la `SVC`-dispetĉon (la vektora tabelo).
pub fn instali_sistemvokon() {
    sistemvoko::instali();
}

/// Ĉu la procesoro havas `PAN`-on?
pub fn pan_subtenata() -> bool {
    procesoro::pan_subtenata()
}

/// Ĉu la `PAN`-barilo estas aktiva? Ĉiu escepta eniro fermas la fenestron
/// (`SCTLR_EL1.SPAN = 0`), do la barilo funkcias kiam ajn `PAN` ekzistas.
pub fn pan_aktiva() -> bool {
    procesoro::pan_subtenata()
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
    let pan = pan_subtenata();
    raporto.aldonu(FirmigaTrajto {
        nomo: "PAN",
        subtenata: pan,
        aktiva: pan,
        priskribo: "EL1 legas uzantajn paĝojn nur en malfermita fenestro",
    });
    raporto
}

/// La profundo de malfermitaj fenestroj. `PAN` estas nur skribebla, ne
/// legebla, do la kerno mem memoras la staton; unu procesoro kaj unu `EL1`-
/// kunteksto sufiĉas por tio.
static FENESTRO: Kunhavigita<u32> = Kunhavigita::nova(0);

/// Fenestro por la kerna aliro al uzantaj bufroj sub `PAN`.
///
/// `nova` malfermas la fenestron ĉe la unua gardo; `Drop` refermas ĝin nur
/// post la lasta. Nestitaj gardoj do ne fermas fenestron, kiun malfermis la
/// ekstera gardo. La `SPSR_EL1.PAN`-bito konservas la staton trans
/// esceptoj, do interrompo meze de fenestro ne rompas la profundon.
pub struct StacGardo {
    agordis: bool,
}

impl StacGardo {
    pub fn nova() -> Self {
        if !pan_subtenata() {
            return Self { agordis: false };
        }
        let profundo = unsafe { &mut *FENESTRO.akiri() };
        if *profundo == 0 {
            procesoro::malfermi_pan();
        }
        *profundo += 1;
        Self { agordis: true }
    }
}

impl Drop for StacGardo {
    fn drop(&mut self) {
        if self.agordis {
            let profundo = unsafe { &mut *FENESTRO.akiri() };
            *profundo -= 1;
            if *profundo == 0 {
                procesoro::fermi_pan();
            }
        }
    }
}

/// Pruvu, ke `EL1` ne povas legi uzantan datumfolion kun `PAN = 1`.
///
/// La funkcio mapas unu `U=1`-datumfolion en la aktivan kernan spacon kaj legas
/// ĝin sen malfermi `StacGardo`. Se `PAN` funkcias, la datena aborto (`EC=0x25`)
/// kun `FAR` en la uzanta fenestro estas raportata kiel `PAN`-malobservo kaj la
/// kerno haltas; la legado neniam revenas.
pub fn testi_pan(spaco: &mut pagado::ArmAdresaSpaco, alokilo: &mut KadraAlokilo) {
    if let Err(eraro) = spaco.mapi_uzantan_regionon(
        VirtuaAdreso::nova(UZANTA_BAZO),
        KADRA_GRANDO as usize,
        PaĝaPermesoj::UZANTA_DATUMOJ,
        alokilo,
    ) {
        crate::skribu!("[memoro] PAN-provo: mapado malsukcesis: {eraro}\n");
        return;
    }
    let Some(fizika) = spaco.traduki(VirtuaAdreso::nova(UZANTA_BAZO)) else {
        crate::skribu!("[memoro] PAN-provo: traduko mankas\n");
        return;
    };
    // SAFETY: la mapado liveris freŝan kadron, atingeblan per la identeca mapo;
    // ni posedas ĝin kaj skribas unu kontrolitan bajton.
    unsafe {
        core::ptr::write_volatile(fizika.adreso() as *mut u8, 0xA5);
    }
    spaco.ŝalti();
    procesoro::fermi_pan();
    crate::skribu!(
        "[memoro] PAN-provo: EL1 provos legi la uzantan folion ĉe 0x{:x}\n",
        UZANTA_BAZO
    );

    // SAFETY: la celo estas intence uzanta, nur-EL0 datumfolio; kun `PAN=1`
    // la legado devas aborti antaŭ ol la valoro alvenas. La `read_volatile`
    // malhelpas al la kompililo forigi la provon.
    let valoro = unsafe { core::ptr::read_volatile(UZANTA_BAZO as *const u8) };
    crate::skribu!(
        "[memoro] PAN (MALBONE): EL1 legis uzantan paĝon (valoro 0x{:x})\n",
        valoro
    );
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
    let mapo = unsafe { &mut *MAPO.akiri() };
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
    let rezervoj = unsafe { &mut *REZERVOJ.akiri() };
    rezervoj[0] = MemoraRegiono {
        komenco: dtb as u64,
        longo: dtb_longo,
        tipo: RegionaTipo::Rezervita,
    };
    let mut rezervoj_nombro = 1;

    unsafe { *MODULA_NOMBRO.akiri() = 0 };
    if let Some((komenco, fino)) = fdt.initrd() {
        rezervoj[rezervoj_nombro] = MemoraRegiono {
            komenco,
            longo: fino - komenco,
            tipo: RegionaTipo::Rezervita,
        };
        rezervoj_nombro += 1;
        unsafe {
            (*MODULOJ.akiri())[0] = Modulo {
                komenco,
                longo: fino - komenco,
            };
        }
        unsafe { *MODULA_NOMBRO.akiri() = 1 };
    }
    unsafe { *REZERVOJ_NOMBRO.akiri() = rezervoj_nombro };

    let modulo_nombro = unsafe { *MODULA_NOMBRO.akiri() };
    EniraInformo {
        memora_mapo: unsafe { &*MAPO.akiri() },
        rezervindaj: unsafe { core::slice::from_raw_parts(rezervoj.as_ptr(), rezervoj_nombro) },
        // La frambufferio venas de `fw_cfg`/`ramfb` post la pretigo de la
        // kadro-alokilo, do la enira informo ne portas unu.
        frambufo: None,
        moduloj: unsafe { core::slice::from_raw_parts((*MODULOJ.akiri()).as_ptr(), modulo_nombro) },
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
    if let Some(adreso) = fdt.fw_cfg_adreso() {
        fw_cfg::agordi_bazon(adreso);
    }

    let informo = prepari_eniron(dtb, &fdt);
    if let Some((komenco, fino)) = fdt.initrd() {
        crate::skribu!("[kerno] initrd: 0x{:x}..0x{:x}\n", komenco, fino);
    }

    pagado::pravalorizi();
    interrompo::pravalorizi();
    gic::pravalorizi();

    crate::kerno::ĉefa(informo, pagado::KERNA_BAZO, pagado::KERNA_BAZO);
    procesoro::halti()
}
