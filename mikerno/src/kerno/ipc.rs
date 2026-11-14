//! Minimuma mesaĝa IPC: finpunktaj objektoj, kapabloj, teniloj.
//!
//! Ekde Fazo 2 de M14 la mesaĝvico apartenas al la finpunkto, ne al la fadeno.
//! Ĉiu fadeno havas defaŭltan finpunkton (kreitan ĉe ĝia registrado), kaj
//! ekde Fazo 3 la sistemvokaj vojoj `SENDI`/`RICEVI` uzas veran `Tenilo`-n el
//! la kapablotabelo de la procezo. La kernaj fadenoj (procezo `0`) ne havas
//! kapablotabelon; ili uzas la internajn `sendi_mesagon`/`ricevi_mesagon`
//! funkciojn, kiuj preterpasas la kapablokontrolon.
//!
//! Ekde Fazo 4 `REVOKI` forigas la tutan finpunktan objekton: ĉiuj teniloj en
//! ĉiuj procezaj tabeloj malvalidiĝas kaj blokita ricevanto vekiĝas per `-32`.
//! Ĉe la detruo de procezo la sama purigo forigas ĝiajn kapablojn kaj la
//! finpunktojn, kiujn ĝi posedas.
//!
//! Ekde Fazo 5 plena mesaĝvico premegas la sendanton: `SENDI` blokas ĝis la
//! ricevanto liberigas lokon (aŭ la finpunkto revokiĝas, kio redonas `-32`).
//! Nur la ripoza kunteksto (`0`) ne povas bloki kaj ricevas `-4`; la mesaĝoj
//! restas identecaj — la kerno ĉiam anstataŭigas `sendinto` per la nuna fadeno
//! kaj `ricevonto` per la finpunkta identigilo — kaj `IpcStatistiko` nombras
//! la liverojn, la premregojn kaj la revokojn.
//!
//! Ekde Fazo 6 `RICEVI_TEMPO`/`SENDI_TEMPO` aldonas la atendantan politikon:
//! `r10 = u32::MAX` blokas sen limdato, `0` estas nebloka (`-11`), kaj alia
//! valoro estas tempolimo en milisekundoj, kiun la planisto mezuras en
//! `ceil(ms/10)` tikoj (`-110`). `RICEVI_TEMPO.rdx` limigas la fonton al unu
//! fadeno. Revoko (`-32`) havas prioritaton antaŭ la tempolimo, kaj mesaĝo,
//! kiu alvenas samtempe, antaŭas ambaŭ.
//!
//! Ĉiuj operacioj tenas interrompojn malŝaltitaj.

use crate::interrompo::InterrompaGardo;
use crate::kunhavigita::Kunhavigita;
use libmikerno::Tenilo;

use super::objekto::FadenoId;
use super::planisto::{self, MAKSAJ_FADENOJ};
use super::procezo::{MAKSAJ_PROCEZOJ, ProcezaId};

/// La mesaĝa aranĝo, la permesoj kaj la kapabloj venas el `libmikerno`, do la
/// kerno kaj Ring 3 uzas la saman `#[repr(C)]`-strukturon sen konverto kaj sen
/// duobla difino.
pub use libmikerno::{MAKSA_MESAGA_LONGO, Mesago, Permesoj};

/// La kongruo kun la komuna ABI estas fiksita je kompila tempo.
const _: () = {
    assert!(libmikerno::ABI_VERSIO == 2);
    assert!(core::mem::size_of::<Mesago>() == 96);
    assert!(core::mem::align_of::<Mesago>() == 8);
    assert!(core::mem::offset_of!(Mesago, tipo) == 0);
    assert!(core::mem::offset_of!(Mesago, sendinto) == 8);
    assert!(core::mem::offset_of!(Mesago, ricevonto) == 16);
    assert!(core::mem::offset_of!(Mesago, longo) == 24);
};

/// La mesaĝvica kapacito; la testa premrega scenaro uzas `+ 1` mesaĝojn.
pub const MAKSAJ_MESAGOJ: usize = 16;
/// La finpunkta tabelo kaj la kapablotabelo havas po 16 fendojn; `Tenilo`
/// povas adresi nur `0..=15`.
pub const MAKSAJ_FINPUNKTOJ: usize = 16;
pub const MAKSAJ_KAPABLOJ: usize = 16;

/// La identigilo, kiun `Mesago.ricevonto` portas: la defaŭltaj finpunktoj
/// uzas la fadenan identigilon (`1..=8`), la eksplicitaj ekde `16`.
pub type FinaPunktaId = u64;

const UNUA_FINPUNKTA_ID: FinaPunktaId = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IpcEraro {
    NevalidaCelloko,
    MesagujoPlena,
    NevalidaTenilo,
    /// La kapablo validas, sed ĝi ne posedas la postulatan permeson.
    NesufiĉajPermesoj,
    TabeloPlena,
    /// La finpunkto estis revokita dum la ricevanto aŭ sendanto atendis; la
    /// objekto kaj ĉiuj teniloj malaperis (`FINPUNKTO_REVOKITA`).
    Revokita,
    /// La mesaĝa `longo` superas `MAKSA_MESAGA_LONGO` (`TRO_LONGA_MESAGO`).
    TroLongaMesago,
    /// La nebloka voko trovis malplenan vicon aŭ plenan mesaĝujon
    /// (`PROVU_DENOVE`).
    ProvuDenove,
    /// La tempolimo pasis antaŭ ol la operacio eblis (`TEMPOLIMO`).
    Tempolimo,
}

impl IpcEraro {
    /// La negativa erar-kodo de la sistemvoka ABI.
    pub const fn kodo(self) -> i64 {
        match self {
            Self::NesufiĉajPermesoj => -1,
            Self::NevalidaCelloko => -3,
            Self::MesagujoPlena => -4,
            Self::NevalidaTenilo => -9,
            Self::ProvuDenove => -11,
            Self::TabeloPlena => -28,
            Self::Revokita => -32,
            Self::TroLongaMesago => -90,
            Self::Tempolimo => -110,
        }
    }
}

const _: () = {
    assert!(IpcEraro::NesufiĉajPermesoj.kodo() == -1);
    assert!(IpcEraro::NevalidaCelloko.kodo() == -3);
    assert!(IpcEraro::MesagujoPlena.kodo() == -4);
    assert!(IpcEraro::NevalidaTenilo.kodo() == libmikerno::syscall::NEVALIDA_TENILO as i64);
    assert!(IpcEraro::ProvuDenove.kodo() == libmikerno::syscall::PROVU_DENOVE as i64);
    assert!(IpcEraro::TabeloPlena.kodo() == libmikerno::syscall::TABELO_PLENA as i64);
    assert!(IpcEraro::Revokita.kodo() == libmikerno::syscall::FINPUNKTO_REVOKITA as i64);
    assert!(IpcEraro::TroLongaMesago.kodo() == libmikerno::syscall::TRO_LONGA_MESAGO as i64);
    assert!(IpcEraro::Tempolimo.kodo() == libmikerno::syscall::TEMPOLIMO as i64);
};

/// La atenda politiko de IPC-voko (`r10` en la sistemvoka ABI, C15).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Atendo {
    /// `u32::MAX`: bloku sen limdato.
    Senfina,
    /// `0`: ne bloku; malplena vico aŭ plena mesaĝujo redonas `-11`.
    Nebloka,
    /// La nombro da temporizilaj tikoj ĝis la tempolimo (`ceil(ms/10) ≥ 1`).
    Tempa(u64),
}

/// Konvertu la tempoliman argumenton (`r10`) al atenda politiko. `None` por
/// valoro super `u32::MAX`; `u32::MAX` mem signifas senfinan atendo.
pub const fn el_tempolimo(valoro: u64) -> Option<Atendo> {
    if valoro == u32::MAX as u64 {
        Some(Atendo::Senfina)
    } else if valoro == 0 {
        Some(Atendo::Nebloka)
    } else if valoro > u32::MAX as u64 {
        None
    } else {
        Some(Atendo::Tempa((valoro as u32).div_ceil(10) as u64))
    }
}

/// La absoluta temporizila tiko, ĉe kiu la voko tempolimiĝos. Ĝi estas fiksata
/// je la eniro, do ripeta blokado (ekzemple post veko pro mesaĝo el alia fonto)
/// ne plilongigas la atendadon.
fn absoluta_limdato(atendo: Atendo) -> Option<u64> {
    match atendo {
        Atendo::Tempa(tikoj) => {
            Some(crate::arkitekturo::temporizilaj_tikoj().saturating_add(tikoj))
        }
        _ => None,
    }
}

#[derive(Clone, Copy)]
struct Ero {
    fonto: FadenoId,
    mesago: Mesago,
}

#[derive(Clone, Copy)]
struct Mesagujo {
    eroj: [Option<Ero>; MAKSAJ_MESAGOJ],
    nombro: usize,
}

impl Mesagujo {
    const fn nova() -> Self {
        Self {
            eroj: [None; MAKSAJ_MESAGOJ],
            nombro: 0,
        }
    }

    fn plena(&self) -> bool {
        self.nombro >= MAKSAJ_MESAGOJ
    }

    fn pusxi(&mut self, ero: Ero) -> Result<(), IpcEraro> {
        if self.plena() {
            return Err(IpcEraro::MesagujoPlena);
        }
        self.eroj[self.nombro] = Some(ero);
        self.nombro += 1;
        Ok(())
    }

    /// Prenu la unuan mesaĝon; `Some(fonto)` limigas al tiu sendinto, `None`
    /// akceptas ajnan fonton (la fonta filtro de `RICEVI_TEMPO`, C4).
    fn preni_de(&mut self, fonto: Option<FadenoId>) -> Option<Mesago> {
        let indekso = (0..self.nombro)
            .find(|&i| self.eroj[i].is_some_and(|e| fonto.is_none_or(|f| e.fonto == f)))?;
        let ero = self.eroj[indekso].take()?;
        for movo in indekso..self.nombro - 1 {
            self.eroj[movo] = self.eroj[movo + 1];
        }
        self.nombro -= 1;
        self.eroj[self.nombro] = None;
        Some(ero.mesago)
    }
}

/// La decido de unu sendprovo antaŭ ol tuŝi la vicon.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SendaDecido {
    Enmetu,
    /// La vico estas plena, sed la sendanto povas bloki (ĝi ne estas la
    /// ripoza kunteksto).
    Atendu,
    /// La vico estas plena kaj la sendanto ne povas bloki.
    Rifuzi,
}

/// La premrega politiko kiel pura funkcio, por ke Kani povu pruvi ĝin.
const fn senda_decido(enmetitaj: usize, maks: usize, povas_bloki: bool) -> SendaDecido {
    if enmetitaj < maks {
        SendaDecido::Enmetu
    } else if povas_bloki {
        SendaDecido::Atendu
    } else {
        SendaDecido::Rifuzi
    }
}

/// Nombrilo de la IPC-trafiko ekde la starto; `Copy` por ke la raporto ne
/// pruntedonu la tutmondan staton.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct IpcStatistiko {
    /// Mesaĝoj sukcese enmetitaj en vicon.
    pub senditaj: u64,
    /// Mesaĝoj sukcese prenitaj el vico.
    pub ricevitaj: u64,
    /// Sendprovoj, kiuj trovis plenan vicon kaj atendis liberan lokon.
    pub premregoj: u64,
    /// Finpunktoj revokitaj (inkluzive de proceza purigo).
    pub revokoj: u64,
    /// Atendoj, kiuj finiĝis per la tempolimo `-110`.
    pub tempolimoj: u64,
}

static STATISTIKO: Kunhavigita<IpcStatistiko> = Kunhavigita::nova(IpcStatistiko {
    senditaj: 0,
    ricevitaj: 0,
    premregoj: 0,
    revokoj: 0,
    tempolimoj: 0,
});

fn statistiko() -> &'static mut IpcStatistiko {
    STATISTIKO.akiri()
}

/// Kopio de la nuna statistiko; la vokanto tenas interrompojn malŝaltitajn aŭ
/// akceptas malstrikte kongruan legadon.
pub fn nuna_statistiko() -> IpcStatistiko {
    *STATISTIKO.akiri()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FinaPunktaStato {
    Libera,
    Aktiva,
    /// La generacio eluziĝis; la fendo neniam plu estos atribuata (C12).
    Emerita,
}

#[derive(Clone, Copy)]
struct FinaPunktoObjekto {
    stato: FinaPunktaStato,
    id: FinaPunktaId,
    generacio: u32,
    posedanto: ProcezaId,
    /// La fadeno, kies defaŭlta finpunkto ĉi tio estas; `0` por eksplicitaj.
    fadeno: FadenoId,
    /// La fadeno blokita en `RICEVI` sur ĉi tiu finpunkto; `0` se neniu.
    ricevanto: FadenoId,
    /// Bitmasko de la fadenoj blokitaj en `SENDI` pro plena vico; bito `id − 1`.
    sendantoj: u16,
    referencoj: u16,
    vico: Mesagujo,
}

impl FinaPunktoObjekto {
    const fn nova() -> Self {
        Self {
            stato: FinaPunktaStato::Libera,
            id: 0,
            generacio: 1,
            posedanto: 0,
            fadeno: 0,
            ricevanto: 0,
            sendantoj: 0,
            referencoj: 0,
            vico: Mesagujo::nova(),
        }
    }

    fn aktiva(&self) -> bool {
        self.stato == FinaPunktaStato::Aktiva
    }

    /// Forigu la mesaĝojn kaj redonu la fendon; la generacio altiĝas, do ĉiuj
    /// malnovaj teniloj restas malvalidaj (kontraŭ-ABA).
    fn liberigu(&mut self) {
        self.vico = Mesagujo::nova();
        self.referencoj = 0;
        self.id = 0;
        self.posedanto = 0;
        self.fadeno = 0;
        self.ricevanto = 0;
        self.sendantoj = 0;
        match pliigita_generacio(self.generacio) {
            Some(sekva) => {
                self.generacio = sekva;
                self.stato = FinaPunktaStato::Libera;
            }
            None => {
                self.generacio = Tenilo::MAKSA_GENERACIO;
                self.stato = FinaPunktaStato::Emerita;
            }
        }
    }
}

/// La sekva generacio de fendo, aŭ `None` se ĝi eluziĝis. Generacio `0` estas
/// ĉiam malvalida kaj ne havas sekvan valoron.
const fn pliigita_generacio(generacio: u32) -> Option<u32> {
    if generacio == 0 || generacio >= Tenilo::MAKSA_GENERACIO {
        None
    } else {
        Some(generacio + 1)
    }
}

/// Malaltigu referenckalkulon sen suba transfluo.
const fn forprenu_referencon(referencoj: u16) -> u16 {
    if referencoj == 0 { 0 } else { referencoj - 1 }
}

/// Ĉu la kapabla fendo ankoraŭ kongruas kun viva finpunkta objekto? Revoko
/// altigas la generacion de la objekto, do la malnovaj fendoj ne plu akordas.
const fn akordas(aktiva: bool, objekta_generacio: u32, kapabla_generacio: u32) -> bool {
    aktiva && objekta_generacio == kapabla_generacio
}

/// Ĉu `petitaj` estas subaro de `fontaj`? Transdono nur malpliiĝas.
const fn dono_permesita(fontaj: Permesoj, petitaj: Permesoj) -> bool {
    petitaj.bitoj() & !fontaj.bitoj() == 0
}

/// La unua libera finpunkta fendo, aŭ `None` se ĉiuj estas okupataj aŭ
/// emeritaj. La serĉo estas pura por ke Kani povas pruvi ĝin.
fn elekti_liberan(statoj: &[FinaPunktaStato; MAKSAJ_FINPUNKTOJ]) -> Option<u8> {
    let mut indekso = 0;
    while indekso < MAKSAJ_FINPUNKTOJ {
        if statoj[indekso] == FinaPunktaStato::Libera {
            return Some(indekso as u8);
        }
        indekso += 1;
    }
    None
}

static FINAJ_PUNKTOJ: Kunhavigita<[FinaPunktoObjekto; MAKSAJ_FINPUNKTOJ]> =
    Kunhavigita::nova([FinaPunktoObjekto::nova(); MAKSAJ_FINPUNKTOJ]);

static SEKVA_FINPUNKTA_ID: Kunhavigita<FinaPunktaId> = Kunhavigita::nova(UNUA_FINPUNKTA_ID);

fn finpunktoj() -> &'static mut [FinaPunktoObjekto; MAKSAJ_FINPUNKTOJ] {
    FINAJ_PUNKTOJ.akiri()
}

fn sekva_finpunkta_id() -> FinaPunktaId {
    let krado = SEKVA_FINPUNKTA_ID.akiri();
    let id = *krado;
    *krado += 1;
    id
}

/// La fadenoj vekitaj de revoko, kiuj ankoraŭ ne ricevis la rezulton: la
/// riceva buklo konsumas la markon kaj raportas `-32` anstataŭ provi denove
/// la malaperintan kapablon.
static REVOKAJ_VEKOJ: Kunhavigita<[bool; MAKSAJ_FADENOJ + 1]> =
    Kunhavigita::nova([false; MAKSAJ_FADENOJ + 1]);

fn marki_revokan_vekon(fadeno: FadenoId) {
    if fadeno == 0 {
        return;
    }
    if let Some(marko) = REVOKAJ_VEKOJ.akiri().get_mut(fadeno as usize) {
        *marko = true;
    }
}

/// Konsumu la revokan markon de fadeno; `true` nur unufoje por ĉiu revoko.
fn preni_revokan_vekon(fadeno: FadenoId) -> bool {
    REVOKAJ_VEKOJ
        .akiri()
        .get_mut(fadeno as usize)
        .is_some_and(core::mem::take)
}

/// La bito de fadeno en `FinaPunktoObjekto::sendantoj`: la identigiloj estas
/// `1..=8`, do la plej malalta bito apartenas al fadeno `1`.
const fn senda_bito(fadeno: FadenoId) -> u16 {
    1 << (fadeno - 1)
}

/// Veku la sendantojn, kiuj atendis lokon en plena vico; la voko okazas post
/// `preni_de`, do almenaŭ unu loko estas libera kaj la vekitoj rajtas reprovi.
fn veki_sendantojn(fendo: u8) {
    let objekto = &mut finpunktoj()[fendo as usize];
    let mut bitoj = objekto.sendantoj;
    objekto.sendantoj = 0;
    let mut id = 1;
    while bitoj != 0 {
        if bitoj & 1 != 0 {
            planisto::veki(id);
        }
        bitoj >>= 1;
        id += 1;
    }
}

/// Veku la blokitajn sendantojn per la revoka marko (`-32`), ĉar la finpunkto
/// malaperas kaj ilia atendo neniam plenumiĝos.
fn revoku_sendantojn(sendantoj: u16) {
    let mut bitoj = sendantoj;
    let mut id = 1;
    while bitoj != 0 {
        if bitoj & 1 != 0 {
            marki_revokan_vekon(id);
            planisto::veki(id);
        }
        bitoj >>= 1;
        id += 1;
    }
}

/// Forigu la fadenon el la premrega bitmasko post tempolimo; la finpunkto
/// restas aktiva, do ĝi ne plu atendu la vekon de `veki_sendantojn`.
fn forigu_sendanton(fendo: u8, fadeno: FadenoId) {
    if fadeno == 0 {
        return;
    }
    let objekto = &mut finpunktoj()[fendo as usize];
    if objekto.aktiva() {
        objekto.sendantoj &= !senda_bito(fadeno);
    }
}

/// Malplenigu la ricevantan fadenon post tempolimo, se ĝi ankoraŭ estas tiu.
fn forigu_ricevanton(fendo: u8, fadeno: FadenoId) {
    let objekto = &mut finpunktoj()[fendo as usize];
    if objekto.aktiva() && objekto.ricevanto == fadeno {
        objekto.ricevanto = 0;
    }
}

fn nombri_tempolimon() {
    let stat = statistiko();
    stat.tempolimoj = stat.tempolimoj.saturating_add(1);
}

/// Kapabla fendo de unu procezo: kiu finpunkto kaj kun kiuj permesoj.
///
/// La tenilo portas la generacion de la fendo, kaj `fp_generacio` estas la
/// dua, sendependa generacio de la finpunkta objekto. Ambaŭ devas kongrui, do
/// malnova tenilo ne reviviĝas eĉ se la fenda indekso reuziĝas.
#[derive(Clone, Copy)]
struct KapabloEniro {
    tenilo: Tenilo,
    permesoj: Permesoj,
    finpunkto: u8,
    fp_generacio: u32,
    uzata: bool,
}

impl KapabloEniro {
    const fn libera() -> Self {
        Self {
            tenilo: Tenilo::MALVALIDA,
            permesoj: Permesoj::NENIU,
            finpunkto: 0,
            fp_generacio: 0,
            uzata: false,
        }
    }
}

#[derive(Clone, Copy)]
struct Kapablotabelo {
    eroj: [KapabloEniro; MAKSAJ_KAPABLOJ],
    /// La sekva generacio de ĉiu fendo; `0` signifas emeritan fendon.
    generacioj: [u32; MAKSAJ_KAPABLOJ],
}

impl Kapablotabelo {
    const fn nova() -> Self {
        Self {
            eroj: [KapabloEniro::libera(); MAKSAJ_KAPABLOJ],
            generacioj: [1; MAKSAJ_KAPABLOJ],
        }
    }

    fn unua_libera(&self) -> Option<usize> {
        (0..MAKSAJ_KAPABLOJ).find(|&fendo| !self.eroj[fendo].uzata && self.generacioj[fendo] != 0)
    }

    /// Instalu kapablon en la donitan fendon; `None` se ĝi estas okupata aŭ
    /// emerita.
    fn instalu_en(
        &mut self,
        fendo: usize,
        finpunkto: u8,
        fp_generacio: u32,
        permesoj: Permesoj,
    ) -> Option<Tenilo> {
        if fendo >= MAKSAJ_KAPABLOJ || self.eroj[fendo].uzata {
            return None;
        }
        let tenilo = Tenilo::nova(fendo as u8, self.generacioj[fendo])?;
        self.eroj[fendo] = KapabloEniro {
            tenilo,
            permesoj,
            finpunkto,
            fp_generacio,
            uzata: true,
        };
        Some(tenilo)
    }

    /// Malplenigu fendon kaj antaŭenigu ĝian generacion; eluzita generacio
    /// emeritas la fendon (`0`).
    fn liberigu_fendon(&mut self, fendo: usize) {
        self.eroj[fendo] = KapabloEniro::libera();
        self.generacioj[fendo] = pliigita_generacio(self.generacioj[fendo]).unwrap_or_default();
    }
}

/// La unua nivelo de la kapabla validigo: la fendo kongruas kun la tenilo kaj
/// posedas la postulatajn permesojn.
fn serĉi_kapablon(
    tabelo: &Kapablotabelo,
    tenilo: Tenilo,
    bezonataj: Permesoj,
) -> Result<usize, IpcEraro> {
    let indekso = tenilo.indekso() as usize;
    let eniro = tabelo.eroj.get(indekso).ok_or(IpcEraro::NevalidaTenilo)?;
    if !eniro.uzata || eniro.tenilo != tenilo {
        return Err(IpcEraro::NevalidaTenilo);
    }
    if !eniro.permesoj.posedas(bezonataj) {
        return Err(IpcEraro::NesufiĉajPermesoj);
    }
    Ok(indekso)
}

/// La dua nivelo: la kapabla fendo devas montri al viva finpunkta objekto kun
/// la generacio, kiun la fendo memoras.
fn akiri_kapablon(
    procezo: ProcezaId,
    tenilo: Tenilo,
    bezonataj: Permesoj,
) -> Result<(u8, KapabloEniro), IpcEraro> {
    let tabelo = kapabloj(procezo).ok_or(IpcEraro::NevalidaTenilo)?;
    let indekso = serĉi_kapablon(tabelo, tenilo, bezonataj)?;
    let eniro = tabelo.eroj[indekso];
    let objekto = &finpunktoj()[eniro.finpunkto as usize];
    if !akordas(objekto.aktiva(), objekto.generacio, eniro.fp_generacio) {
        return Err(IpcEraro::NevalidaTenilo);
    }
    Ok((eniro.finpunkto, eniro))
}

/// La kapablotabeloj de la procezoj `0..=7`; linio `0` (la kerno) restas ĉiam
/// malplena, do `kapabloj(0)` redonas `None` (C8).
static KAPABLOJ: Kunhavigita<[Kapablotabelo; MAKSAJ_PROCEZOJ]> =
    Kunhavigita::nova([Kapablotabelo::nova(); MAKSAJ_PROCEZOJ]);

fn kapabloj(procezo: ProcezaId) -> Option<&'static mut Kapablotabelo> {
    if procezo == 0 || procezo as usize >= MAKSAJ_PROCEZOJ {
        return None;
    }
    KAPABLOJ.akiri().get_mut(procezo as usize)
}

fn libera_finpunkta_fendo() -> Option<u8> {
    let mut statoj = [FinaPunktaStato::Emerita; MAKSAJ_FINPUNKTOJ];
    for (indekso, objekto) in finpunktoj().iter().enumerate() {
        statoj[indekso] = objekto.stato;
    }
    elekti_liberan(&statoj)
}

/// La indekso de la defaŭlta finpunkto de la fadeno, se ĝi jam ekzistas.
fn defaŭlta_indekso(fadeno: FadenoId) -> Option<usize> {
    finpunktoj()
        .iter()
        .position(|objekto| objekto.aktiva() && objekto.fadeno == fadeno)
}

/// Forigu finpunktan objekton tute: liberigu ĝian fendon, forprenu ĉiujn
/// kongruajn kapablojn el ĉiuj procezaj tabeloj kaj veku la blokitan
/// ricevanton per la revoka marko (`-32`). `false` se la fendo jam ne estis
/// aktiva.
fn forigu_finpunkton(fendo: u8) -> bool {
    let objekto = &finpunktoj()[fendo as usize];
    if !objekto.aktiva() {
        return false;
    }
    let generacio = objekto.generacio;
    let sendantoj = objekto.sendantoj;
    let celo = if objekto.ricevanto != 0 {
        objekto.ricevanto
    } else {
        objekto.fadeno
    };

    let mut procezo = 1;
    while procezo < MAKSAJ_PROCEZOJ {
        if let Some(tabelo) = kapabloj(procezo as ProcezaId) {
            let mut fendo_kapabla = 0;
            while fendo_kapabla < MAKSAJ_KAPABLOJ {
                let eniro = tabelo.eroj[fendo_kapabla];
                if eniro.uzata && eniro.finpunkto == fendo && eniro.fp_generacio == generacio {
                    tabelo.liberigu_fendon(fendo_kapabla);
                }
                fendo_kapabla += 1;
            }
        }
        procezo += 1;
    }

    finpunktoj()[fendo as usize].liberigu();
    revoku_sendantojn(sendantoj);
    if celo != 0 {
        marki_revokan_vekon(celo);
        planisto::veki(celo);
    }
    let stat = statistiko();
    stat.revokoj = stat.revokoj.saturating_add(1);
    true
}

/// Rezervu fendon por nova finpunkto; la alvokanto jam tenas interrompojn
/// malŝaltitaj.
fn rezervi_finpunkton(
    procezo: ProcezaId,
    fadeno: FadenoId,
    id: FinaPunktaId,
) -> Result<u8, IpcEraro> {
    let fendo = libera_finpunkta_fendo().ok_or(IpcEraro::TabeloPlena)?;
    let objekto = &mut finpunktoj()[fendo as usize];
    if Tenilo::nova(0, objekto.generacio).is_none() {
        return Err(IpcEraro::TabeloPlena);
    }
    objekto.stato = FinaPunktaStato::Aktiva;
    objekto.id = id;
    objekto.posedanto = procezo;
    objekto.fadeno = fadeno;
    objekto.referencoj = 1;
    Ok(fendo)
}

/// Registru la defaŭltan finpunkton de fadeno. Procezaj fadenoj ankaŭ ricevas
/// kapablan fendon `0` kun `PLENAJ`, do la malnova vojo restas uzebla ĝis la
/// tenila transiro en Fazo 3. `procezo == 0` por kernaj fadenoj (C8).
pub fn registri_fadenon(fadeno: FadenoId, procezo: ProcezaId) -> Result<(), IpcEraro> {
    let _gardo = InterrompaGardo::nova();
    if fadeno == 0 || fadeno as usize > MAKSAJ_FADENOJ {
        return Err(IpcEraro::NevalidaCelloko);
    }
    if defaŭlta_indekso(fadeno).is_some() {
        return Ok(());
    }
    if procezo != 0 && kapabloj(procezo).is_none() {
        return Err(IpcEraro::NevalidaTenilo);
    }
    let fendo = rezervi_finpunkton(procezo, fadeno, fadeno)?;
    if procezo == 0 {
        return Ok(());
    }
    let fp_generacio = finpunktoj()[fendo as usize].generacio;
    let Some(tabelo) = kapabloj(procezo) else {
        finpunktoj()[fendo as usize].liberigu();
        return Err(IpcEraro::TabeloPlena);
    };
    if tabelo
        .instalu_en(0, fendo, fp_generacio, Permesoj::PLENAJ)
        .is_none()
    {
        finpunktoj()[fendo as usize].liberigu();
        return Err(IpcEraro::TabeloPlena);
    }
    finpunktoj()[fendo as usize].referencoj += 1;
    Ok(())
}

/// Puŝu mesaĝon en la vicon de finpunkta fendo, fiksu la identecon kaj veku la
/// ricevanton. La alvokanto tenas interrompojn malŝaltitaj.
fn enmeti(fendo: u8, mesago: Mesago) -> Result<(), IpcEraro> {
    let nuna = planisto::nuna_id();
    let objekto = &mut finpunktoj()[fendo as usize];
    let mut mesago = mesago;
    mesago.sendinto = nuna;
    mesago.ricevonto = objekto.id;
    objekto.vico.pusxi(Ero {
        fonto: nuna,
        mesago,
    })?;
    let stat = statistiko();
    stat.senditaj = stat.senditaj.saturating_add(1);
    let celo = if objekto.ricevanto != 0 {
        objekto.ricevanto
    } else {
        objekto.fadeno
    };
    objekto.ricevanto = 0;
    if celo != 0 {
        planisto::veki(celo);
    }
    Ok(())
}

/// Sendu mesaĝon al la defaŭlta finpunkto de fadeno; la fonto estas la nuna
/// fadeno. Ĉi tiu estas la kerna vojo sen kapablokontrolo (procezo `0`).
///
/// La mesaĝo estas kopiata, kaj `sendinto`/`ricevonto` estas fiksitaj de la
/// kerno: la uzanto ne povas mensogi pri ili. `ricevonto` estas la finpunkta
/// identigilo, ne la celo-fadeno (C1).
///
/// # Eraroj
/// `NevalidaCelloko` se la celo ne estas viva fadeno (C16); `MesagujoPlena`
/// se la vico estas plena kaj la vokanto ne povas bloki (la ripoza
/// kunteksto); `Revokita` se la finpunkto malaperas dum la atendo;
/// `TabeloPlena` se la defaŭlta finpunkto mankas.
pub fn sendi_mesagon(destinacio: FadenoId, mesago: &Mesago) -> Result<(), IpcEraro> {
    let nuna = planisto::nuna_id();
    loop {
        let _gardo = InterrompaGardo::nova();
        if !planisto::vivas(destinacio) {
            return Err(IpcEraro::NevalidaCelloko);
        }
        let indekso = defaŭlta_indekso(destinacio).ok_or(IpcEraro::TabeloPlena)?;
        match senda_decido(finpunktoj()[indekso].vico.nombro, MAKSAJ_MESAGOJ, nuna != 0) {
            SendaDecido::Enmetu => return enmeti(indekso as u8, *mesago),
            SendaDecido::Rifuzi => return Err(IpcEraro::MesagujoPlena),
            SendaDecido::Atendu => {
                finpunktoj()[indekso].sendantoj |= senda_bito(nuna);
                let stat = statistiko();
                stat.premregoj = stat.premregoj.saturating_add(1);
                planisto::bloki();
                if preni_revokan_vekon(nuna) {
                    return Err(IpcEraro::Revokita);
                }
            }
        }
    }
}

/// Sendu mesaĝon tra la kapablo `tenilo` de la procezo. Postulas `SENDI`.
///
/// Se la finpunkto estas defaŭlta (apartenas al fadeno), la celo devas ankoraŭ
/// vivi; alie `NevalidaCelloko`. Plena vico atendigas la sendanton laŭ
/// `atendo`: senfine, nebloke (`-11`) aŭ kun tempolimo (`-110`); revoko de la
/// finpunkto vekas ĝin per `-32`.
pub fn sendi_kapable(
    procezo: ProcezaId,
    tenilo: Tenilo,
    mesago: &Mesago,
    atendo: Atendo,
) -> Result<(), IpcEraro> {
    let nuna = planisto::nuna_id();
    let limdato = absoluta_limdato(atendo);
    let mut tempolimita = false;
    loop {
        let _gardo = InterrompaGardo::nova();
        let (fendo, _) = akiri_kapablon(procezo, tenilo, Permesoj::SENDI)?;
        {
            let objekto = &finpunktoj()[fendo as usize];
            if objekto.fadeno != 0 && !planisto::vivas(objekto.fadeno) {
                return Err(IpcEraro::NevalidaCelloko);
            }
        }
        match senda_decido(
            finpunktoj()[fendo as usize].vico.nombro,
            MAKSAJ_MESAGOJ,
            nuna != 0,
        ) {
            SendaDecido::Enmetu => return enmeti(fendo, *mesago),
            SendaDecido::Rifuzi => return Err(IpcEraro::MesagujoPlena),
            SendaDecido::Atendu => {
                if tempolimita {
                    forigu_sendanton(fendo, nuna);
                    nombri_tempolimon();
                    return Err(IpcEraro::Tempolimo);
                }
                if atendo == Atendo::Nebloka {
                    return Err(IpcEraro::ProvuDenove);
                }
                finpunktoj()[fendo as usize].sendantoj |= senda_bito(nuna);
                let stat = statistiko();
                stat.premregoj = stat.premregoj.saturating_add(1);
                match limdato {
                    Some(limdato) => planisto::bloki_ĝis_absoluta(limdato),
                    None => planisto::bloki(),
                }
                if planisto::preni_tempolimon() {
                    tempolimita = true;
                }
                if preni_revokan_vekon(nuna) {
                    return Err(IpcEraro::Revokita);
                }
            }
        }
    }
}

/// Ricevu mesaĝon el la finpunkto de la kapablo `tenilo`; `fonto` limigas la
/// sendinton (`None` akceptas ajnan). Postulas `RICEVI`; la permeso estas
/// kontrolata ankaŭ post ĉiu veko, do revoko ne lasos la fadenon en la vico.
/// Revoko de la atendata finpunkto vekas la ricevanton per `-32`; tempolimo
/// redonas `-110`, kaj mesaĝo, kiu alvenas samtempe, havas prioritaton.
pub fn ricevi_kapable(
    procezo: ProcezaId,
    tenilo: Tenilo,
    fonto: Option<FadenoId>,
    atendo: Atendo,
) -> Result<Mesago, IpcEraro> {
    let limdato = absoluta_limdato(atendo);
    let mut tempolimita = false;
    loop {
        let _gardo = InterrompaGardo::nova();
        let (fendo, _) = akiri_kapablon(procezo, tenilo, Permesoj::RICEVI)?;
        let nuna = planisto::nuna_id();
        if let Some(mesago) = finpunktoj()[fendo as usize].vico.preni_de(fonto) {
            finpunktoj()[fendo as usize].ricevanto = 0;
            let stat = statistiko();
            stat.ricevitaj = stat.ricevitaj.saturating_add(1);
            veki_sendantojn(fendo);
            return Ok(mesago);
        }
        if tempolimita {
            forigu_ricevanton(fendo, nuna);
            nombri_tempolimon();
            return Err(IpcEraro::Tempolimo);
        }
        if atendo == Atendo::Nebloka {
            return Err(IpcEraro::ProvuDenove);
        }
        if nuna == 0 {
            // La ripoza kunteksto ne havas fadenan vivciklon; ne bloku ĝin.
            return Err(IpcEraro::NevalidaCelloko);
        }
        finpunktoj()[fendo as usize].ricevanto = nuna;
        match limdato {
            Some(limdato) => planisto::bloki_ĝis_absoluta(limdato),
            None => planisto::bloki(),
        }
        if planisto::preni_tempolimon() {
            tempolimita = true;
        }
        if preni_revokan_vekon(nuna) {
            forigu_ricevanton(fendo, nuna);
            return Err(IpcEraro::Revokita);
        }
    }
}

/// Ricevu mesaĝon de la donita fonto el la defaŭlta finpunkto de la nuna
/// fadeno; bloku se mankas. Ĉi tiu estas la kerna vojo sen kapablokontrolo.
///
/// Revoko de la defaŭlta finpunkto vekas la fadenon per `-32`, do la voko
/// redonas `Err` anstataŭ pendi porĉiame.
pub fn ricevi_mesagon(fonto: FadenoId) -> Result<Mesago, IpcEraro> {
    ricevi_mesagon_atenda(Some(fonto), Atendo::Senfina)
}

/// Kerna vojo kun eksplicita fonta filtro kaj atenda politiko; ĝi ne havas
/// kapablokontrolon, ĉar la kernaj fadenoj ne posedas kapablotabelon (C8).
/// La tempolimo uzas la saman planistan vekon kiel `RICEVI_TEMPO`.
pub fn ricevi_mesagon_atenda(fonto: Option<FadenoId>, atendo: Atendo) -> Result<Mesago, IpcEraro> {
    let limdato = absoluta_limdato(atendo);
    let mut tempolimita = false;
    loop {
        let _gardo = InterrompaGardo::nova();
        let nuna = planisto::nuna_id();
        if nuna == 0 {
            // La ripoza kunteksto ne havas fadenan vivciklon; ne bloku ĝin.
            return Err(IpcEraro::NevalidaCelloko);
        }
        if preni_revokan_vekon(nuna) {
            return Err(IpcEraro::Revokita);
        }
        let Some(indekso) = defaŭlta_indekso(nuna) else {
            return Err(IpcEraro::TabeloPlena);
        };
        if let Some(mesago) = finpunktoj()[indekso].vico.preni_de(fonto) {
            finpunktoj()[indekso].ricevanto = 0;
            let stat = statistiko();
            stat.ricevitaj = stat.ricevitaj.saturating_add(1);
            veki_sendantojn(indekso as u8);
            return Ok(mesago);
        }
        if tempolimita {
            forigu_ricevanton(indekso as u8, nuna);
            nombri_tempolimon();
            return Err(IpcEraro::Tempolimo);
        }
        if atendo == Atendo::Nebloka {
            return Err(IpcEraro::ProvuDenove);
        }
        finpunktoj()[indekso].ricevanto = nuna;
        match limdato {
            Some(limdato) => planisto::bloki_ĝis_absoluta(limdato),
            None => planisto::bloki(),
        }
        if planisto::preni_tempolimon() {
            tempolimita = true;
        }
    }
}

/// Kreu eksplicitan finpunkton por la procezo kaj redonu veran tenilon.
///
/// La alvokanto ricevas la petatajn permesojn. `TabeloPlena` se la kapabla aŭ
/// la finpunkta tabelo ne plu havas liberan fendon; la kerno mem (`procezo 0`)
/// ne havas kapablotabelon (C8).
pub fn krei_finpunkton(procezo: ProcezaId, permesoj: Permesoj) -> Result<Tenilo, IpcEraro> {
    let _gardo = InterrompaGardo::nova();
    let tabelo = kapabloj(procezo).ok_or(IpcEraro::TabeloPlena)?;
    let fendo_kapabla = tabelo.unua_libera().ok_or(IpcEraro::TabeloPlena)?;
    let fendo_finpunkta = rezervi_finpunkton(procezo, 0, sekva_finpunkta_id())?;
    let fp_generacio = finpunktoj()[fendo_finpunkta as usize].generacio;
    let Some(tenilo) = tabelo.instalu_en(fendo_kapabla, fendo_finpunkta, fp_generacio, permesoj)
    else {
        finpunktoj()[fendo_finpunkta as usize].liberigu();
        return Err(IpcEraro::TabeloPlena);
    };
    Ok(tenilo)
}

/// Donu la kapablon `tenilo` al la procezo `celo` kun la permesoj `permesoj`.
///
/// La petitaj permesoj devas esti subaro de la fontaj (transdono nur
/// malpliiĝas); la fonta kapablo restas valida. La celo ricevas novan tenilon
/// en sia propra tabelo, kun la generacio de tiu fendo.
pub fn doni(
    procezo: ProcezaId,
    tenilo: Tenilo,
    celo: ProcezaId,
    permesoj: Permesoj,
) -> Result<Tenilo, IpcEraro> {
    let _gardo = InterrompaGardo::nova();
    if permesoj.bitoj() == 0 {
        return Err(IpcEraro::NesufiĉajPermesoj);
    }
    if celo == 0 || celo as usize >= MAKSAJ_PROCEZOJ {
        return Err(IpcEraro::NevalidaCelloko);
    }
    let (fendo, eniro) = akiri_kapablon(procezo, tenilo, Permesoj::NENIU)?;
    if !dono_permesita(eniro.permesoj, permesoj) {
        return Err(IpcEraro::NesufiĉajPermesoj);
    }
    let tabelo = kapabloj(celo).ok_or(IpcEraro::NevalidaCelloko)?;
    let fendo_kapabla = tabelo.unua_libera().ok_or(IpcEraro::TabeloPlena)?;
    let Some(nova) = tabelo.instalu_en(fendo_kapabla, fendo, eniro.fp_generacio, permesoj) else {
        return Err(IpcEraro::TabeloPlena);
    };
    let objekto = &mut finpunktoj()[fendo as usize];
    objekto.referencoj = objekto.referencoj.saturating_add(1);
    Ok(nova)
}

/// Fermu unu lokan kapablofendon. La finpunkto restas viva se aliaj referencoj
/// ekzistas; ĝi nur malaperas kiam la lasta tenilo fermiĝas (`FERMI` neniam
/// revokas, C14).
pub fn fermi(procezo: ProcezaId, tenilo: Tenilo) -> Result<(), IpcEraro> {
    let _gardo = InterrompaGardo::nova();
    let tabelo = kapabloj(procezo).ok_or(IpcEraro::NevalidaTenilo)?;
    let indekso = serĉi_kapablon(tabelo, tenilo, Permesoj::NENIU)?;
    let eniro = tabelo.eroj[indekso];
    tabelo.liberigu_fendon(indekso);
    let objekto = &mut finpunktoj()[eniro.finpunkto as usize];
    if !akordas(objekto.aktiva(), objekto.generacio, eniro.fp_generacio) {
        return Err(IpcEraro::NevalidaTenilo);
    }
    objekto.referencoj = forprenu_referencon(objekto.referencoj);
    if objekto.referencoj == 0 {
        // La finpunkto malaperas: blokitaj sendantoj ne plu atendu lokon.
        revoku_sendantojn(objekto.sendantoj);
        objekto.liberigu();
    }
    Ok(())
}

/// Revoku la tutan finpunkton de la kapablo `tenilo` (C14): la objekto
/// malaperas, ĉiuj teniloj en ĉiuj procezaj tabeloj malvalidiĝas kaj blokita
/// ricevanto vekiĝas per `-32`. Postulas `REVOKI` en la kapablo.
pub fn revoki(procezo: ProcezaId, tenilo: Tenilo) -> Result<(), IpcEraro> {
    let _gardo = InterrompaGardo::nova();
    let (fendo, _) = akiri_kapablon(procezo, tenilo, Permesoj::REVOKI)?;
    if forigu_finpunkton(fendo) {
        Ok(())
    } else {
        Err(IpcEraro::NevalidaTenilo)
    }
}

/// Kerna vojo: revoku la defaŭltan finpunkton de fadeno sen kapabla kontrolo.
/// La kernaj fadenoj ne havas kapablotabelon (C8); la Ring 0-memtesto uzas
/// ĉi tiun funkcion por pruvi la revokan vekon.
pub fn revoki_defaŭltan(fadeno: FadenoId) -> Result<(), IpcEraro> {
    let _gardo = InterrompaGardo::nova();
    let indekso = defaŭlta_indekso(fadeno).ok_or(IpcEraro::NevalidaCelloko)?;
    if forigu_finpunkton(indekso as u8) {
        Ok(())
    } else {
        Err(IpcEraro::NevalidaTenilo)
    }
}

/// Restarigu la kapablotabelon de ĵus kreita procezo.
///
/// La tabeloj estas statikaj laŭ proceza identigilo, do sen restarigo nova
/// procezo heredus la generaciojn de antaŭa procezo kun la sama identigilo
/// (aŭ de la Ring 0-memtestoj). Tiam la fenda `0` ne havus la unuan generacion
/// kaj la fiksa `BOOTSTRAP_TENILO` (`0x100`) ne plu validus.
pub fn inicialigi_procezon(procezo: ProcezaId) {
    if let Some(tabelo) = kapabloj(procezo) {
        *tabelo = Kapablotabelo::nova();
    }
}

/// Purigu la IPC-staton de mortanta procezo: liberigu ĉiujn ĝiajn
/// kapablofendojn, malaltigu la referenckalkulojn de fremdaj finpunktoj kaj
/// forigu la finpunktojn, kiujn ĝi posedas (inkluzive de la defaŭlta).
/// Forigo vekas blokitan ricevanton per `-32`.
///
/// Vokenda antaŭ ol la procezo malaperas el la planisto kaj sen tenata
/// `&mut Planisto`, ĉar la revokaj vekoj uzas la pretan vicon.
pub fn purigi_procezon(procezo: ProcezaId) {
    if procezo == 0 || procezo as usize >= MAKSAJ_PROCEZOJ {
        return;
    }
    let _gardo = InterrompaGardo::nova();
    let mut liberigitaj = 0;
    if let Some(tabelo) = kapabloj(procezo) {
        for fendo in 0..MAKSAJ_KAPABLOJ {
            if !tabelo.eroj[fendo].uzata {
                continue;
            }
            let eniro = tabelo.eroj[fendo];
            tabelo.liberigu_fendon(fendo);
            liberigitaj += 1;
            let objekto = &mut finpunktoj()[eniro.finpunkto as usize];
            if akordas(objekto.aktiva(), objekto.generacio, eniro.fp_generacio) {
                objekto.referencoj = forprenu_referencon(objekto.referencoj);
            }
        }
    }
    let mut revokitaj = 0;
    for fendo in 0..MAKSAJ_FINPUNKTOJ {
        let objekto = &finpunktoj()[fendo];
        let posedata = objekto.aktiva() && objekto.posedanto == procezo;
        if posedata && forigu_finpunkton(fendo as u8) {
            revokitaj += 1;
        }
    }
    if liberigitaj != 0 || revokitaj != 0 {
        crate::skribu!(
            "[ipc] purigo de procezo {}: {} kapabloj liberigitaj, {} finpunktoj revokitaj\n",
            procezo,
            liberigitaj,
            revokitaj
        );
    }
}

/// Ring 0-memtesto de la finpunktaj tabeloj, rulata antaŭ la unua `lanĉi()`.
///
/// La testo uzas la kapablolinion de la rezervita proceza identigilo `6`; la
/// kerna linio `0` devas resti malplena. Ĉiuj kreitaj finpunktoj fermiĝas, do
/// la stato revenas al la komenca (krom la generacioj).
pub fn testi_tabelojn() {
    let procezo: ProcezaId = 6;
    let mut teniloj = [Tenilo::MALVALIDA; MAKSAJ_KAPABLOJ];
    let mut nombro = 0;
    while nombro < teniloj.len() {
        match krei_finpunkton(procezo, Permesoj::PLENAJ) {
            Ok(tenilo) => {
                teniloj[nombro] = tenilo;
                nombro += 1;
            }
            Err(IpcEraro::TabeloPlena) => break,
            Err(eraro) => {
                crate::skribu!("[ipc] tabeloj (MALBONE): neatendita eraro {:?}\n", eraro);
                return;
            }
        }
    }
    let plena = matches!(
        krei_finpunkton(procezo, Permesoj::PLENAJ),
        Err(IpcEraro::TabeloPlena)
    );
    let unua = teniloj[0];
    let mut fermitaj = 0;
    for tenilo in &teniloj[..nombro] {
        if fermi(procezo, *tenilo).is_ok() {
            fermitaj += 1;
        }
    }
    let duobla = matches!(fermi(procezo, unua), Err(IpcEraro::NevalidaTenilo));
    let rekrea = krei_finpunkton(procezo, Permesoj::SENDI);
    let nova_generacio = rekrea
        .as_ref()
        .is_ok_and(|tenilo| tenilo.indekso() == unua.indekso());
    let malnova = matches!(fermi(procezo, unua), Err(IpcEraro::NevalidaTenilo));
    if let Ok(tenilo) = rekrea {
        let _ = fermi(procezo, tenilo);
    }
    let kerno = matches!(
        krei_finpunkton(0, Permesoj::PLENAJ),
        Err(IpcEraro::TabeloPlena)
    );
    let bone = nombro > 0
        && nombro < MAKSAJ_FINPUNKTOJ
        && plena
        && fermitaj == nombro
        && duobla
        && nova_generacio
        && malnova
        && kerno;
    crate::skribu!(
        "[ipc] tabeloj ({}): kreitaj={}, plena={}, fermitaj={}, duobla_fermo={}, nova_generacio={}, malnova_tenilo={}, kerno={}\n",
        if bone { "bone" } else { "MALBONE" },
        nombro,
        plena,
        fermitaj,
        duobla,
        nova_generacio,
        malnova,
        kerno
    );
}

/// Ring 0-memtesto de la permesoj, de la kapabla izolado kaj de `DONI`.
///
/// La testoj uzas la kapabloliniojn de la rezervitaj procezaj identigiloj `6`
/// kaj `7`; ĉiuj kreitaj kapabloj fermiĝas antaŭ la unua `lanĉi()`, do la
/// realaj procezoj ne vidas ilin.
pub fn testi_kapablojn() {
    let procezo: ProcezaId = 6;
    let alia: ProcezaId = 7;
    let mesago = Mesago::malplena();

    // Rajtoj: la kapablo devas posedi la koncernan permeson.
    let (sendi_per_riceva, ricevi_per_senda) = match (
        krei_finpunkton(procezo, Permesoj::RICEVI),
        krei_finpunkton(procezo, Permesoj::SENDI),
    ) {
        (Ok(riceva), Ok(senda)) => {
            let sendi = sendi_kapable(procezo, riceva, &mesago, Atendo::Senfina)
                .err()
                .map_or(0, IpcEraro::kodo);
            let ricevi = ricevi_kapable(procezo, senda, None, Atendo::Senfina)
                .err()
                .map_or(0, IpcEraro::kodo);
            let _ = fermi(procezo, riceva);
            let _ = fermi(procezo, senda);
            (sendi, ricevi)
        }
        _ => (0, 0),
    };
    let rajtoj = sendi_per_riceva == -1 && ricevi_per_senda == -1;
    crate::skribu!(
        "[ipc] rajtoj ({}): sendi per RICEVI-nura → {}, ricevi per SENDI-nura → {}\n",
        if rajtoj { "bone" } else { "MALBONE" },
        sendi_per_riceva,
        ricevi_per_senda
    );

    // Izolado: tenilo de unu procezo ne validas en la tabelo de alia.
    let izolado = match krei_finpunkton(procezo, Permesoj::PLENAJ) {
        Ok(tenilo) => {
            let kodo = sendi_kapable(alia, tenilo, &mesago, Atendo::Senfina)
                .err()
                .map_or(0, IpcEraro::kodo);
            let _ = fermi(procezo, tenilo);
            kodo
        }
        Err(_) => 0,
    };
    crate::skribu!(
        "[ipc] izolado de kapabloj ({}): procezo {} uzas tenilon de procezo {} → {}\n",
        if izolado == -9 { "bone" } else { "MALBONE" },
        alia,
        procezo,
        izolado
    );

    // Dono: nur malpliiĝo; la celo ricevas propran tenilon.
    let mut donitaj = (0u8, 0u8);
    let mut eskalo = 0;
    if let Ok(fonto) = krei_finpunkton(procezo, Permesoj::ĈIUJ) {
        if let Ok(celo) = doni(procezo, fonto, alia, Permesoj::RICEVI) {
            let sendita = sendi_kapable(procezo, fonto, &mesago, Atendo::Senfina).is_ok();
            let ricevita = sendita && ricevi_kapable(alia, celo, None, Atendo::Senfina).is_ok();
            let rifuzo = sendi_kapable(alia, celo, &mesago, Atendo::Senfina)
                .err()
                .map_or(0, IpcEraro::kodo);
            donitaj = if ricevita && rifuzo == -1 {
                (Permesoj::ĈIUJ.bitoj(), Permesoj::RICEVI.bitoj())
            } else {
                (0, 0)
            };
            let _ = fermi(alia, celo);
        }
        eskalo = doni(procezo, fonto, alia, Permesoj::REVOKI)
            .err()
            .map_or(0, IpcEraro::kodo);
        let _ = fermi(procezo, fonto);
    }
    let dono_bone = donitaj == (Permesoj::ĈIUJ.bitoj(), Permesoj::RICEVI.bitoj()) && eskalo == -1;
    crate::skribu!(
        "[ipc] dono ({}): rajtoj 0b{:03b} → 0b{:03b}, eskalado 0b{:03b} → {}\n",
        if dono_bone { "bone" } else { "MALBONE" },
        donitaj.0,
        donitaj.1,
        Permesoj::REVOKI.bitoj(),
        eskalo
    );
}

/// Ring 0-memtesto de `REVOKI`: la voko estas tutfinpunkta kaj postulas la
/// permeson `REVOKI`; revoko malvalidigas kaj la tenilon de la revokanto kaj
/// la tenilon donitan al alia procezo.
pub fn testi_revokon() {
    let procezo: ProcezaId = 6;
    let alia: ProcezaId = 7;
    let mesago = Mesago::malplena();

    // Sen la permeso `REVOKI` la voko estas malakceptata per `-1`.
    let senpermesa = match krei_finpunkton(procezo, Permesoj::ĈIUJ) {
        Ok(tenilo) => {
            let kodo = revoki(procezo, tenilo).err().map_or(0, IpcEraro::kodo);
            let _ = fermi(procezo, tenilo);
            kodo
        }
        Err(_) => 0,
    };

    // Tutfinpunkta revoko: ankaŭ la tenilo donita al `alia` malaperas.
    let mut revokita = false;
    let mut post = (0i64, 0i64, 0i64);
    if let Ok(fonto) = krei_finpunkton(procezo, Permesoj::PLENAJ) {
        if let Ok(donita) = doni(procezo, fonto, alia, Permesoj::PLENAJ)
            && revoki(procezo, fonto).is_ok()
        {
            revokita = true;
            post = (
                sendi_kapable(alia, donita, &mesago, Atendo::Senfina)
                    .err()
                    .map_or(0, IpcEraro::kodo),
                fermi(alia, donita).err().map_or(0, IpcEraro::kodo),
                revoki(procezo, fonto).err().map_or(0, IpcEraro::kodo),
            );
        }
        let _ = fermi(procezo, fonto);
    }
    let bone = senpermesa == -1 && revokita && post == (-9, -9, -9);
    crate::skribu!(
        "[ipc] revoko ({}): sen_revoki={}, revoko={}, donita={}, fermo={}, duobla={}\n",
        if bone { "bone" } else { "MALBONE" },
        senpermesa,
        if revokita { 0 } else { 1 },
        post.0,
        post.1,
        post.2
    );
}

#[cfg(kani)]
mod kani_proofs {
    //! Kani-pruvoj por la mesaĝujo.
    //!
    //! `sendi_mesagon`, `ricevi_mesagon`, `sendi_kapable` kaj `ricevi_kapable`
    //! dependas de la planisto, de interrompoj kaj de tutmonda stato, kiuj ne
    //! estas modeligeblaj per Kani sen eliri la no_std-medion. La pruvoj do
    //! celas la enan `Mesagujo`-kodon, kiun la funkcioj rekte uzas.
    //!
    //! La pruvoj uzas la plej malgrandan `unwind`-limon, kiun la koncerna
    //! buklo bezonas: 8 por la FIFO-pruvoj (maksimume tri mesaĝoj kaj tri
    //! `preni_de`-paŝoj), 20 por la plenigo de la 16 fendoj, 70 por la
    //! 64-bajta kopio de `Mesago::nova`. La mesaĝoj de la vicaj pruvoj havas
    //! malplenan datumaron, ĉar la vica ordo dependas nur de `tipo` kaj de la
    //! fonto; la plenlongan buferlimon aparte pruvas `mesaĝa_enhavo_en_limoj`.

    use crate::kerno::objekto::FadenoId;

    use super::{Ero, IpcEraro, MAKSA_MESAGA_LONGO, MAKSAJ_MESAGOJ, Mesago, Mesagujo};

    /// Arbitra mesaĝo kun fiksa `tipo`. La vicaj pruvoj nur dependas de `tipo`
    /// kaj de la fonto, do la datumaro restas malplena kaj la kopi-buklo de
    /// `Mesago::nova` malaperas el la SAT-formulo.
    fn iu_mesago(tipo: u64) -> Mesago {
        Mesago::nova(tipo, 0, 0, &[])
    }

    /// Arbitra mesaĝo, kies `longo` rajtas atingi la realan buferlimon.
    fn plenlonga_mesago() -> Mesago {
        let longo: usize = kani::any();
        kani::assume(longo <= MAKSA_MESAGA_LONGO);
        let datumo: [u8; MAKSA_MESAGA_LONGO] = kani::any();
        Mesago::nova(kani::any(), kani::any(), kani::any(), &datumo[..longo])
    }

    fn iu_ero(tipo: u64) -> Ero {
        Ero {
            fonto: kani::any(),
            mesago: iu_mesago(tipo),
        }
    }

    /// Mesaĝa `longo` neniam eliras la datumaron.
    #[kani::proof]
    #[kani::unwind(70)]
    fn mesaĝa_enhavo_en_limoj() {
        let mesago = plenlonga_mesago();
        assert_eq!(mesago.enhavo().len(), mesago.longo());
        assert!(mesago.longo() <= MAKSA_MESAGA_LONGO);
    }

    /// Plena mesaĝujo rifuzas pliajn mesaĝojn sen ŝanĝi `nombro`.
    #[kani::proof]
    #[kani::unwind(20)]
    fn plena_mesagujo_rifuzas() {
        let mut ujo = Mesagujo::nova();
        let fonto: FadenoId = kani::any();

        let mut indekso = 0;
        while indekso < MAKSAJ_MESAGOJ {
            let ero = Ero {
                fonto,
                mesago: Mesago::malplena(),
            };
            assert!(ujo.pusxi(ero).is_ok());
            indekso += 1;
        }
        assert_eq!(ujo.nombro, MAKSAJ_MESAGOJ);
        assert!(ujo.eroj[0].is_some());

        assert_eq!(ujo.pusxi(iu_ero(1)), Err(IpcEraro::MesagujoPlena));
        assert_eq!(ujo.nombro, MAKSAJ_MESAGOJ);
        assert!(ujo.eroj[0].is_some());
    }

    /// FIFO: de la sama fonto, la unue sendita mesaĝo unue revenas.
    #[kani::proof]
    #[kani::unwind(8)]
    fn fifo_ordo() {
        let mut ujo = Mesagujo::nova();
        let fonto: FadenoId = kani::any();
        let unua = iu_mesago(1);
        let dua = iu_mesago(2);

        assert!(
            ujo.pusxi(Ero {
                fonto,
                mesago: unua
            })
            .is_ok()
        );
        assert!(ujo.pusxi(Ero { fonto, mesago: dua }).is_ok());

        let ricevita = ujo.preni_de(Some(fonto)).unwrap();
        assert_eq!(ricevita.tipo, 1);
        assert_eq!(ricevita.longo(), unua.longo());

        let ricevita = ujo.preni_de(Some(fonto)).unwrap();
        assert_eq!(ricevita.tipo, 2);
        assert_eq!(ricevita.longo(), dua.longo());

        assert_eq!(ujo.nombro, 0);
        assert!(ujo.preni_de(Some(fonto)).is_none());
        assert!(ujo.eroj[0].is_none());
    }

    /// FIFO restas valida ankaŭ inter malsamaj fontoj: forpreno de unu fonto
    /// ne renversas la ordon de la aliaj.
    #[kani::proof]
    #[kani::unwind(8)]
    fn fifo_ordo_inter_fontoj() {
        let mut ujo = Mesagujo::nova();
        let fonto_a: FadenoId = kani::any();
        let fonto_b: FadenoId = kani::any();
        kani::assume(fonto_a != fonto_b);

        assert!(
            ujo.pusxi(Ero {
                fonto: fonto_a,
                mesago: iu_mesago(1),
            })
            .is_ok()
        );
        assert!(
            ujo.pusxi(Ero {
                fonto: fonto_b,
                mesago: iu_mesago(2),
            })
            .is_ok()
        );
        assert!(
            ujo.pusxi(Ero {
                fonto: fonto_a,
                mesago: iu_mesago(3),
            })
            .is_ok()
        );

        assert_eq!(ujo.preni_de(Some(fonto_a)).unwrap().tipo, 1);
        assert_eq!(ujo.preni_de(Some(fonto_b)).unwrap().tipo, 2);
        assert_eq!(ujo.preni_de(Some(fonto_a)).unwrap().tipo, 3);
        assert!(ujo.preni_de(Some(fonto_a)).is_none());
        assert!(ujo.preni_de(Some(fonto_b)).is_none());
        assert_eq!(ujo.nombro, 0);
    }

    /// Fonta filtro: fremdaj mesaĝoj restas en la vico, la unua kongrua
    /// mesaĝo forpreniĝas, kaj senfiltra preno ankoraŭ redonas la fremdan.
    #[kani::proof]
    #[kani::unwind(8)]
    fn fonta_filtro_konservas_fremdajn() {
        let mut ujo = Mesagujo::nova();
        let fonto_a: FadenoId = kani::any();
        let fonto_b: FadenoId = kani::any();
        kani::assume(fonto_a != fonto_b);

        assert!(
            ujo.pusxi(Ero {
                fonto: fonto_a,
                mesago: iu_mesago(1),
            })
            .is_ok()
        );
        assert!(
            ujo.pusxi(Ero {
                fonto: fonto_b,
                mesago: iu_mesago(2),
            })
            .is_ok()
        );
        assert!(
            ujo.pusxi(Ero {
                fonto: fonto_a,
                mesago: iu_mesago(3),
            })
            .is_ok()
        );

        assert_eq!(ujo.preni_de(Some(fonto_a)).unwrap().tipo, 1);
        assert_eq!(ujo.nombro, 2);
        assert_eq!(ujo.preni_de(Some(fonto_a)).unwrap().tipo, 3);
        assert_eq!(ujo.nombro, 1);
        assert_eq!(ujo.preni_de(None).unwrap().tipo, 2);
        assert_eq!(ujo.nombro, 0);
        assert!(ujo.preni_de(Some(fonto_b)).is_none());
    }

    /// Malplena mesaĝujo redonas `None` sen subtrahi de nulo.
    #[kani::proof]
    fn malplena_mesagujo_ricevas_nenion() {
        let mut ujo = Mesagujo::nova();
        let fonto: FadenoId = kani::any();

        assert!(ujo.preni_de(Some(fonto)).is_none());
        assert_eq!(ujo.nombro, 0);
        assert!(ujo.eroj[0].is_none());
    }

    /// Forpreno konservas la invariantojn: `nombro` restas en limoj kaj ĉiuj
    /// fendoj ekster `nombro` restas malplenaj.
    #[kani::proof]
    #[kani::unwind(20)]
    fn preno_konservas_la_invarianton() {
        let mut ujo = Mesagujo::nova();
        let fonto: FadenoId = kani::any();
        assert!(ujo.pusxi(iu_ero(1)).is_ok());

        let _ = ujo.preni_de(Some(fonto));
        assert!(ujo.nombro <= MAKSAJ_MESAGOJ);

        let mut indekso = ujo.nombro;
        while indekso < MAKSAJ_MESAGOJ {
            assert!(ujo.eroj[indekso].is_none());
            indekso += 1;
        }
    }
}

#[cfg(kani)]
mod kani_tenilo {
    //! Pruvo de la `Tenilo`-kodigo: valida paro (indekso, generacio) revenas
    //! senŝanĝe, dum generacio `0` kaj indekso super `15` estas rifuzataj — kaj
    //! ĉe la kunmeto kaj ĉe la malpakado de kruda valoro.

    use libmikerno::Tenilo;

    #[kani::proof]
    fn tenilo_kodigo_roundtrip() {
        let indekso: u8 = kani::any();
        let generacio: u32 = kani::any();
        let valida = indekso <= Tenilo::MAKSA_INDEKSO
            && generacio != 0
            && generacio <= Tenilo::MAKSA_GENERACIO;

        match Tenilo::nova(indekso, generacio) {
            Some(tenilo) => {
                assert!(valida);
                assert_eq!(tenilo.indekso(), indekso);
                assert_eq!(tenilo.generacio(), generacio);
                let rekonstruita = Tenilo::el_nombro(tenilo.nombro()).unwrap();
                assert!(rekonstruita == tenilo);
            }
            None => assert!(!valida),
        }

        // Generacio 0 kaj indekso super 15 estas malvalidaj en ambaŭ vojoj.
        assert!(Tenilo::nova(0, 0).is_none());
        assert!(Tenilo::nova(16, 1).is_none());
        assert!(Tenilo::nova(255, 1).is_none());
        assert!(Tenilo::el_nombro(0).is_none());
        assert!(Tenilo::el_nombro((1 << 8) | 16).is_none());

        let kruda: u32 = kani::any();
        match Tenilo::el_nombro(kruda) {
            Some(tenilo) => {
                assert!(tenilo.indekso() <= Tenilo::MAKSA_INDEKSO);
                assert_ne!(tenilo.generacio(), 0);
                assert_eq!(tenilo.nombro(), kruda);
            }
            None => {
                let indekso = (kruda & 0xFF) as u8;
                assert!(indekso > Tenilo::MAKSA_INDEKSO || kruda >> 8 == 0);
            }
        }
    }
}

#[cfg(kani)]
mod kani_finpunktoj {
    //! Pruvoj por la generacia kaj referenca aritmetiko kaj por la elekto de
    //! libera finpunkta fendo.

    use libmikerno::Tenilo;

    use crate::kerno::objekto::FadenoId;
    use crate::kerno::planisto::MAKSAJ_FADENOJ;

    use super::{
        FinaPunktaStato, MAKSAJ_FINPUNKTOJ, SendaDecido, akordas, elekti_liberan,
        forprenu_referencon, pliigita_generacio, senda_bito, senda_decido,
    };

    /// La sendanta bito estas unu-uma kaj kongruas kun la vek-buklo, kiu traktas
    /// la plej malaltan biton kiel fadenon `1`.
    #[kani::proof]
    fn senda_bito_unuuma() {
        let fadeno: FadenoId = kani::any();
        kani::assume(fadeno >= 1 && fadeno <= MAKSAJ_FADENOJ as FadenoId);
        let bito = senda_bito(fadeno);
        assert_eq!(bito.count_ones(), 1);
        assert_eq!(bito.trailing_zeros() + 1, fadeno as u32);
    }

    /// La premrega politiko: plena vico nur atendigas sendanton, kiu povas
    /// bloki, kaj neniam rifuzas mesaĝon dum loko restas.
    #[kani::proof]
    fn premrego_postulas_plenan_vicon() {
        let enmetitaj: usize = kani::any();
        let maks: usize = kani::any();
        let povas_bloki: bool = kani::any();
        match senda_decido(enmetitaj, maks, povas_bloki) {
            SendaDecido::Enmetu => assert!(enmetitaj < maks),
            SendaDecido::Atendu => assert!(enmetitaj >= maks && povas_bloki),
            SendaDecido::Rifuzi => assert!(enmetitaj >= maks && !povas_bloki),
        }
    }

    /// La sekva generacio estas strikte pli granda, neniam nula, kaj la fendo
    /// emeritiĝas ekzakte ĉe la eluziĝo de la 24 bitoj.
    #[kani::proof]
    fn generacio_emeritigas() {
        let generacio: u32 = kani::any();
        match pliigita_generacio(generacio) {
            Some(sekva) => {
                assert!(generacio >= 1 && generacio < Tenilo::MAKSA_GENERACIO);
                assert_eq!(sekva, generacio + 1);
                assert!(sekva > generacio);
                assert!(sekva <= Tenilo::MAKSA_GENERACIO);
            }
            None => assert!(generacio == 0 || generacio >= Tenilo::MAKSA_GENERACIO),
        }
    }

    /// La referenckalkulo malaltiĝas per unu kaj saturiĝas ĉe nulo.
    #[kani::proof]
    fn referenco_saturigas() {
        let referencoj: u16 = kani::any();
        let post = forprenu_referencon(referencoj);
        if referencoj == 0 {
            assert_eq!(post, 0);
        } else {
            assert_eq!(post, referencoj - 1);
        }
    }

    /// Revoko altigas la finpunktan generacion, do neniu kapabla fendo kun la
    /// malnova generacio plu kongruas; Libera aŭ Emerita finpunkto ankaŭ ne
    /// akordas, eĉ kun egala generacio.
    #[kani::proof]
    fn revoko_malvalidigas_tenilojn() {
        let generacio: u32 = kani::any();
        assert!(akordas(true, generacio, generacio));
        assert!(!akordas(false, generacio, generacio));
        match pliigita_generacio(generacio) {
            Some(sekva) => assert!(!akordas(true, sekva, generacio)),
            None => assert!(generacio == 0 || generacio >= Tenilo::MAKSA_GENERACIO),
        }
    }

    /// Se la elekto trovas fendon, ĝi estas la unua libera; se ĝi redonas
    /// `None`, neniu fendo estas libera.
    #[kani::proof]
    #[kani::unwind(20)]
    fn libera_fendo_elektita() {
        let mut statoj = [FinaPunktaStato::Emerita; MAKSAJ_FINPUNKTOJ];
        let mut indekso = 0;
        while indekso < MAKSAJ_FINPUNKTOJ {
            let valoro: u8 = kani::any();
            statoj[indekso] = match valoro % 3 {
                0 => FinaPunktaStato::Libera,
                1 => FinaPunktaStato::Aktiva,
                _ => FinaPunktaStato::Emerita,
            };
            indekso += 1;
        }

        let ekzistas = statoj.iter().any(|stato| *stato == FinaPunktaStato::Libera);
        match elekti_liberan(&statoj) {
            Some(fendo) => {
                assert_eq!(statoj[fendo as usize], FinaPunktaStato::Libera);
                let mut antaŭa = 0;
                while antaŭa < fendo as usize {
                    assert_ne!(statoj[antaŭa], FinaPunktaStato::Libera);
                    antaŭa += 1;
                }
            }
            None => assert!(!ekzistas),
        }
    }
}

#[cfg(kani)]
mod kani_tempolimoj {
    //! Pruvoj por la `r10`-politiko: la du specialaj valoroj, la rifuzo super
    //! `u32::MAX` kaj la suprenrondigita konverto al tikoj (C15).

    use super::{Atendo, el_tempolimo};

    /// Ĉiu valoro havas ekzakte unu signifon, kaj `Tempa` neniam estas nula
    /// nek malsuprenrondigita.
    #[kani::proof]
    fn tempolima_politiko() {
        let valoro: u64 = kani::any();
        match el_tempolimo(valoro) {
            Some(Atendo::Senfina) => assert_eq!(valoro, u32::MAX as u64),
            Some(Atendo::Nebloka) => assert_eq!(valoro, 0),
            Some(Atendo::Tempa(tikoj)) => {
                assert!(valoro >= 1 && valoro < u32::MAX as u64);
                assert!(tikoj >= 1);
                assert!(tikoj <= u32::MAX.div_ceil(10) as u64);
                assert!(tikoj * 10 >= valoro);
                assert!((tikoj - 1) * 10 < valoro);
            }
            None => assert!(valoro > u32::MAX as u64),
        }
    }

    /// Pozitiva tempolimo sub `u32::MAX` ĉiam fariĝas `Tempa` kun almenaŭ unu
    /// tiko; 1–9 ms do ne fariĝas tuja `-110`.
    #[kani::proof]
    fn tempolimo_ne_nuligas() {
        let ms: u32 = kani::any();
        kani::assume(ms >= 1 && ms < u32::MAX);
        match el_tempolimo(ms as u64) {
            Some(Atendo::Tempa(tikoj)) => {
                assert!(tikoj >= 1);
                assert!(tikoj * 10 >= ms as u64);
            }
            _ => panic!("pozitiva tempolimo devas esti Tempa"),
        }
    }
}

#[cfg(kani)]
mod kani_kapabloj {
    //! Pruvoj por la kapabla tavolo: permesoj, dono nur-malpliiĝa kaj proceza
    //! izolado. La tutmondaj tabeloj ne estas modeligeblaj, do la pruvoj uzas
    //! malgrandajn lokajn `Kapablotabelo`-ojn.

    use libmikerno::{Permesoj, Tenilo};

    use super::{IpcEraro, Kapablotabelo, dono_permesita, serĉi_kapablon};

    fn iuj_permesoj() -> Permesoj {
        let bitoj: u8 = kani::any();
        Permesoj::el_bitoj(bitoj & Permesoj::PLENAJ.bitoj()).unwrap()
    }

    /// Sen `SENDI` la kapablo ne permesas sendi.
    #[kani::proof]
    fn rajtoj_devigas_sendon() {
        let permesoj = iuj_permesoj();
        let mut tabelo = Kapablotabelo::nova();
        let tenilo = tabelo.instalu_en(0, 0, 1, permesoj).unwrap();
        let rezulto = serĉi_kapablon(&tabelo, tenilo, Permesoj::SENDI);
        if permesoj.posedas(Permesoj::SENDI) {
            assert_eq!(rezulto, Ok(0));
        } else {
            assert_eq!(rezulto, Err(IpcEraro::NesufiĉajPermesoj));
        }
    }

    /// Sen `RICEVI` la kapablo ne permesas ricevi.
    #[kani::proof]
    fn rajtoj_devigas_ricevon() {
        let permesoj = iuj_permesoj();
        let mut tabelo = Kapablotabelo::nova();
        let tenilo = tabelo.instalu_en(0, 0, 1, permesoj).unwrap();
        let rezulto = serĉi_kapablon(&tabelo, tenilo, Permesoj::RICEVI);
        if permesoj.posedas(Permesoj::RICEVI) {
            assert_eq!(rezulto, Ok(0));
        } else {
            assert_eq!(rezulto, Err(IpcEraro::NesufiĉajPermesoj));
        }
    }

    /// `DONI` akceptas nur subaron de la fontaj permesoj.
    #[kani::proof]
    fn dono_nur_malpligrandigas_rajtajn() {
        let fontaj = iuj_permesoj();
        let petitaj = iuj_permesoj();
        let permesita = dono_permesita(fontaj, petitaj);
        if permesita {
            assert_eq!(petitaj.bitoj() & !fontaj.bitoj(), 0);
        } else {
            assert_ne!(petitaj.bitoj() & !fontaj.bitoj(), 0);
        }
        assert!(dono_permesita(fontaj, fontaj));
    }

    /// Tenilo el la tabelo de unu procezo ne validas en la tabelo de alia.
    #[kani::proof]
    fn transproceza_tenilo_nevalidas() {
        let mut unua = Kapablotabelo::nova();
        let dua = Kapablotabelo::nova();
        let tenilo = unua.instalu_en(0, 0, 1, Permesoj::PLENAJ).unwrap();
        assert_eq!(serĉi_kapablon(&unua, tenilo, Permesoj::NENIU), Ok(0));
        assert_eq!(
            serĉi_kapablon(&dua, tenilo, Permesoj::NENIU),
            Err(IpcEraro::NevalidaTenilo)
        );
        // La sama fenda indekso kun alia generacio ankaŭ estas rifuzata.
        let malnova = Tenilo::nova(0, 2).unwrap();
        assert_eq!(
            serĉi_kapablon(&unua, malnova, Permesoj::NENIU),
            Err(IpcEraro::NevalidaTenilo)
        );
    }
}
