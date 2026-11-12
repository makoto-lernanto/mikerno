//! Kerna nukleo — arkitekture sendependa; la platformon ĝi tuŝas nur tra
//! `crate::arkitekturo`.

use crate::arkitekturo::Spaco;
use crate::kunhavigita::Kunhavigita;
use crate::memoro::EniraInformo;
use crate::memoro::KADRA_GRANDO;
use crate::memoro::Modulo;
use crate::memoro::kadro::KadraAlokilo;
use crate::memoro::virtuala::{
    AdresaSpaco, PaĝaPermesoj, PaĝaTablo, UZANTA_BAZO, UZANTA_STAKO_SUPRO, VirtuaAdreso,
};
use libmikerno::starto::{REĜIMO_FINI, REĜIMO_KRUI};

pub mod ipc;
pub mod objekto;
pub mod planisto;
pub mod procezo;
pub mod sistemvoko;
pub mod uzantspaco;

static ALOKILO: Kunhavigita<KadraAlokilo> = Kunhavigita::nova(KadraAlokilo::nova());

/// La tutmonda kadro-alokilo, uzata ankaŭ de la proceza detruo en la
/// planisto. Vokenda kun interrompoj malŝaltitaj.
pub fn kadra_alokilo() -> &'static mut KadraAlokilo {
    ALOKILO.akiri()
}

pub fn ĉefa(informo: EniraInformo) {
    crate::skribu!("[MiKerno] Saluton, mi estas MiKerno v0.2.3!\n");
    sistemvoko::pravalorizi();
    crate::skribu!(
        "[kerno] memoro: {} regionoj, {} MiB uzeblaj\n",
        informo.memora_mapo.regionoj().len(),
        informo.memora_mapo.uzebla_memoro() / (1024 * 1024)
    );

    // La alokilo restas prunteprenita nur en ĉi tiu bloko: la planisto
    // aliras ĝin mem dum la proceza detruo, do longa `&mut` malhelpus.
    let (mut spaco, framebuffer) = {
        let alokilo = kadra_alokilo();
        alokilo.pravalorizi(informo.memora_mapo, informo.rezervindaj);
        testi_liberigon(alokilo);

        let mut spaco = match crate::arkitekturo::krei_adresan_spacon(alokilo) {
            Ok(spaco) => spaco,
            Err(eraro) => {
                crate::skribu!("[kerno] adresa spaco malsukcesis: {}\n", eraro);
                return;
            }
        };
        if let Err(eraro) = spaco.idente_mapigi(informo.memora_mapo, PaĝaPermesoj::KERNELA, alokilo)
        {
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
        (spaco, framebuffer)
    };

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

    let apic_id = {
        let alokilo = kadra_alokilo();
        match crate::arkitekturo::pravalorizi_apicon(&mut spaco, alokilo) {
            Ok(id) => id,
            Err(eraro) => {
                crate::skribu!("[kerno] APIC malsukcesis: {}\n", eraro);
                return;
            }
        }
    };
    crate::arkitekturo::pravalorizi_temporizilon();
    crate::skribu!(
        "[kerno] APIC {} kaj paĝado pretaj; temporizilo 100 Hz\n",
        apic_id
    );

    let modulo = informo.moduloj.first();
    match modulo {
        Some(modulo) => testi_procezan_reuzon(&spaco, modulo),
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

    // La testaj fadenoj finiĝis; nun la du `horizonto`-procezoj ekas en Ring 3,
    // ĉiu en sia propra adresa spaco. Ili plenumas sinsekve: la unua finiĝas
    // per `SYS_FINI`, la dua per intenca paĝa eraro. Ambaŭ vojoj detruas la
    // procezan spacon antaŭ ol la planisto revenas al la ripoza kunteksto.
    if let Some(modulo) = modulo {
        let _ = lanĉi_uzantan_procezon(&spaco, modulo, UZANTA_FADENO_ID);
        let _ = lanĉi_uzantan_procezon(&spaco, modulo, KRUA_FADENO_ID);
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
/// La fadeno — kaj samnoma procezo — de la unua `horizonto`.
const UZANTA_FADENO_ID: u64 = TESTAJ_FADENOJ as u64 + 1;
/// La dua `horizonto`, kiu intence kaŭzas paĝan eraron.
const KRUA_FADENO_ID: u64 = UZANTA_FADENO_ID + 1;
/// Ripeta ciklo de la reuzoprovo; libera identigilo, ne fadeno.
const PROVA_PROCEZA_ID: u64 = 6;
const REUZOJ: usize = 10;

/// La scenaro, kiun `horizonto` plenumas: normala `SYS_FINI` aŭ intenca
/// paĝa eraro por pruvi la kapton de Ring 3-esceptoj.
fn komenca_reĝimo(fadeno: u64) -> u64 {
    if fadeno == KRUA_FADENO_ID {
        REĜIMO_KRUI
    } else {
        REĜIMO_FINI
    }
}

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

/// Kreu kaj detruu la saman procezan spacon dek fojojn. Ĉiu ciklo muntas la
/// bildon kaj preparas la stakon; la nombro de liberaj kadroj devas resti
/// egala antaŭ kaj post la provo, do la detruo vere resorbas ĉion.
fn testi_procezan_reuzon(kerna: &Spaco, modulo: &Modulo) {
    let alokilo = kadra_alokilo();
    let antaŭ = alokilo.liberaj();
    let mut bone = true;
    for fojo in 1..=REUZOJ {
        let id = match procezo::krei(PROVA_PROCEZA_ID, kerna, alokilo) {
            Ok(id) => id,
            Err(eraro) => {
                crate::skribu!("[memoro] reuzo {} kreado malsukcesis: {}\n", fojo, eraro);
                bone = false;
                break;
            }
        };
        let muntita = match procezo::akiri(id) {
            Some(procezo) => uzantspaco::munti(procezo.spaco(), alokilo, modulo),
            None => {
                crate::skribu!("[memoro] reuzo {}: registro mankas\n", fojo);
                bone = false;
                break;
            }
        };
        if let Err(eraro) = muntita {
            crate::skribu!("[memoro] reuzo {} ŝargo malsukcesis: {}\n", fojo, eraro);
            bone = false;
        }
        if let Err(eraro) = procezo::detrui(id, alokilo) {
            crate::skribu!("[memoro] reuzo {} detruo malsukcesis: {}\n", fojo, eraro);
            bone = false;
        }
        if !bone {
            break;
        }
    }
    let post = alokilo.liberaj();
    if bone && post == antaŭ {
        crate::skribu!(
            "[memoro] reuzo (bone): {} cikloj, {} kadroj antaŭ, {} post\n",
            REUZOJ,
            antaŭ,
            post
        );
    } else {
        crate::skribu!(
            "[memoro] reuzo (MALBONE): {} kadroj antaŭ, {} post\n",
            antaŭ,
            post
        );
    }
}

/// Kreu la procezon, ŝarĝu la bildon en ĝian privatan spacon kaj planu ĝian
/// fadenon. La voko revenas, kiam la procezo mortis — per `SYS_FINI` aŭ per
/// mortiga escepto — kaj la planisto denove elektis la ripozan kuntekston.
fn lanĉi_uzantan_procezon(kerna: &Spaco, modulo: &Modulo, proceza_id: u64) -> bool {
    let enirpunkto = {
        let alokilo = kadra_alokilo();
        let id = match procezo::krei(proceza_id, kerna, alokilo) {
            Ok(id) => id,
            Err(eraro) => {
                crate::skribu!("[procezo] {} kreado malsukcesis: {}\n", proceza_id, eraro);
                return false;
            }
        };
        let Some(procezo) = procezo::akiri(id) else {
            crate::skribu!("[procezo] {}: registro mankas\n", id);
            return false;
        };
        let stato = procezo.stato();
        let kerna_pinto = procezo.kerna_stako_pinto();
        let radiko = procezo.radika_adreso();
        let proceza_spaco = procezo.spaco();
        if let Err(eraro) = uzantspaco::pravalorizi(proceza_spaco, alokilo, modulo) {
            crate::skribu!("[uzanto] ŝargo malsukcesis: {}\n", eraro);
            let _ = procezo::detrui(id, alokilo);
            return false;
        }
        let (bilda_komenco, _) = uzantspaco::bildaj_limoj();
        let kerna_mapo = proceza_spaco.traduki(VirtuaAdreso::nova(0x1000)).is_some();
        let privata = kerna.traduki(VirtuaAdreso::nova(bilda_komenco)).is_none();
        crate::skribu!(
            "[procezo] {}: {:?}, kerna stako 0x{:x}, PML4 0x{:x}, kerna mapo {}, bildo nur en la proceza spaco {}\n",
            id,
            stato,
            kerna_pinto,
            radiko.adreso(),
            if kerna_mapo { "kunhavata" } else { "MANKANTA" },
            if privata { "jes" } else { "NE" }
        );
        let Some(enirpunkto) = uzantspaco::enirpunkto() else {
            crate::skribu!("[uzanto] enirpunkto mankas; Ring 3 ne ekas\n");
            let _ = procezo::detrui(id, alokilo);
            return false;
        };
        let fadeno =
            planisto::pretigi_fadenon(id, id, VirtuaAdreso::nova(kerna_pinto), uzanta_fadeno);
        if let Err(eraro) = planisto::aldoni_fadenon(fadeno) {
            crate::skribu!("[procezo] {} fadena kreado malsukcesis: {:?}\n", id, eraro);
            let _ = procezo::detrui(id, alokilo);
            return false;
        }
        enirpunkto
    };

    crate::skribu!(
        "[kerno] horizonto ekas en Ring 3: CS=0x23, SS=0x1B, RFLAGS=0x202, enirpunkto 0x{:x}\n",
        enirpunkto
    );
    planisto::lanĉi();
    true
}

fn uzanta_fadeno(id: u64) {
    let Some(enirpunkto) = uzantspaco::enirpunkto() else {
        planisto::morti();
    };
    let reĝimo = komenca_reĝimo(id);
    // SAFETY: `uzantspaco::pravalorizi` muntis la bildon kun bito `U` kaj
    // kontrolis la enirpunkton; la stako estas la mapita 64 KiB-a regiono.
    unsafe { crate::arkitekturo::transiri_al_ring3(enirpunkto, UZANTA_STAKO_SUPRO, reĝimo) }
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
