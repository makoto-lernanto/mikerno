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
use libmikerno::starto::{REĜIMO_SEKRETO, REĜIMO_SPIONO};

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

pub fn ĉefa(informo: EniraInformo, bazo: u64, origino: u64) {
    crate::skribu!("[MiKerno] Saluton, mi estas MiKerno v0.3.2!\n");
    if bazo == origino {
        crate::skribu!(
            "[kerno] KASLR malŝaltita: la bildo restas ĉe 0x{:x}\n",
            bazo
        );
    } else {
        crate::skribu!(
            "[kerno] KASLR: kerna bazo 0x{:x} (deŝovo 0x{:x}); la bildo venis de 0x{:x}\n",
            bazo,
            bazo - 0x10_0000,
            origino
        );
    }
    let trajtoj = crate::arkitekturo::procesoraj_trajtoj();
    crate::skribu!("[kerno] SMEP subtenata: {}\n", jes_ne(trajtoj.smep));
    crate::skribu!("[kerno] SMAP subtenata: {}\n", jes_ne(trajtoj.smap));
    if crate::arkitekturo::smep_aktiva() {
        crate::skribu!("[kerno] SMEP ŝaltita; Ring 0 ne povas plenumi uzantajn paĝojn.\n");
    }
    if crate::arkitekturo::smap_aktiva() {
        crate::skribu!(
            "[kerno] SMAP ŝaltita; Ring 0 legas uzantajn bufrojn nur en STAC-fenestroj.\n"
        );
    }
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
        alokilo.pravalorizi(informo.memora_mapo, informo.rezervindaj, origino);
        testi_liberigon(alokilo);

        let mut spaco = match crate::arkitekturo::krei_adresan_spacon(alokilo) {
            Ok(spaco) => spaco,
            Err(eraro) => {
                crate::skribu!("[kerno] adresa spaco malsukcesis: {}\n", eraro);
                return;
            }
        };
        // Moduloj povas kuŝi en regiono, kiun la memormapo ne markas uzebla; la
        // identeca mapo tiam mankus. Mapu ilin unue kiel nurlegajn: la sekva
        // identeca mapado respektas la jamajn paĝojn, do la ELF-bildo restas
        // R+NX anstataŭ fariĝi skribebla RAM. La bildo estas datumoj por la
        // nukleo, ne kodo de Ring 0.
        for modulo in informo.moduloj {
            if let Err(eraro) = spaco.mapigi_intervalon(
                VirtuaAdreso::nova(modulo.komenco),
                VirtuaAdreso::nova(modulo.fino()),
                PaĝaPermesoj::KERNA_LEGA,
                alokilo,
            ) {
                crate::skribu!("[kerno] modula mapado malsukcesis: {}\n", eraro);
                return;
            }
        }
        if let Err(eraro) = spaco.idente_mapigi(informo.memora_mapo, alokilo) {
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
        testi_w_x(&spaco, informo.moduloj.first());
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

    // La du `horizonto`-procezoj estas registritaj kune; unu `lanĉi()` lasas la
    // planiston interplekti ilin kaj ŝalti `CR3` inter iliaj fadenoj. La
    // spionprocezo legas paĝon, kiu ekzistas nur en la viktima spaco: la `#PF`
    // mortigas nur ĝin, kaj la viktimo pluvivas ĝis `SYS_FINI`.
    if let Some(modulo) = modulo {
        let antaŭ = {
            let alokilo = kadra_alokilo();
            alokilo.liberaj()
        };
        let viktimo = prepari_uzantan_procezon(&spaco, modulo, UZANTA_FADENO_ID);
        let spiono = prepari_uzantan_procezon(&spaco, modulo, SPIONA_FADENO_ID);
        if viktimo && spiono {
            crate::skribu!("[kerno] du procezoj pretaj; unu planista voko\n");
            planisto::lanĉi();
            let post = {
                let alokilo = kadra_alokilo();
                alokilo.liberaj()
            };
            crate::skribu!(
                "[memoro] izolado ({}): {} kadroj antaŭ, {} post\n",
                if post == antaŭ { "bone" } else { "MALBONE" },
                antaŭ,
                post
            );
        } else {
            crate::skribu!("[kerno] uzantaj procezoj ne pretiĝis; scenaro ne ekas\n");
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

    // La memprovo venas laste: ĝi intence elvokas Ring 0 `#PF` kaj haltigas la
    // kernon, do ĉiuj antaŭaj scenaroj jam raportis sian rezulton. Se SMAP
    // mankas sed SMEP ekzistas, la pli malnova SMEP-provo ankoraŭ ruliĝas.
    if crate::arkitekturo::smap_aktiva() {
        crate::arkitekturo::testi_smap(&mut spaco, kadra_alokilo());
    } else if crate::arkitekturo::smep_aktiva() {
        crate::arkitekturo::testi_smep(&mut spaco, kadra_alokilo());
    } else {
        crate::skribu!(
            "[memoro] firmiga provo preterlasita: la procesoro ne subtenas SMEP aŭ SMAP\n"
        );
    }
    crate::arkitekturo::halti()
}

fn jes_ne(valoro: bool) -> &'static str {
    if valoro { "jes" } else { "ne" }
}

const STAKA_GRANDE: usize = 16 * 1024;
const TESTAJ_FADENOJ: usize = 3;
/// La fadeno — kaj samnoma procezo — de la viktima `horizonto`.
const UZANTA_FADENO_ID: u64 = TESTAJ_FADENOJ as u64 + 1;
/// La spionprocezo: ĝi legas la duan sekretan paĝon, kiu ekzistas nur en la
/// viktima spaco.
const SPIONA_FADENO_ID: u64 = UZANTA_FADENO_ID + 1;
/// Ripeta ciklo de la reuzoprovo; libera identigilo, ne fadeno.
const PROVA_PROCEZA_ID: u64 = 6;
const REUZOJ: usize = 10;

/// La scenaro, kiun `horizonto` plenumas: kontrolo de la propra sekreta paĝo
/// aŭ spionado de la viktima dua paĝo.
fn komenca_reĝimo(fadeno: u64) -> u64 {
    if fadeno == SPIONA_FADENO_ID {
        REĜIMO_SPIONO
    } else {
        REĜIMO_SEKRETO
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

/// Kontrolu la efektivajn W^X-permesojn de la kerna bildo kaj de la modulo.
/// La valoroj estas legitaj el la paĝtabeloj de `spaco`, do la linio pruvas la
/// bitojn `W`/`NX`, ne nur la intencon de la mapigaj alvokoj.
fn testi_w_x(spaco: &Spaco, modulo: Option<&Modulo>) {
    let limoj = crate::memoro::virtuala::kernaj_limoj();
    let kodo = spaco.permesoj(VirtuaAdreso::nova(limoj.kodo.0));
    let rodata = spaco.permesoj(VirtuaAdreso::nova(limoj.rodata.0));
    let datumoj = spaco.permesoj(VirtuaAdreso::nova(limoj.datumoj.0));
    let modula = modulo.and_then(|m| spaco.permesoj(VirtuaAdreso::nova(m.komenco)));

    let kodo_bone = matches!(kodo, Some(p) if p.plenumebla && !p.skribebla && !p.uzanta);
    let rodata_bone = matches!(rodata, Some(p) if !p.plenumebla && !p.skribebla && !p.uzanta);
    let datumoj_bone = matches!(datumoj, Some(p) if p.skribebla && !p.plenumebla && !p.uzanta);
    let modulo_bone = match (modulo, modula) {
        (None, _) => true,
        (Some(_), Some(p)) => !p.plenumebla && !p.skribebla && !p.uzanta,
        (Some(_), None) => false,
    };
    let bone = kodo_bone && rodata_bone && datumoj_bone && modulo_bone;
    crate::skribu!(
        "[memoro] W^X ({}): .text {}, .rodata {}, .data/.bss {}, modulo {}\n",
        if bone { "bone" } else { "MALBONE" },
        Priskribo(kodo),
        Priskribo(rodata),
        Priskribo(datumoj),
        Priskribo(modula)
    );
}

/// Prezento de `Option<PaĝaPermesoj>` por la W^X-linio.
struct Priskribo(Option<PaĝaPermesoj>);

impl core::fmt::Display for Priskribo {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.0 {
            Some(permesoj) => write!(f, "{permesoj}"),
            None => f.write_str("mankas"),
        }
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

/// Kreu la procezon, ŝarĝu la bildon, mapu la sekretajn paĝojn kaj registru
/// ĝian fadenon. La procezo restas `Preta`; la alvokanto lanĉas ĉiujn uzantajn
/// procezojn per unu `planisto::lanĉi()`, por ke la planisto interplektu ilin.
fn prepari_uzantan_procezon(kerna: &Spaco, modulo: &Modulo, proceza_id: u64) -> bool {
    let id = match procezo::krei(proceza_id, kerna, kadra_alokilo()) {
        Ok(id) => id,
        Err(eraro) => {
            crate::skribu!("[procezo] {} kreado malsukcesis: {}\n", proceza_id, eraro);
            return false;
        }
    };

    let muntado = {
        let alokilo = kadra_alokilo();
        let Some(procezo) = procezo::akiri(id) else {
            crate::skribu!("[procezo] {}: registro mankas\n", id);
            return false;
        };
        uzantspaco::pravalorizi(procezo.spaco(), alokilo, modulo)
    };
    let bildo = match muntado {
        Ok(bildo) => bildo,
        Err(eraro) => {
            crate::skribu!("[uzanto] ŝargo malsukcesis: {}\n", eraro);
            let _ = procezo::detrui(id, kadra_alokilo());
            return false;
        }
    };

    let sekreta = {
        let alokilo = kadra_alokilo();
        let Some(procezo) = procezo::akiri(id) else {
            crate::skribu!("[procezo] {}: registro mankas\n", id);
            return false;
        };
        procezo.agordi_bildon(bildo.komenco, bildo.fino, bildo.enirpunkto);
        let dua = proceza_id == UZANTA_FADENO_ID;
        match uzantspaco::mapi_sekreton(procezo.spaco(), alokilo, id, dua) {
            Ok(adreso) => {
                procezo.agordi_sekretan(adreso);
                Ok(adreso)
            }
            Err(eraro) => Err(eraro),
        }
    };
    if let Err(eraro) = sekreta {
        crate::skribu!("[procezo] {} sekreta paĝo malsukcesis: {}\n", id, eraro);
        let _ = procezo::detrui(id, kadra_alokilo());
        return false;
    }

    let (stato, kerna_pinto, radiko, enirpunkto, kerna_mapo, privata, sekreta) = {
        let Some(procezo) = procezo::akiri(id) else {
            crate::skribu!("[procezo] {}: registro mankas\n", id);
            return false;
        };
        let kerna_mapo = procezo
            .spaco()
            .traduki(VirtuaAdreso::nova(0x1000))
            .is_some();
        let privata = kerna.traduki(VirtuaAdreso::nova(bildo.komenco)).is_none();
        (
            procezo.stato(),
            procezo.kerna_stako_pinto(),
            procezo.radika_adreso(),
            procezo.enirpunkto(),
            kerna_mapo,
            privata,
            procezo.sekreta_adreso(),
        )
    };
    let Some(enirpunkto) = enirpunkto else {
        crate::skribu!("[uzanto] enirpunkto mankas; Ring 3 ne ekas\n");
        let _ = procezo::detrui(id, kadra_alokilo());
        return false;
    };
    crate::skribu!(
        "[procezo] {}: {:?}, kerna stako 0x{:x}, PML4 0x{:x}, kerna mapo {}, bildo nur en la proceza spaco {}, sekreta paĝo 0x{:x}\n",
        id,
        stato,
        kerna_pinto,
        radiko.adreso(),
        if kerna_mapo { "kunhavata" } else { "MANKANTA" },
        if privata { "jes" } else { "NE" },
        sekreta
    );

    let fadeno = planisto::pretigi_fadenon(id, id, VirtuaAdreso::nova(kerna_pinto), uzanta_fadeno);
    if let Err(eraro) = planisto::aldoni_fadenon(fadeno) {
        crate::skribu!("[procezo] {} fadena kreado malsukcesis: {:?}\n", id, eraro);
        let _ = procezo::detrui(id, kadra_alokilo());
        return false;
    }
    crate::skribu!(
        "[kerno] horizonto {} pretas: CS=0x23, SS=0x1B, RFLAGS=0x202, enirpunkto 0x{:x}\n",
        id,
        enirpunkto
    );
    true
}

fn uzanta_fadeno(id: u64) {
    let enirpunkto = {
        let Some(procezo) = procezo::akiri(id) else {
            planisto::morti();
        };
        procezo.enirpunkto()
    };
    let Some(enirpunkto) = enirpunkto else {
        planisto::morti();
    };
    let reĝimo = komenca_reĝimo(id);
    // SAFETY: `pravalorizi` muntis la bildon kun bito `U` kaj registris ĝian
    // enirpunkton en la proceza ero; la stako estas la mapita 64 KiB-a regiono,
    // kaj `id` estas la proceza identigilo skribita en la sekretan paĝon.
    unsafe { crate::arkitekturo::transiri_al_ring3(enirpunkto, UZANTA_STAKO_SUPRO, reĝimo, id) }
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
