//! arm64-malantaŭo: A2 — vektora tabelo, GICv2, ĝenerala temporizilo kaj la
//! kuntekstŝanĝo de la planisto.

use crate::interrompo::InterrompaGardo;
use crate::kerno::planisto;
use crate::memoro::virtuala::VirtuaAdreso;

pub mod eniro;
pub mod fdt;
pub mod gic;
pub mod interrompo;
pub mod pagxado;
pub mod planado;
pub mod procesoro;
pub mod seria;
pub mod temporizilo;

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
    interrompo::pravalorizi();
    gic::pravalorizi();
    crate::skribu!("[kerno] VBAR_EL1 kaj GICv2 pretaj; temporizilo 100 Hz\n");

    if let Err(eraro) = krei_fadenojn() {
        crate::skribu!("[kerno] fadenoj ne pretiĝis: {:?}\n", eraro);
        procesoro::halti();
    }
    temporizilo::pravalorizi();

    crate::skribu!("[kerno] planisto ekas: 3 fadenoj, antaŭokupo ĉiu 10 ms\n");
    planisto::lanĉi();

    crate::arkitekturo::malebligi_interrompojn();
    crate::skribu!("[kerno] planisto revenis al la ripoza fadeno\n");
    crate::arkitekturo::ebligi_interrompojn();

    let tikoj = loop {
        let nuna = temporizilo::tikoj();
        if nuna >= CELA_TIKOJ {
            break nuna;
        }
        procesoro::atendi_interrompon();
    };
    crate::arkitekturo::malebligi_interrompojn();

    let (mortintaj, pretaj, blokitaj) = planisto::raporto();
    crate::skribu!(
        "[kerno] fina stato: {} mortintaj, {} pretaj, {} blokitaj; {} tikoj\n",
        mortintaj,
        pretaj,
        blokitaj,
        tikoj
    );
    procesoro::halti()
}

const STAKA_GRANDE: usize = 16 * 1024;
const TESTAJ_FADENOJ: usize = 3;
/// La celo de la observa fadeno kaj la punkto, kie la tria fadeno vekas la duan.
const CELA_TIKOJ: u64 = 1000;
const VEKA_TIKO: u64 = 500;

#[repr(align(16))]
struct FadenaStako([u8; STAKA_GRANDE]);

static STAKOJ: [FadenaStako; TESTAJ_FADENOJ] =
    [const { FadenaStako([0; STAKA_GRANDE]) }; TESTAJ_FADENOJ];

fn krei_fadenojn() -> Result<(), planisto::PlanistaEraro> {
    for indekso in 0..TESTAJ_FADENOJ {
        let id = indekso as u64 + 1;
        let pinto = &raw const STAKOJ as u64 + id * STAKA_GRANDE as u64;
        let enirpunkto = match id {
            1 => fadeno_unu,
            2 => fadeno_du,
            _ => fadeno_tri,
        };
        planisto::aldoni_fadenon(planisto::pretigi_fadenon(
            id,
            0,
            VirtuaAdreso::nova(pinto),
            enirpunkto,
        ))?;
    }
    Ok(())
}

/// Nombras la tikojn ĝis la celo kaj raportas ĉiun centan.
fn fadeno_unu(_id: u64) {
    let mut sekva = 100u64;
    loop {
        let nuna = temporizilo::tikoj();
        if nuna >= CELA_TIKOJ {
            break;
        }
        if nuna >= sekva {
            crate::skribu!("[fadeno 1] {} tikoj\n", sekva);
            sekva += 100;
        }
    }
    crate::skribu!("[fadeno 1] finiĝis post {} tikoj\n", temporizilo::tikoj());
}

/// Blokiĝas kaj atendas la vekon de la tria fadeno.
fn fadeno_du(_id: u64) {
    crate::skribu!("[fadeno 2] atendas vekon\n");
    {
        let _gardo = InterrompaGardo::nova();
        planisto::bloki();
    }
    crate::skribu!("[fadeno 2] vekita ĉe {} tikoj\n", temporizilo::tikoj());
}

/// Atendas `VEKA_TIKO`-n, vekas la duan fadenon kaj finiĝas.
fn fadeno_tri(_id: u64) {
    crate::skribu!("[fadeno 3] atendas {} tikojn\n", VEKA_TIKO);
    while temporizilo::tikoj() < VEKA_TIKO {}
    {
        let _gardo = InterrompaGardo::nova();
        planisto::veki(2);
    }
    crate::skribu!(
        "[fadeno 3] vekis fadenon 2 ĉe {} tikoj\n",
        temporizilo::tikoj()
    );
}
