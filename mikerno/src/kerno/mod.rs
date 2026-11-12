//! Kerna nukleo — arkitekture sendependa; la platformon ĝi tuŝas nur tra
//! `crate::arkitekturo`.

use crate::kunhavigita::Kunhavigita;
use crate::memoro::EniraInformo;
use crate::memoro::KADRA_GRANDO;
use crate::memoro::kadro::KadraAlokilo;
use crate::memoro::virtuala::{
    AdresaSpaco, PaĝaPermesoj, PaĝaTablo, UZANTA_BAZO, UZANTA_STAKO_SUPRO, VirtuaAdreso,
};

pub mod ipc;
pub mod objekto;
pub mod planisto;
pub mod procezo;
pub mod sistemvoko;
pub mod uzantspaco;

static ALOKILO: Kunhavigita<KadraAlokilo> = Kunhavigita::nova(KadraAlokilo::nova());

pub fn ĉefa(informo: EniraInformo) {
    crate::skribu!("[MiKerno] Saluton, mi estas MiKerno v0.2.2!\n");
    sistemvoko::pravalorizi();
    crate::skribu!(
        "[kerno] memoro: {} regionoj, {} MiB uzeblaj\n",
        informo.memora_mapo.regionoj().len(),
        informo.memora_mapo.uzebla_memoro() / (1024 * 1024)
    );

    let alokilo = ALOKILO.akiri();
    alokilo.pravalorizi(informo.memora_mapo, informo.rezervindaj);
    testi_liberigon(alokilo);

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
    // Moduloj povas kuŝi en regiono, kiun la memormapo ne markas uzebla; la
    // identeca mapo tiam mankas, do mapu ilin antaŭ ol ŝalti CR3.
    for modulo in informo.moduloj {
        if let Err(eraro) = spaco.mapigi_intervalon(
            VirtuaAdreso::nova(modulo.komenco),
            VirtuaAdreso::nova(modulo.fino()),
            PaĝaPermesoj::KERNELA,
            alokilo,
        ) {
            crate::skribu!("[kerno] modula mapado malsukcesis: {}\n", eraro);
            return;
        }
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
    procezo::agordi_kernan_spacon(spaco.radika_adreso());

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

    // La bildo estas ŝargata rekte en la privatan spacon de la procezo, ne en
    // la kernan: `CR3` ŝanĝiĝos al tiu spaco antaŭ ol `horizonto` ekas.
    let mut uzanta_preta = false;
    match informo.moduloj.first() {
        Some(modulo) => match procezo::krei(UZANTA_FADENO_ID, &spaco, alokilo) {
            Ok(id) => match procezo::akiri(id) {
                Some(procezo) => {
                    let stato = procezo.stato();
                    let kerna_pinto = procezo.kerna_stako_pinto();
                    let radiko = procezo.radika_adreso();
                    let spaco_proceza = procezo.spaco();
                    match uzantspaco::pravalorizi(spaco_proceza, alokilo, modulo) {
                        Ok(()) => {
                            let (bilda_komenco, _) = uzantspaco::bildaj_limoj();
                            let kerna = spaco_proceza.traduki(VirtuaAdreso::nova(0x1000)).is_some();
                            let privata =
                                spaco.traduki(VirtuaAdreso::nova(bilda_komenco)).is_none();
                            crate::skribu!(
                                "[procezo] {}: {:?}, kerna stako 0x{:x}, PML4 0x{:x}, kerna mapo {}, bildo nur en la proceza spaco {}\n",
                                procezo.id(),
                                stato,
                                kerna_pinto,
                                radiko.adreso(),
                                if kerna { "kunhavata" } else { "MANKANTA" },
                                if privata { "jes" } else { "NE" }
                            );
                            uzanta_preta = true;
                        }
                        Err(eraro) => {
                            crate::skribu!("[uzanto] ŝargo malsukcesis: {}\n", eraro);
                        }
                    }
                }
                None => crate::skribu!("[procezo] {}: registro mankas\n", id),
            },
            Err(eraro) => crate::skribu!("[procezo] kreado malsukcesis: {}\n", eraro),
        },
        None => crate::skribu!("[uzanto] horizonto ne liverita (mankas modulo)\n"),
    }

    if let Err(eraro) = krei_testajn_fadenojn() {
        crate::skribu!("[kerno] fadena kreado malsukcesis: {:?}\n", eraro);
        return;
    }

    let kapablo = ipc::FinaPunkto::nova(2, ipc::Permesoj::ĈIUJ).kapablo();
    if kapablo.permesoj.posedas(ipc::Permesoj::SENDI) {
        let mesago = ipc::Mesago::nova(1, 0, 2, "saluton el la praŝarĝo".as_bytes());
        match ipc::sendi_mesagon(kapablo.objekto_id, &mesago) {
            Ok(()) => crate::skribu!("[kerno] IPC: mesaĝo sendita al fadeno 2\n"),
            Err(eraro) => crate::skribu!("[kerno] IPC: sendado malsukcesis: {:?}\n", eraro),
        }
    }

    crate::skribu!("[kerno] planisto ekas: 3 fadenoj, antaŭokupo ĉiu 10 ms\n");
    planisto::lanĉi();

    // La testaj fadenoj finiĝis; nun `horizonto` ekas en Ring 3, en la propra
    // adresa spaco de la procezo.
    if uzanta_preta {
        match uzantspaco::enirpunkto() {
            Some(enirpunkto) => match lanĉi_horizonton(enirpunkto) {
                Ok(()) => crate::skribu!("[kerno] horizonto finiĝis; planisto revenis\n"),
                Err(eraro) => crate::skribu!("[uzanto] fadena kreado malsukcesis: {:?}\n", eraro),
            },
            None => crate::skribu!("[uzanto] enirpunkto mankas; Ring 3 ne ekas\n"),
        }
    }

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
const UZANTA_FADENO_ID: u64 = TESTAJ_FADENOJ as u64 + 1;

/// Pruvu la liberigan vojon per proviza adresa spaco: kreu ĝin, mapu unu
/// uzantan paĝon, detruu ĝin kaj komparu la nombron de liberaj kadroj.
fn testi_liberigon(alokilo: &mut KadraAlokilo) {
    let antaŭ = alokilo.liberaj();
    let mut spaco = match crate::arkitekturo::krei_adresan_spacon(alokilo) {
        Ok(spaco) => spaco,
        Err(eraro) => {
            crate::skribu!("[memoro] liberiga provo malsukcesis: {}\n", eraro);
            return;
        }
    };
    let mapado = spaco.mapigi_freŝan_regionon(
        VirtuaAdreso::nova(UZANTA_BAZO),
        KADRA_GRANDO as usize,
        PaĝaPermesoj::UZANTA_DATUMOJ,
        alokilo,
    );
    let detruo = spaco.detruu(alokilo);
    let post = alokilo.liberaj();
    match mapado.and(detruo) {
        Ok(()) if post == antaŭ => crate::skribu!(
            "[memoro] liberigo (bone): {} kadroj antaŭ, {} post\n",
            antaŭ,
            post
        ),
        Ok(()) => crate::skribu!(
            "[memoro] liberigo (MALBONE): {} kadroj antaŭ, {} post\n",
            antaŭ,
            post
        ),
        Err(eraro) => crate::skribu!("[memoro] liberigo malsukcesis: {}\n", eraro),
    }
}

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
            0,
            VirtuaAdreso::nova(pinto),
            testa_fadeno,
        ))?;
    }
    Ok(())
}

/// Aldonu la uzantan fadenon kaj planu ĝin. La fadeno uzas la kernan stakon
/// de sia procezo; post `iretq` ĝi plenumas sur la uzanta stako. La voko
/// revenas, kiam `fini` mortigas ĝin kaj la planisto elektas la ripozan
/// kuntekston.
fn lanĉi_horizonton(enirpunkto: u64) -> Result<(), planisto::PlanistaEraro> {
    let Some(procezo) = procezo::akiri(UZANTA_FADENO_ID) else {
        return Err(planisto::PlanistaEraro::NevalidaIdentigilo);
    };
    let pinto = procezo.kerna_stako_pinto();
    planisto::aldoni_fadenon(planisto::pretigi_fadenon(
        UZANTA_FADENO_ID,
        UZANTA_FADENO_ID,
        VirtuaAdreso::nova(pinto),
        uzanta_fadeno,
    ))?;
    crate::skribu!(
        "[kerno] horizonto ekas en Ring 3: CS=0x23, SS=0x1B, RFLAGS=0x202, enirpunkto 0x{:x}\n",
        enirpunkto
    );
    planisto::lanĉi();
    Ok(())
}

fn uzanta_fadeno(_id: u64) {
    let Some(enirpunkto) = uzantspaco::enirpunkto() else {
        planisto::morti();
    };
    // SAFETY: `uzantspaco::pravalorizi` muntis la bildon kun bito `U` kaj
    // kontrolis la enirpunkton; la stako estas la mapita 64 KiB-a regiono.
    unsafe { crate::arkitekturo::transiri_al_ring3(enirpunkto, UZANTA_STAKO_SUPRO) }
}

fn testa_fadeno(id: u64) {
    if id == 1 {
        sistemvoko::testi_ring0();
    } else if id == 2 {
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
            match ipc::sendi_mesagon(
                3,
                &ipc::Mesago::nova(2, 1, 3, "vekiĝu, fadeno 3".as_bytes()),
            ) {
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
