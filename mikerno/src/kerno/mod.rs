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
    crate::skribu!(
        "[MiKerno] Saluton, mi estas MiKerno v{}!\n",
        env!("CARGO_PKG_VERSION")
    );
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
    let firmigo = crate::arkitekturo::firmiga_raporto();
    for trajto in firmigo.trajtoj() {
        crate::skribu!(
            "[kerno] {} subtenata: {}\n",
            trajto.nomo,
            if trajto.subtenata { "jes" } else { "ne" }
        );
    }
    for trajto in firmigo.trajtoj() {
        if trajto.aktiva {
            crate::skribu!("[kerno] {} ŝaltita; {}\n", trajto.nomo, trajto.priskribo);
        }
    }
    crate::arkitekturo::instali_sistemvokon();
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
    registri_testajn_finpunktojn();
    ipc::testi_tabelojn();
    ipc::testi_kapablojn();
    ipc::testi_revokon();

    let mesago = ipc::Mesago::nova(1, 0, 2, "saluton el la praŝarĝo".as_bytes());
    match ipc::sendi_mesagon(2, &mesago) {
        Ok(()) => crate::skribu!("[kerno] IPC: mesaĝo sendita al fadeno 2\n"),
        Err(eraro) => crate::skribu!("[kerno] IPC: sendado malsukcesis: {:?}\n", eraro),
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

    let stat = ipc::nuna_statistiko();
    crate::skribu!(
        "[ipc] statistiko ({}): senditaj={}, ricevitaj={}, premregoj={}, revokoj={}\n",
        if stat.premregoj >= 2 && stat.revokoj >= 2 {
            "bone"
        } else {
            "MALBONE"
        },
        stat.senditaj,
        stat.ricevitaj,
        stat.premregoj,
        stat.revokoj
    );

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

/// Donu al ĉiu kerna testfadeno defaŭltan finpunkton; la kernaj fadenoj uzas
/// la internan `sendi_mesagon`/`ricevi_mesagon`-vojon (sen kapablotabelo).
fn registri_testajn_finpunktojn() {
    for id in 1..=TESTAJ_FADENOJ as u64 {
        if let Err(eraro) = ipc::registri_fadenon(id, 0) {
            crate::skribu!(
                "[ipc] fadeno {}: defaŭlta finpunkto malsukcesis: {:?}\n",
                id,
                eraro
            );
        }
    }
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

/// Detruu uzantan procezon kaj purigu ĝian IPC-staton; la finpunktoj, kiujn
/// ĝi posedas, estas revokitaj antaŭ ol la adresa spaco malaperas.
fn detrui_uzantan(id: u64) {
    ipc::purigi_procezon(id);
    let _ = procezo::detrui(id, kadra_alokilo());
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
            detrui_uzantan(id);
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
        let sekreta = uzantspaco::mapi_sekreton(procezo.spaco(), alokilo, id, dua);
        if let Ok(adreso) = sekreta {
            procezo.agordi_sekretan(adreso);
        }
        sekreta
    };
    if let Err(eraro) = sekreta {
        crate::skribu!("[procezo] {} sekreta paĝo malsukcesis: {}\n", id, eraro);
        detrui_uzantan(id);
        return false;
    }

    // La defaŭlta finpunkto ekzistas antaŭ ol la fadeno eniras la planiston, do
    // la malnova `SENDI`/`RICEVI`-vojo kaj la kapabla fendo `0` pretas ekde la
    // unua Ring 3-instrukcio.
    if let Err(eraro) = ipc::registri_fadenon(id, id) {
        crate::skribu!(
            "[uzanto] {}: defaŭlta finpunkto malsukcesis: {:?}\n",
            id,
            eraro
        );
        detrui_uzantan(id);
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
        detrui_uzantan(id);
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
        detrui_uzantan(id);
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
    unsafe { crate::arkitekturo::eniri_uzanton(enirpunkto, UZANTA_STAKO_SUPRO, reĝimo, id) }
}

fn testa_fadeno(id: u64) {
    if id == 1 {
        sistemvoko::testi_ring0();
        premrega_sendo();
    } else if id == 2 {
        match ipc::ricevi_mesagon(0) {
            Ok(mesago) => crate::skribu!(
                "[fadeno 2] ricevis mesaĝon: tipo {}, ricevonto {} (finpunkta id), \"{}\"\n",
                mesago.tipo,
                mesago.ricevonto,
                teksto(&mesago)
            ),
            Err(eraro) => crate::skribu!(
                "[fadeno 2] ricevo malsukcesis: {:?} ({})\n",
                eraro,
                eraro.kodo()
            ),
        }
        premrega_ricevo();
        // Plenigu la vicon de fadeno 1 kaj bloku sur la 17-a sendo; fadeno 3
        // revokas tiun finpunkton post la revoka veko de fadeno 1.
        let mesago = ipc::Mesago::nova(78, 0, 0, &[]);
        let mut enmetitaj = 0;
        for _ in 0..ipc::MAKSAJ_MESAGOJ {
            if ipc::sendi_mesagon(1, &mesago).is_ok() {
                enmetitaj += 1;
            }
        }
        let kodo = ipc::sendi_mesagon(1, &mesago)
            .err()
            .map_or(0, ipc::IpcEraro::kodo);
        crate::skribu!(
            "[fadeno 2] sendblokada revoko ({}): enmetitaj={}, kodo={}\n",
            if enmetitaj == ipc::MAKSAJ_MESAGOJ && kodo == -32 {
                "bone"
            } else {
                "MALBONE"
            },
            enmetitaj,
            kodo
        );
    } else if id == 3 {
        // Fadeno 1 revokas ĉi tiun defaŭltan finpunkton dum la blokado; la
        // revoka veko devas liveri `-32`, ne pendigi aŭ redoni `-9`.
        match ipc::ricevi_mesagon(1) {
            Ok(mesago) => crate::skribu!(
                "[fadeno 3] neatendita mesaĝo: ricevonto {}, \"{}\"\n",
                mesago.ricevonto,
                teksto(&mesago)
            ),
            Err(eraro) => {
                crate::skribu!(
                    "[fadeno 3] revoka veko ({}): kodo {}\n",
                    if eraro == ipc::IpcEraro::Revokita {
                        "bone"
                    } else {
                        "MALBONE"
                    },
                    eraro.kodo()
                );
                if eraro == ipc::IpcEraro::Revokita {
                    // Fadeno 2 atendas lokon en la plena vico de finpunkto 1;
                    // revoko redonas al ĝi `-32` anstataŭ eterna atendo.
                    match ipc::revoki_defaŭltan(1) {
                        Ok(()) => crate::skribu!(
                            "[fadeno 3] revokis finpunkton 1; vekas la blokitan sendanton\n"
                        ),
                        Err(eraro) => crate::skribu!(
                            "[fadeno 3] revoko de finpunkto 1 malsukcesis: {:?} ({})\n",
                            eraro,
                            eraro.kodo()
                        ),
                    }
                }
            }
        }
    }

    for fojo in 1..=10 {
        crate::skribu!("[fadeno {}] kuranta (fojo {})\n", id, fojo);
        for n in 0..300_000u64 {
            core::hint::black_box(n);
        }
        if id == 1 && fojo == 3 {
            match ipc::revoki_defaŭltan(3) {
                Ok(()) => {
                    crate::skribu!("[fadeno 1] revokis la defaŭltan finpunkton de fadeno 3\n")
                }
                Err(eraro) => {
                    crate::skribu!("[fadeno 1] revoko de fadeno 3 malsukcesis: {:?}\n", eraro)
                }
            }
        }
        planisto::cedi();
    }
    crate::skribu!("[fadeno {}] finas sian laboron\n", id);
}

/// Fadeno 1 plenigas la vicon de fadeno 2 per `MAKSAJ_MESAGOJ` mesaĝoj; la
/// sekva sendo blokas ĝis la ricevanto liberigas lokon. La mesaĝo mem portas
/// falsan `sendinto` (`999`), kiun la kerno devas anstataŭigi.
fn premrega_sendo() {
    let mesago = ipc::Mesago::nova(77, 999, 0, "premrego".as_bytes());
    let mut senditaj = 0;
    for _ in 0..=ipc::MAKSAJ_MESAGOJ {
        if ipc::sendi_mesagon(2, &mesago).is_ok() {
            senditaj += 1;
        }
    }
    crate::skribu!(
        "[fadeno 1] premrego ({}): {} mesaĝoj senditaj; la lasta atendis liberan lokon\n",
        if senditaj == ipc::MAKSAJ_MESAGOJ + 1 {
            "bone"
        } else {
            "MALBONE"
        },
        senditaj
    );
}

/// Fadeno 2 malplenigas la vicon; la identecon fiksas la kerno, do `sendinto`
/// estas la senda fadeno (`1`), ne la `999` de la mesaĝo, kaj `ricevonto` estas
/// la finpunkta identigilo (`2`).
fn premrega_ricevo() {
    let mut nombro = 0;
    let mut idento = (0u64, 0u64);
    for _ in 0..=ipc::MAKSAJ_MESAGOJ {
        match ipc::ricevi_mesagon(1) {
            Ok(mesago) => {
                nombro += 1;
                idento = (mesago.sendinto, mesago.ricevonto);
            }
            Err(_) => break,
        }
    }
    let bone = nombro == ipc::MAKSAJ_MESAGOJ + 1 && idento == (1, 2);
    crate::skribu!(
        "[fadeno 2] premrego ({}): {} mesaĝoj, idento sendinto={}, ricevonto={}\n",
        if bone { "bone" } else { "MALBONE" },
        nombro,
        idento.0,
        idento.1
    );
}

fn teksto(mesago: &ipc::Mesago) -> &str {
    core::str::from_utf8(mesago.enhavo()).unwrap_or("<nevalida>")
}
