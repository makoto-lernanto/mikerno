//! Kerna nukleo — arkitekture sendependa; la platformon ĝi tuŝas nur tra
//! `crate::arkitekturo`.

use crate::kunhavigita::Kunhavigita;
use crate::memoro::EniraInformo;
use crate::memoro::kadro::KadraAlokilo;
use crate::memoro::virtuala::{AdresaSpaco, PaĝaPermesoj, VirtuaAdreso};

pub mod ipc;
pub mod objekto;
pub mod planisto;

static ALOKILO: Kunhavigita<KadraAlokilo> = Kunhavigita::nova(KadraAlokilo::nova());

pub fn ĉefa(informo: EniraInformo) {
    crate::skribu!("[MiKerno] Saluton, mi estas MiKerno v0.1.0!\n");
    crate::skribu!(
        "[kerno] memoro: {} regionoj, {} MiB uzeblaj\n",
        informo.memora_mapo.regionoj().len(),
        informo.memora_mapo.uzebla_memoro() / (1024 * 1024)
    );

    let alokilo = ALOKILO.akiri();
    alokilo.pravalorizi(informo.memora_mapo, informo.rezervindaj);

    let mut spaco = match crate::arkitekturo::krei_adresan_spacon(alokilo) {
        Ok(spaco) => spaco,
        Err(eraro) => {
            crate::skribu!("[kerno] adresa spaco malsukcesis: {}\n", eraro);
            return;
        }
    };
    if let Err(eraro) = spaco.idente_mapigi(informo.memora_mapo, PaĝaPermesoj::KERNELA, alokilo) {
        crate::skribu!("[kerno] identeca mapo malsukcesis: {}\n", eraro);
        return;
    }
    let framebuffer = match informo.framebuffer {
        Some(fb) => {
            let fino = fb.bazo + fb.paŝo as u64 * fb.alto as u64;
            match spaco.mapigi_intervalon(
                VirtuaAdreso::nova(fb.bazo),
                VirtuaAdreso::nova(fino),
                PaĝaPermesoj::SKRIBA_SENKAŜE,
                alokilo,
            ) {
                Ok(()) => Some(fb),
                Err(eraro) => {
                    crate::skribu!("[kerno] framebuffer-mapado malsukcesis: {}\n", eraro);
                    None
                }
            }
        }
        None => None,
    };
    spaco.ŝalti();

    match framebuffer {
        Some(fb) if crate::ekrano::pravalorizi(fb) => crate::skribu!(
            "[kerno] framebuffer {}x{}, {} bpp\n",
            fb.larĝo,
            fb.alto,
            fb.bpp
        ),
        Some(_) => crate::skribu!("[kerno] framebuffer ne taŭgas; nur COM1\n"),
        None => crate::skribu!("[kerno] framebuffer ne havebla; nur COM1\n"),
    }

    crate::arkitekturo::inicialigi_kernostakon();

    let apic_id = match crate::arkitekturo::pravalorizi_apicon(&mut spaco, alokilo) {
        Ok(id) => id,
        Err(eraro) => {
            crate::skribu!("[kerno] APIC malsukcesis: {}\n", eraro);
            return;
        }
    };
    crate::arkitekturo::pravalorizi_temporizilon();
    crate::skribu!(
        "[kerno] APIC {} kaj paĝado pretaj; temporizilo 100 Hz\n",
        apic_id
    );

    if let Err(eraro) = krei_testajn_fadenojn() {
        crate::skribu!("[kerno] fadena kreado malsukcesis: {:?}\n", eraro);
        return;
    }

    let kapablo = ipc::FinaPunkto::nova(2, ipc::Permesoj::ĈIUJ).kapablo();
    if kapablo.permesoj.posedas(ipc::Permesoj::SENDI) {
        let mesago = ipc::Mesago::nova(1, "saluton el la praŝarĝo".as_bytes());
        match ipc::sendi_mesagon(kapablo.objekto_id, &mesago) {
            Ok(()) => crate::skribu!("[kerno] IPC: mesaĝo sendita al fadeno 2\n"),
            Err(eraro) => crate::skribu!("[kerno] IPC: sendado malsukcesis: {:?}\n", eraro),
        }
    }

    crate::skribu!("[kerno] planisto ekas: 3 fadenoj, antaŭokupo ĉiu 10 ms\n");
    planisto::lanĉi();

    crate::arkitekturo::malebligi_interrompojn();
    crate::skribu!("[kerno] planisto revenis al la ripoza fadeno\n");

    // Observu la sistemon dek sekundojn (1000 tikoj) post la fadenoj.
    crate::interrompo::ebligi_interrompojn();
    let tikoj = loop {
        let nuna = crate::arkitekturo::temporizilaj_tikoj();
        if nuna >= 1000 {
            break nuna;
        }
        crate::arkitekturo::atendi_interrompon();
    };
    crate::interrompo::malebligi_interrompojn();

    let (mortintaj, pretaj, blokitaj) = planisto::raporto();
    crate::skribu!(
        "[kerno] fina stato: {} mortintaj, {} pretaj, {} blokitaj; {} tikoj\n",
        mortintaj,
        pretaj,
        blokitaj,
        tikoj
    );
    crate::arkitekturo::halti()
}

const STAKA_GRANDE: usize = 16 * 1024;
const TESTAJ_FADENOJ: usize = 3;

#[repr(align(16))]
struct FadenaStako(#[allow(dead_code)] [u8; STAKA_GRANDE]);

static mut STAKOJ: [FadenaStako; TESTAJ_FADENOJ] =
    [const { FadenaStako([0; STAKA_GRANDE]) }; TESTAJ_FADENOJ];

fn krei_testajn_fadenojn() -> Result<(), planisto::PlanistaEraro> {
    for indekso in 0..TESTAJ_FADENOJ {
        let id = indekso as u64 + 1;
        let pinto = &raw const STAKOJ as u64 + id * STAKA_GRANDE as u64;
        planisto::aldoni_fadenon(planisto::pretigi_fadenon(
            id,
            VirtuaAdreso::nova(pinto),
            testa_fadeno,
        ))?;
    }
    Ok(())
}

fn testa_fadeno(id: u64) {
    if id == 2 {
        let mesago = ipc::ricevi_mesagon(0);
        crate::skribu!(
            "[fadeno 2] ricevis mesaĝon: tipo {}, \"{}\"\n",
            mesago.tipo,
            teksto(&mesago)
        );
    } else if id == 3 {
        let mesago = ipc::ricevi_mesagon(1);
        crate::skribu!("[fadeno 3] vekiĝis: \"{}\"\n", teksto(&mesago));
    }

    for fojo in 1..=10 {
        crate::skribu!("[fadeno {}] kuranta (fojo {})\n", id, fojo);
        for n in 0..300_000u64 {
            core::hint::black_box(n);
        }
        if id == 1 && fojo == 3 {
            match ipc::sendi_mesagon(3, &ipc::Mesago::nova(2, "vekiĝu, fadeno 3".as_bytes())) {
                Ok(()) => crate::skribu!("[fadeno 1] sendis mesaĝon al fadeno 3\n"),
                Err(eraro) => crate::skribu!("[fadeno 1] IPC: {:?}\n", eraro),
            }
        }
        planisto::cedi();
    }
    crate::skribu!("[fadeno {}] finas sian laboron\n", id);
}

fn teksto(mesago: &ipc::Mesago) -> &str {
    core::str::from_utf8(mesago.enhavo()).unwrap_or("<nevalida>")
}
