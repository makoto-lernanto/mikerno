//! Kerna nukleo — arkitekture sendependa; la platformon ĝi tuŝas nur tra
//! `crate::arkitekturo`.

use core::sync::atomic::{AtomicU64, Ordering};

use crate::arkitekturo::Spaco;
use crate::interrompo::InterrompaGardo;
use crate::kunhavigita::Kunhavigita;
use crate::memoro::EniraInformo;
use crate::memoro::KADRA_GRANDO;
use crate::memoro::Modulo;
use crate::memoro::kadro::KadraAlokilo;
use crate::memoro::virtuala::{
    AdresaSpaco, PaĝaPermesoj, PaĝaTablo, UZANTA_BAZO, UZANTA_STAKO_SUPRO, VirtuaAdreso,
};
use libmikerno::starto::{REĜIMO_SEKRETO, REĜIMO_SPIONO};

pub mod exec;
pub mod ik;
pub mod objekto;
pub mod planisto;
pub mod procezo;
pub mod sistemvoko;
pub mod uzantspaco;
mod vekotemporizilo;

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
    #[cfg(target_arch = "x86_64")]
    crate::arkitekturo::testi_ac();
    testi_gardojn();
    crate::arkitekturo::instali_sistemvokon();
    crate::skribu!(
        "[kerno] memoro: {} regionoj, {} MiB uzeblaj\n",
        informo.memora_mapo.regionoj().len(),
        informo.memora_mapo.uzebla_memoro() / (1024 * 1024)
    );

    // La alokilo restas prunteprenita nur en ĉi tiu bloko: la planisto
    // aliras ĝin mem dum la proceza detruo, do longa `&mut` malhelpus.
    let (mut spaco, frambufo) = {
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
        // Se la praŝarĝilo ne liveras frambufferion, la platformo povas krei
        // sian propran (arm64: RAM-bufro anoncita al `ramfb`).
        let frambufo = informo
            .frambufo
            .or_else(|| crate::arkitekturo::krei_frambufon(alokilo));
        let frambufo = match frambufo {
            Some(fb) => {
                let fino = fb.bazo + fb.paŝo as u64 * fb.alto as u64;
                // MMIO-frambufferio (x86) ankoraŭ mankas en la identeca mapo;
                // RAM-frambufferio (arm64) jam estas identece mapita.
                let jam_mapita = fb.bazo != 0
                    && fb.paŝo != 0
                    && spaco.traduki(VirtuaAdreso::nova(fb.bazo)).is_some()
                    && spaco
                        .traduki(VirtuaAdreso::nova(fino.saturating_sub(1)))
                        .is_some();
                if jam_mapita {
                    Some(fb)
                } else {
                    match spaco.mapigi_intervalon(
                        VirtuaAdreso::nova(fb.bazo),
                        VirtuaAdreso::nova(fino),
                        PaĝaPermesoj::SKRIBA_SENKAŜE,
                        alokilo,
                    ) {
                        Ok(()) => Some(fb),
                        Err(eraro) => {
                            crate::skribu!("[kerno] frambufa mapado malsukcesis: {}\n", eraro);
                            None
                        }
                    }
                }
            }
            None => None,
        };
        testi_w_x(&spaco, informo.moduloj.first());
        spaco.ŝalti();
        procezo::agordi_kernan_spacon(spaco.radika_adreso());
        (spaco, frambufo)
    };
    #[cfg(target_arch = "x86_64")]
    testi_cr3(spaco.radika_adreso().adreso());

    match frambufo {
        Some(fb) if crate::ekrano::pravalorizi(fb) => crate::skribu!(
            "[kerno] frambufo {}x{}, {} bitoj/pikselo\n",
            fb.larĝo,
            fb.alto,
            fb.bitoj_po_pikselo
        ),
        Some(_) => crate::skribu!("[kerno] frambufo ne taŭgas; nur {}\n", SERIA_KONZOLO),
        None => crate::skribu!("[kerno] frambufo ne havebla; nur {}\n", SERIA_KONZOLO),
    }

    crate::arkitekturo::inicialigi_kernostakon();

    #[cfg(target_arch = "x86_64")]
    {
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
    }
    #[cfg(target_arch = "aarch64")]
    {
        crate::arkitekturo::pravalorizi_temporizilon();
        crate::skribu!("[kerno] GICv2 kaj paĝado pretaj; temporizilo 100 Hz\n");
    }

    match crate::konservado::pravalorizi(&mut spaco, kadra_alokilo()) {
        Ok(()) => {
            if let Err(eraro) = crate::konservado::snds::pravalorizi() {
                crate::skribu!("[dosiersistemo] SNDS (MALBONE): {eraro}\n");
            }
        }
        Err(eraro) => crate::skribu!("[konservado] virtio-blk (MALBONE): {eraro}\n"),
    }

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
    ik::testi_tabelojn();
    ik::testi_kapablojn();
    ik::testi_revokon();

    let mesago = ik::Mesago::nova(1, 0, 2, "saluton el la praŝarĝo".as_bytes());
    match ik::sendi_mesagon(2, &mesago) {
        Ok(()) => crate::skribu!("[kerno] IK: mesaĝo sendita al fadeno 2\n"),
        Err(eraro) => crate::skribu!("[kerno] IK: sendado malsukcesis: {:?}\n", eraro),
    }

    crate::skribu!(
        "[kerno] planisto ekas: {} fadenoj, antaŭokupo ĉiu 10 ms\n",
        TESTAJ_FADENOJ
    );
    planisto::lanĉi();

    // La du `horizonto`-procezoj estas registritaj kune; unu `lanĉi()` lasas la
    // planiston interplekti ilin kaj ŝalti `CR3` inter iliaj fadenoj. La
    // spionprocezo legas paĝon, kiu ekzistas nur en la viktima spaco: la `#PF`
    // mortigas nur ĝin, kaj la viktimo pluvivas ĝis `SYS_FINI`. La viktimo
    // povas esti blokita (tempolimo) kiam la spiono mortas, do `lanĉi()`
    // revenas antaŭ ĝia fino; la kadrokontrolo kaj la statistiko sekvas post
    // la observa periodo, kiam ambaŭ procezoj jam malaperis.
    let mut izolado_antaŭ = None;
    if let Some(modulo) = modulo {
        let antaŭ = {
            let alokilo = kadra_alokilo();
            alokilo.liberaj()
        };
        let viktimo = prepari_uzantan_procezon(&spaco, modulo, UZANTA_FADENO_ID);
        let spiono = prepari_uzantan_procezon(&spaco, modulo, SPIONA_FADENO_ID);
        if viktimo && spiono {
            crate::skribu!("[kerno] du procezoj pretaj; unu planista voko\n");
            izolado_antaŭ = Some(antaŭ);
            planisto::lanĉi();
        } else {
            crate::skribu!("[kerno] uzantaj procezoj ne pretiĝis; scenaro ne ekas\n");
        }
    }

    crate::arkitekturo::malebligi_interrompojn();
    crate::skribu!("[kerno] planisto revenis al la ripoza fadeno\n");

    // Observu la sistemon dek sekundojn (1000 tikoj) post la fadenoj.
    crate::interrompo::ebligi_interrompojn();
    loop {
        if crate::arkitekturo::temporizilaj_tikoj() >= 1000 {
            break;
        }
        crate::arkitekturo::atendi_interrompon();
    }
    crate::interrompo::malebligi_interrompojn();
    let tikoj = crate::arkitekturo::temporizilaj_tikoj();

    // Ĉiu temporizila interrompo devas ricevi EOI; mankanta EOI lasus la
    // APIC-ISR-biton ŝaltita kaj sufokus la sekvajn vekojn.
    #[cfg(target_arch = "x86_64")]
    {
        let konfirmoj = crate::arkitekturo::konfirmoj_interrompoj();
        crate::skribu!(
            "[apic] EOI ({}): {} konfirmoj, {} tikoj\n",
            if konfirmoj >= tikoj {
                "bone"
            } else {
                "MALBONE"
            },
            konfirmoj,
            tikoj
        );
    }

    if let Some(antaŭ) = izolado_antaŭ {
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
    }

    let stat = ik::nuna_statistiko();
    crate::skribu!(
        "[ik] statistiko ({}): senditaj={}, ricevitaj={}, premregoj={}, revokoj={}, tempolimoj={}\n",
        if stat.premregoj >= 2 && stat.revokoj >= 2 && stat.tempolimoj >= 3 {
            "bone"
        } else {
            "MALBONE"
        },
        stat.senditaj,
        stat.ricevitaj,
        stat.premregoj,
        stat.revokoj,
        stat.tempolimoj
    );

    let (mortintaj, pretaj, blokitaj) = planisto::raporto();
    crate::skribu!(
        "[kerno] fina stato: {} mortintaj, {} pretaj, {} blokitaj; {} tikoj\n",
        mortintaj,
        pretaj,
        blokitaj,
        tikoj
    );

    let stat = planisto::statistiko();
    let veturoj = planisto::veturoj();
    let minimumo = veturoj
        .iter()
        .take(TESTAJ_FADENOJ + 1)
        .skip(1)
        .copied()
        .min()
        .unwrap_or(0);
    let bone = minimumo >= 10 && stat.vekigoj >= 4 && stat.maks_vekprokrasto <= 1;
    crate::skribu!(
        "[planisto] statistiko ({}): ŝaltoj={}, limdataj_blokoj={}, vekigoj={}, maks_vekprokrasto={} tikoj\n",
        if bone { "bone" } else { "MALBONE" },
        stat.ŝaltoj,
        stat.limdataj_blokoj,
        stat.vekigoj,
        stat.maks_vekprokrasto
    );
    crate::skribu!("[planisto] veturoj:");
    for veturo in veturoj.iter().take(TESTAJ_FADENOJ + 1).skip(1) {
        crate::skribu!(" {}", veturo);
    }
    crate::skribu!("; minimumo {}\n", minimumo);

    let plenumtikoj = planisto::plenumtikoj();
    let sumo = plenumtikoj.iter().copied().fold(0u64, u64::saturating_add);
    let laboraj = plenumtikoj[1..]
        .iter()
        .copied()
        .fold(0u64, u64::saturating_add);
    let bone = sumo == tikoj
        && plenumtikoj[0] > 0
        && plenumtikoj[1..=TESTAJ_FADENOJ].iter().all(|&n| n > 0);
    crate::skribu!(
        "[planisto] plenumtempo ({}): sumo={}, horloĝo={}, laboraj={}, ripozaj={} tikoj; nominale 10 ms/specimeno\n",
        if bone { "bone" } else { "MALBONE" },
        sumo,
        tikoj,
        laboraj,
        plenumtikoj[0]
    );
    for (id, nombro) in plenumtikoj.iter().enumerate() {
        crate::skribu!("[planisto] plenumtikoj: fadeno {} = {}\n", id, nombro);
    }

    // La piksela fiksaĵo estas identa sur ambaŭ arkoj, kiam la frambufferio
    // havas la saman geometrion; la haketo permesas kompari la du bildojn el
    // la seria protokolo.
    match crate::ekrano::testi_pikselojn() {
        Some(raporto) => crate::skribu!(
            "[ekrano] piksela fiksaĵo {}x{}, {} bitoj/pikselo, tiparo {}x{}; haketo 0x{:016x}\n",
            raporto.larĝo,
            raporto.alto,
            raporto.bitoj_po_pikselo,
            raporto.tiparo.0,
            raporto.tiparo.1,
            raporto.haketo
        ),
        None => crate::skribu!("[ekrano] piksela fiksaĵo preterlasita: frambufferio mankas\n"),
    }

    // La memprovo venas laste: ĝi intence elvokas Ring 0 `#PF` aŭ `EL1`-an
    // datenan aborton kaj haltigas la kernon, do ĉiuj antaŭaj scenaroj jam
    // raportis sian rezulton. Se SMAP mankas sed SMEP ekzistas, la pli malnova
    // SMEP-provo ankoraŭ ruliĝas.
    #[cfg(target_arch = "x86_64")]
    if crate::arkitekturo::smap_aktiva() {
        crate::arkitekturo::testi_smap(&mut spaco, kadra_alokilo());
    } else if crate::arkitekturo::smep_aktiva() {
        crate::arkitekturo::testi_smep(&mut spaco, kadra_alokilo());
    } else {
        crate::skribu!(
            "[memoro] firmiga provo preterlasita: la procesoro ne subtenas SMEP aŭ SMAP\n"
        );
    }
    #[cfg(target_arch = "aarch64")]
    if crate::arkitekturo::pan_aktiva() {
        crate::arkitekturo::testi_pan(&mut spaco, kadra_alokilo());
    } else {
        crate::skribu!("[memoro] firmiga provo preterlasita: la procesoro ne subtenas PAN\n");
    }
    crate::arkitekturo::halti()
}

const STAKA_GRANDE: usize = 16 * 1024;
const TESTAJ_FADENOJ: usize = 4;
static ANTAŬOKUPAJ_PRETOJ: AtomicU64 = AtomicU64::new(0);
static ANTAŬOKUPA_KOMENCO: AtomicU64 = AtomicU64::new(0);
/// La dormodaŭroj de la testfadenoj en milisekundoj. Ili dormas kune ekde la
/// starto, do la kupo tenas kvar malsamajn limdatojn kaj la vekoj sekvas ilin:
/// 4 (30 ms), 2 (50 ms), 3 (70 ms), 1 (100 ms). La unua fadeno provas la
/// akceptkriterion de M16.
const DORMOJ_MS: [u64; TESTAJ_FADENOJ] = [100, 50, 70, 30];
/// La nomo de la radika tabelo en la proceza raporto.
#[cfg(target_arch = "x86_64")]
const RADIKA_NOMO: &str = "PML4";
#[cfg(target_arch = "aarch64")]
const RADIKA_NOMO: &str = "TTBR0";
/// La seria konzolo de la platformo, por la starta raporto.
#[cfg(target_arch = "x86_64")]
const SERIA_KONZOLO: &str = "COM1";
#[cfg(target_arch = "aarch64")]
const SERIA_KONZOLO: &str = "PL011";
/// La fadeno — kaj samnoma procezo — de la viktima `horizonto`.
const UZANTA_FADENO_ID: u64 = TESTAJ_FADENOJ as u64 + 1;
/// La spionprocezo: ĝi legas la duan sekretan paĝon, kiu ekzistas nur en la
/// viktima spaco.
const SPIONA_FADENO_ID: u64 = UZANTA_FADENO_ID + 1;
/// Ripeta ciklo de la reuzoprovo; libera identigilo, ne fadeno.
const PROVA_PROCEZA_ID: u64 = 7;
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

/// Pruvu la `CR3`-disciplinon post la unua ŝalto: `CR4.PCIDE` devas esti
/// malŝaltita, la aktiva radiko devas esti la ĵus konstruita kerna `PML4`, kaj
/// reŝalto al la sama radiko ne rajtas ŝanĝi ĝin.
#[cfg(target_arch = "x86_64")]
fn testi_cr3(radiko: u64) {
    let aktiva = crate::arkitekturo::aktiva_adreso_spaco();
    let sen_pcid = crate::arkitekturo::ne_pcide();
    // La sama `mov cr3`-vojo kiel ĉe la planisto; reŝalto al la nuna radiko
    // ne rajtas interrompi la plenumadon.
    unsafe { crate::arkitekturo::ŝalti_adresan_spacon(radiko) };
    let post = crate::arkitekturo::aktiva_adreso_spaco();
    let kongrua = aktiva == radiko && post == radiko;
    let aliniita = radiko & (KADRA_GRANDO - 1) == 0;
    let bone = sen_pcid && kongrua && aliniita;
    crate::skribu!(
        "[memoro] CR3 ({}): PCID={}, aktiva=0x{:x}, kongrua={}, aliniita={}\n",
        if bone { "bone" } else { "MALBONE" },
        if sen_pcid { "malŝaltita" } else { "AKTIVA" },
        radiko,
        kongrua,
        aliniita
    );
}

/// Pruvu la liberigan vojon de `InterrompaGardo`: ĝi malŝaltas interrompojn, kaj
/// `Drop` restarigas la antaŭan staton ankaŭ ĉe nestitaj gardoj.
fn testi_gardojn() {
    let antaŭa = crate::interrompo::interrompoj_ebligitaj();
    let ekstera = InterrompaGardo::nova();
    let malŝaltita = !crate::interrompo::interrompoj_ebligitaj();
    let interna = InterrompaGardo::nova();
    drop(interna);
    let interna_nenio = !crate::interrompo::interrompoj_ebligitaj();
    drop(ekstera);
    let restarigita = crate::interrompo::interrompoj_ebligitaj() == antaŭa;

    let bone = malŝaltita && interna_nenio && restarigita;
    crate::skribu!(
        "[kerno] gardoj ({}): malŝalto={}, interna={}, restarigo={} (IF antaŭe {})\n",
        if bone { "bone" } else { "MALBONE" },
        malŝaltita,
        interna_nenio,
        restarigita,
        antaŭa
    );
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
    planisto::testi_prioritatojn();
    planisto::testi_plenumtempon();
    let prioritatoj = [
        objekto::Prioritato::Malalta,
        objekto::Prioritato::Normala,
        objekto::Prioritato::Alta,
        objekto::Prioritato::Malalta,
    ];
    for (indekso, prioritato) in prioritatoj.into_iter().enumerate() {
        let id = indekso as u64 + 1;
        let pinto = &raw const STAKOJ as u64 + id * STAKA_GRANDE as u64;
        let mut fadeno = planisto::pretigi_fadenon(id, 0, VirtuaAdreso::nova(pinto), testa_fadeno);
        fadeno.prioritato = prioritato;
        planisto::aldoni_fadenon(fadeno)?;
    }
    Ok(())
}

/// Donu al ĉiu kerna testfadeno defaŭltan finpunkton; la kernaj fadenoj uzas
/// la internan `sendi_mesagon`/`ricevi_mesagon`-vojon (sen kapablotabelo).
fn registri_testajn_finpunktojn() {
    for id in 1..=TESTAJ_FADENOJ as u64 {
        if let Err(eraro) = ik::registri_fadenon(id, 0) {
            crate::skribu!(
                "[ik] fadeno {}: defaŭlta finpunkto malsukcesis: {:?}\n",
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

/// Detruu uzantan procezon kaj purigu ĝian IK-staton; la finpunktoj, kiujn
/// ĝi posedas, estas revokitaj antaŭ ol la adresa spaco malaperas.
fn detrui_uzantan(id: u64) {
    ik::purigi_procezon(id);
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
    if let Err(eraro) = ik::registri_fadenon(id, id) {
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
        "[procezo] {}: {:?}, kerna stako 0x{:x}, {} 0x{:x}, kerna mapo {}, bildo nur en la proceza spaco {}, sekreta paĝo 0x{:x}\n",
        id,
        stato,
        kerna_pinto,
        RADIKA_NOMO,
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
    #[cfg(target_arch = "x86_64")]
    crate::skribu!(
        "[kerno] horizonto {} pretas: CS=0x23, SS=0x1B, RFLAGS=0x202, enirpunkto 0x{:x}\n",
        id,
        enirpunkto
    );
    #[cfg(target_arch = "aarch64")]
    crate::skribu!(
        "[kerno] horizonto {} pretas: EL0t, SP_EL0=0x{:x}, SPSR=0x0, enirpunkto 0x{:x}\n",
        id,
        UZANTA_STAKO_SUPRO,
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
    sistemvoko::dorma_provo(id, DORMOJ_MS[id as usize - 1]);
    if id == 1 {
        sistemvoko::testi_ring0();
        // La unua tempomesaĝo por fadeno 4 venas antaŭ la premrego, do ĝi estas
        // en la vico antaŭ ol fadeno 4 ekatendas; la dua venas poste kaj devas
        // veki la senfinan atendo.
        let unua = ik::Mesago::nova(90, 0, 0, b"tempo-1");
        if let Err(eraro) = ik::sendi_mesagon(4, &unua) {
            crate::skribu!("[fadeno 1] unua tempomesaĝo malsukcesis: {:?}\n", eraro);
        }
        premrega_sendo();
    } else if id == 2 {
        match ik::ricevi_mesagon(0) {
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
        let mesago = ik::Mesago::nova(78, 0, 0, &[]);
        let mut enmetitaj = 0;
        for _ in 0..ik::MAKSAJ_MESAGOJ {
            if ik::sendi_mesagon(1, &mesago).is_ok() {
                enmetitaj += 1;
            }
        }
        let kodo = ik::sendi_mesagon(1, &mesago)
            .err()
            .map_or(0, ik::IkEraro::kodo);
        crate::skribu!(
            "[fadeno 2] sendblokada revoko ({}): enmetitaj={}, kodo={}\n",
            if enmetitaj == ik::MAKSAJ_MESAGOJ && kodo == -32 {
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
        match ik::ricevi_mesagon(1) {
            Ok(mesago) => crate::skribu!(
                "[fadeno 3] neatendita mesaĝo: ricevonto {}, \"{}\"\n",
                mesago.ricevonto,
                teksto(&mesago)
            ),
            Err(eraro) => {
                crate::skribu!(
                    "[fadeno 3] revoka veko ({}): kodo {}\n",
                    if eraro == ik::IkEraro::Revokita {
                        "bone"
                    } else {
                        "MALBONE"
                    },
                    eraro.kodo()
                );
                if eraro == ik::IkEraro::Revokita {
                    // Fadeno 2 atendas lokon en la plena vico de finpunkto 1;
                    // revoko redonas al ĝi `-32` anstataŭ eterna atendo.
                    match ik::revoki_defaŭltan(1) {
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
    } else if id == 4 {
        tempolima_provo(id);
    }

    for fojo in 1..=10 {
        crate::skribu!("[fadeno {}] kuranta (fojo {})\n", id, fojo);
        for n in 0..300_000u64 {
            core::hint::black_box(n);
        }
        if id == 1 && fojo == 5 {
            let dua = ik::Mesago::nova(91, 0, 0, b"tempo-2");
            if let Err(eraro) = ik::sendi_mesagon(4, &dua) {
                crate::skribu!("[fadeno 1] dua tempomesaĝo malsukcesis: {:?}\n", eraro);
            }
        }
        if id == 1 && fojo == 3 {
            match ik::revoki_defaŭltan(3) {
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
    antaŭokupa_provo(id);
    crate::skribu!("[fadeno {}] finas sian laboron\n", id);
}

// La barilo sen cedo povas pasi nur se la temporizilo antaŭokupas la fadenojn.
fn antaŭokupa_provo(id: u64) {
    ANTAŬOKUPAJ_PRETOJ.fetch_or(1 << (id - 1), Ordering::Relaxed);
    while ANTAŬOKUPAJ_PRETOJ.load(Ordering::Relaxed) != (1 << TESTAJ_FADENOJ) - 1 {
        core::hint::black_box(id);
    }
    let propono = crate::arkitekturo::temporizilaj_tikoj() + 1;
    let komenco =
        match ANTAŬOKUPA_KOMENCO.compare_exchange(0, propono, Ordering::Relaxed, Ordering::Relaxed)
        {
            Ok(_) => propono,
            Err(komenco) => komenco,
        };
    let mut lasta = u64::MAX;
    let mut observitaj = 0;
    loop {
        let tiko = crate::arkitekturo::temporizilaj_tikoj();
        if tiko >= komenco + 32 {
            break;
        }
        if tiko >= komenco && tiko != lasta {
            observitaj += 1;
            lasta = tiko;
        }
        core::hint::black_box(id);
    }
    crate::skribu!(
        "[fadeno {}] antaŭokupa laboro sen cedo ({}): {} observitaj tikoj\n",
        id,
        if observitaj >= 2 { "bone" } else { "MALBONE" },
        observitaj
    );
}

fn premrega_sendo() {
    let mesago = ik::Mesago::nova(77, 999, 0, "premrego".as_bytes());
    let mut senditaj = 0;
    for _ in 0..=ik::MAKSAJ_MESAGOJ {
        if ik::sendi_mesagon(2, &mesago).is_ok() {
            senditaj += 1;
        }
    }
    crate::skribu!(
        "[fadeno 1] premrego ({}): {} mesaĝoj senditaj; la lasta atendis liberan lokon\n",
        if senditaj == ik::MAKSAJ_MESAGOJ + 1 {
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
    loop {
        let premregoj = {
            let _gardo = InterrompaGardo::nova();
            ik::nuna_statistiko().premregoj
        };
        if premregoj != 0 {
            break;
        }
        planisto::cedi();
    }
    let mut nombro = 0;
    let mut idento = (0u64, 0u64);
    for _ in 0..=ik::MAKSAJ_MESAGOJ {
        match ik::ricevi_mesagon(1) {
            Ok(mesago) => {
                nombro += 1;
                idento = (mesago.sendinto, mesago.ricevonto);
            }
            Err(_) => break,
        }
    }
    let bone = nombro == ik::MAKSAJ_MESAGOJ + 1 && idento == (1, 2);
    crate::skribu!(
        "[fadeno 2] premrego ({}): {} mesaĝoj, idento sendinto={}, ricevonto={}\n",
        if bone { "bone" } else { "MALBONE" },
        nombro,
        idento.0,
        idento.1
    );
}

/// Fadeno 4: la tempolima politiko de la kerna vojo.
///
/// Ne-bloka ricevo sen konvena mesaĝo redonas `-11` tuj; tempolimo de 5 tikoj
/// (50 ms) redonas `-110` ĝuste ĉe la limdato; mesaĝo, kiu jam atendas aŭ
/// alvenas dum la atendo, venkas la tempolimon. La fonta filtro pruvas, ke
/// fremda mesaĝo ne kontentigas la vokon.
fn tempolima_provo(id: u64) {
    let t0 = crate::arkitekturo::temporizilaj_tikoj();
    let nebloka = ik::ricevi_mesagon_atenda(Some(3), ik::Atendo::Nebloka)
        .err()
        .map_or(0, ik::IkEraro::kodo);
    let nebloka_daŭro = crate::arkitekturo::temporizilaj_tikoj() - t0;

    let t1 = crate::arkitekturo::temporizilaj_tikoj();
    let tempolimo = ik::ricevi_mesagon_atenda(Some(3), ik::Atendo::Tempa(5))
        .err()
        .map_or(0, ik::IkEraro::kodo);
    let tempolima_daŭro = crate::arkitekturo::temporizilaj_tikoj() - t1;

    let (tempa_kodo, tempa_tipo) = match ik::ricevi_mesagon_atenda(None, ik::Atendo::Tempa(200)) {
        Ok(mesago) => (0, mesago.tipo),
        Err(eraro) => (eraro.kodo(), 0),
    };
    let (senfina_kodo, senfina_tipo) = match ik::ricevi_mesagon_atenda(None, ik::Atendo::Senfina) {
        Ok(mesago) => (0, mesago.tipo),
        Err(eraro) => (eraro.kodo(), 0),
    };

    let bone = nebloka == -11
        && nebloka_daŭro == 0
        && tempolimo == -110
        && (5..=5 + planisto::MAKSA_PRETA_ATENDO + 1).contains(&tempolima_daŭro)
        && tempa_kodo == 0
        && tempa_tipo == 90
        && senfina_kodo == 0
        && senfina_tipo == 91;
    crate::skribu!(
        "[fadeno {}] tempolimoj ({}): nebloka={} ({} tikoj), tempolimo={} ({} tikoj), mesaĝo={}/tipo {}, senfina={}/tipo {}\n",
        id,
        if bone { "bone" } else { "MALBONE" },
        nebloka,
        nebloka_daŭro,
        tempolimo,
        tempolima_daŭro,
        tempa_kodo,
        tempa_tipo,
        senfina_kodo,
        senfina_tipo
    );
}

fn teksto(mesago: &ik::Mesago) -> &str {
    core::str::from_utf8(mesago.enhavo()).unwrap_or("<nevalida>")
}
