//! Unuprocesora antaŭokupa planisto kun cirkla preta vico.
//!
//! Ĉiuj modifoj okazas kun interrompoj malŝaltitaj. La temporizila traktilo
//! vokas `planumi`; fadeno povas cedi per `cedi`, blokiĝi per `bloki` aŭ
//! finiĝi per `morti`. La ripoza fadeno (id 0) portas la praŝarĝan kuntekston.

use crate::arkitekturo::{self, Kunteksto};
use crate::interrompo::InterrompaGardo;
use crate::kunhavigita::Kunhavigita;
use crate::memoro::virtuala::VirtuaAdreso;

use super::objekto::{Fadeno, FadenoEnirpunkto, FadenoId, FadenoStato, Prioritato, ProcezaId};
use super::procezo::{ProcezaEstado, marki};
use super::vekotemporizilo::Vekotemporizilo;

pub const MAKSAJ_FADENOJ: usize = 8;
pub const MAKSA_PRETA_ATENDO: u64 =
    (MAKSAJ_FADENOJ as u64 - 1) * Prioritato::Alta.tempopeco() as u64;
const RIPAZA_ID: FadenoId = 0;
/// La unua fadena identigilo por dinamikaj uzantaj procezoj.
///
/// La identigiloj `1..=4` apartenas al la kernaj provfadenoj; la statistiko
/// legas iliajn `veturoj`- kaj `plenumtikoj`-erojn poste, do ili ne reuziĝas.
/// La izolaj procezoj jam finiĝis, kiam la ŝelo kreiĝas, kaj iliaj fenestroj
/// `5`/`6` estas liberaj.
const UNUA_DINAMIKA: FadenoId = 5;

/// Cirkla vico kun unu plia loko, do `kapo == vosto` ĉiam signifas malplena.
struct PretaVico {
    eroj: [FadenoId; MAKSAJ_FADENOJ + 1],
    kapo: usize,
    vosto: usize,
}

impl PretaVico {
    const fn nova() -> Self {
        Self {
            eroj: [0; MAKSAJ_FADENOJ + 1],
            kapo: 0,
            vosto: 0,
        }
    }

    fn puŝi(&mut self, id: FadenoId) {
        self.eroj[self.vosto] = id;
        self.vosto = (self.vosto + 1) % self.eroj.len();
    }

    fn pop(&mut self) -> Option<FadenoId> {
        if self.kapo == self.vosto {
            return None;
        }
        let id = self.eroj[self.kapo];
        self.kapo = (self.kapo + 1) % self.eroj.len();
        Some(id)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum PlanistaEraro {
    NevalidaIdentigilo,
    JamRegistrita,
}

pub struct Planisto {
    fadenoj: [Option<Fadeno>; MAKSAJ_FADENOJ + 1],
    preta_vico: PretaVico,
    nuna_fadeno: Option<FadenoId>,
    restaj_tikoj: u8,
    /// La minimum-kupo de blokitaj fadenoj kun limdato; malplena limdato ne
    /// estas ero.
    vekoj: Vekotemporizilo,
    /// La fadeno vekiĝis pro sia limdato, ne pro operacio; la markon konsumas
    /// la atendanta voko.
    tempolimoj: [bool; MAKSAJ_FADENOJ + 1],
    ŝaltoj: u64,
    limdataj_blokoj: u64,
    vekigoj: u64,
    maks_vekprokrasto: u64,
    veturoj: [u64; MAKSAJ_FADENOJ + 1],
    plenumtikoj: [u64; MAKSAJ_FADENOJ + 1],
}

/// Nombroj pri la planado ekde la starto; ilin montras la fina raporto.
#[derive(Clone, Copy, Debug)]
pub struct PlanistaStatistiko {
    /// Kuntekstŝaltaj vokoj (`ŝalti_kuntekston`).
    pub ŝaltoj: u64,
    /// Blokadoj kun limdato (dormo aŭ IK-tempolimo).
    pub limdataj_blokoj: u64,
    /// Vekoj pro limdato, ne pro operacio.
    pub vekigoj: u64,
    /// La plej granda prokrasto inter la limdato kaj la veko, en tikoj.
    pub maks_vekprokrasto: u64,
}

impl Planisto {
    pub const fn nova() -> Self {
        let mut fadenoj = [None; MAKSAJ_FADENOJ + 1];
        fadenoj[RIPAZA_ID as usize] = Some(Fadeno::ripoza());
        Self {
            fadenoj,
            preta_vico: PretaVico::nova(),
            nuna_fadeno: None,
            restaj_tikoj: 0,
            vekoj: Vekotemporizilo::nova(),
            tempolimoj: [false; MAKSAJ_FADENOJ + 1],
            ŝaltoj: 0,
            limdataj_blokoj: 0,
            vekigoj: 0,
            maks_vekprokrasto: 0,
            veturoj: [0; MAKSAJ_FADENOJ + 1],
            plenumtikoj: [0; MAKSAJ_FADENOJ + 1],
        }
    }

    pub fn aldoni_fadenon(&mut self, fadeno: Fadeno) -> Result<(), PlanistaEraro> {
        let indekso = fadeno.id as usize;
        if fadeno.id == RIPAZA_ID || indekso >= self.fadenoj.len() {
            return Err(PlanistaEraro::NevalidaIdentigilo);
        }
        // Mortinta fadeno lasas sian eron en la tabelo por la statistiko; la
        // identigilo reuzeblas, kiam la malnova fadeno jam ne kuras.
        if self.fadenoj[indekso]
            .as_ref()
            .is_some_and(|ekzistanta| ekzistanta.stato != FadenoStato::Mortinta)
        {
            return Err(PlanistaEraro::JamRegistrita);
        }
        let _ = self.vekoj.forigi(fadeno.id);
        self.tempolimoj[indekso] = false;
        self.preta_vico.puŝi(fadeno.id);
        self.fadenoj[indekso] = Some(fadeno);
        Ok(())
    }

    fn elekti_proksiman(&mut self) -> Option<FadenoId> {
        while let Some(id) = self.preta_vico.pop() {
            if self.fadenoj[id as usize]
                .as_ref()
                .is_some_and(|fadeno| fadeno.stato == FadenoStato::Preta)
            {
                return Some(id);
            }
        }
        None
    }

    fn kurigu(&mut self, id: FadenoId) {
        if let Some(fadeno) = self.fadenoj[id as usize].as_mut() {
            fadeno.stato = FadenoStato::Kuranta;
            self.restaj_tikoj = fadeno.prioritato.tempopeco();
        }
        self.veturoj[id as usize] = self.veturoj[id as usize].saturating_add(1);
        self.nuna_fadeno = Some(id);
        self.stato_procezan(id, ProcezaEstado::Kuranta);
    }

    /// La procezo de la donita fadeno; `0` por nekonata identigilo aŭ kernaj
    /// fadenoj (`procezo == 0`).
    fn procezo_de(&self, id: FadenoId) -> ProcezaId {
        self.fadenoj
            .get(id as usize)
            .and_then(|fadeno| fadeno.as_ref())
            .map_or(0, |fadeno| fadeno.procezo)
    }

    /// Spegulu la fadenan staton en la proceza tabelo; fadenoj de la nukleo
    /// (`procezo == 0`) ne havas procezan eron.
    fn stato_procezan(&self, id: FadenoId, stato: ProcezaEstado) {
        let procezo = self.procezo_de(id);
        if procezo != 0 {
            marki(procezo, stato);
        }
    }

    fn elekti_por_tiko(&mut self, cedo: bool) -> Option<FadenoId> {
        // La specimeno apartenas al la interrompita fadeno, ne al la sekva elektito.
        if !cedo {
            let indekso = self.nuna_id() as usize;
            self.plenumtikoj[indekso] = self.plenumtikoj[indekso].saturating_add(1);
        }
        let nuna = self.nuna_fadeno?;
        if nuna == RIPAZA_ID {
            return self.elekti_proksiman();
        }
        let fadeno = self.fadenoj[nuna as usize].as_mut()?;
        if fadeno.stato != FadenoStato::Kuranta {
            return None;
        }
        if !cedo && self.restaj_tikoj > 1 {
            self.restaj_tikoj -= 1;
            return None;
        }
        fadeno.stato = FadenoStato::Preta;
        self.preta_vico.puŝi(nuna);
        self.stato_procezan(nuna, ProcezaEstado::Preta);
        Some(self.elekti_proksiman().unwrap_or(nuna))
    }

    fn planumi_kun_cedo(&mut self, cedo: bool) {
        self.venkigi_tempolimojn();
        let nuna = self.nuna_id();
        if let Some(sekva) = self.elekti_por_tiko(cedo) {
            self.kurigu(sekva);
            if sekva != nuna {
                unsafe { self.ŝanĝi(nuna, sekva) };
            }
        }
    }

    pub fn planumi(&mut self) {
        self.planumi_kun_cedo(false);
    }

    /// Veku ĉiujn blokitajn fadenojn, kies limdato jam pasis. La vekito vidas
    /// la tempoliman markon anstataŭ la normalan operacian vekon.
    fn venkigi_tempolimojn(&mut self) {
        let nuna = crate::arkitekturo::temporizilaj_tikoj();
        while let Some((limdato, id)) = self.vekoj.pinti() {
            if limdato > nuna {
                break;
            }
            self.vekoj.elpreni();
            if self.fadenoj[id as usize].is_some_and(|fadeno| fadeno.stato == FadenoStato::Blokitaj)
            {
                self.tempolimoj[id as usize] = true;
                self.maks_vekprokrasto = self.maks_vekprokrasto.max(nuna - limdato);
                self.vekigoj = self.vekigoj.saturating_add(1);
                self.veki_sen_limdato(id);
            }
        }
    }

    pub fn bloki(&mut self, id: FadenoId) {
        self.bloki_kun_limdato(id, 0);
    }

    /// Bloku la fadenon ĝis la absoluta tiko `limdato`; `0` signifas sen
    /// limdato. La marko de antaŭa tempolimo ĉiam malpleniĝas.
    pub fn bloki_kun_limdato(&mut self, id: FadenoId, limdato: u64) {
        let _ = self.vekoj.forigi(id);
        if limdato != 0 {
            self.vekoj.enmeti(id, limdato);
            self.limdataj_blokoj = self.limdataj_blokoj.saturating_add(1);
        }
        if let Some(marko) = self.tempolimoj.get_mut(id as usize) {
            *marko = false;
        }
        if let Some(fadeno) = self.fadenoj[id as usize].as_mut() {
            fadeno.stato = FadenoStato::Blokitaj;
        }
        // Procezo sen kuranta fadeno ne plu estas "kuranta".
        self.stato_procezan(id, ProcezaEstado::Preta);
        let sekva = self.elekti_proksiman().unwrap_or(RIPAZA_ID);
        self.kurigu(sekva);
        unsafe { self.ŝanĝi(id, sekva) };
    }

    pub fn veki(&mut self, id: FadenoId) {
        let _ = self.vekoj.forigi(id);
        self.veki_sen_limdato(id);
    }

    fn veki_sen_limdato(&mut self, id: FadenoId) {
        let Some(fadeno) = self.fadenoj[id as usize].as_mut() else {
            return;
        };
        if fadeno.stato != FadenoStato::Blokitaj {
            return;
        }
        fadeno.stato = FadenoStato::Preta;
        self.preta_vico.puŝi(id);
    }

    /// Trovu la vivantan fadenon de la procezo `proceza` (se estas), sen
    /// ŝanĝi ion ajn.
    fn fadeno_de_procezo(&self, proceza: ProcezaId) -> Option<FadenoId> {
        self.fadenoj.iter().flatten().find_map(|fadeno| {
            (fadeno.procezo == proceza
                && fadeno.id != RIPAZA_ID
                && fadeno.stato != FadenoStato::Mortinta)
                .then_some(fadeno.id)
        })
    }

    /// Finu la fadenon `id` kaj detruu ĝian procezon; la IK jam estis purigita
    /// de la vokanto. La voko ne revenas al `id`: la ero restas en la tabelo
    /// kun la stato `Mortinta`, do la preta vico preterlasas ĝin.
    fn marki_mortinta_kaj_detruu(&mut self, proceza: ProcezaId, id: FadenoId) {
        let _ = self.vekoj.forigi(id);
        if let Some(marko) = self.tempolimoj.get_mut(id as usize) {
            *marko = false;
        }
        if let Some(fadeno) = self.fadenoj[id as usize].as_mut() {
            fadeno.stato = FadenoStato::Mortinta;
        }
        self.stato_procezan(id, ProcezaEstado::Mortinta);
        if let Err(eraro) = super::procezo::detrui(proceza, super::kadra_alokilo()) {
            crate::skribu!("[procezo] {} ne povis esti detruita: {}\n", proceza, eraro);
        }
    }

    /// Konsumu la tempoliman markon de fadeno; `true` nur unufoje por ĉiu
    /// limdato.
    pub fn preni_tempolimon(&mut self, id: FadenoId) -> bool {
        self.tempolimoj
            .get_mut(id as usize)
            .is_some_and(core::mem::take)
    }

    pub fn morti(&mut self, id: FadenoId) -> ! {
        crate::arkitekturo::malebligi_interrompojn();
        let _ = self.vekoj.forigi(id);
        if let Some(marko) = self.tempolimoj.get_mut(id as usize) {
            *marko = false;
        }
        let proceza = self.procezo_de(id);
        if let Some(fadeno) = self.fadenoj[id as usize].as_mut() {
            fadeno.stato = FadenoStato::Mortinta;
        }
        crate::skribu!("[planisto] fadeno {} finiĝis\n", id);
        self.stato_procezan(id, ProcezaEstado::Mortinta);

        // La uzanta fadeno vivas same longe kiel sia procezo: je `SYS_FINI` aŭ
        // mortiga escepto ni liberigas la tutan adresan spacon antaŭ ol elekti
        // la sekvan fadenon. La kernaj mapoj kaj la statika kerna stako restas
        // validaj dum la ŝalto.
        if proceza != 0
            && let Err(eraro) = super::procezo::detrui(proceza, super::kadra_alokilo())
        {
            crate::skribu!("[procezo] {} ne povis esti detruita: {}\n", proceza, eraro);
        }

        let sekva = self.elekti_proksiman().unwrap_or(RIPAZA_ID);
        self.kurigu(sekva);
        unsafe { self.ŝanĝi(id, sekva) };

        loop {
            crate::arkitekturo::atendi_interrompon();
        }
    }

    /// La voko revenas, kiam la ripoza fadeno denove estas planita.
    pub fn lanĉi(&mut self) {
        if self.nuna_fadeno.is_some_and(|id| id != RIPAZA_ID) {
            return;
        }
        let Some(sekva) = self.elekti_proksiman() else {
            return;
        };
        self.kurigu(sekva);
        unsafe { self.ŝanĝi(RIPAZA_ID, sekva) };
    }

    pub fn nuna_id(&self) -> FadenoId {
        self.nuna_fadeno.unwrap_or(RIPAZA_ID)
    }

    /// La procezo de la nuna fadeno; `0` por la ripoza kaj la kernaj fadenoj.
    pub fn nuna_procezo(&self) -> ProcezaId {
        self.nuna_fadeno.map_or(0, |id| self.procezo_de(id))
    }

    pub fn raporto(&self) -> (usize, usize, usize) {
        let mut mortintaj = 0;
        let mut pretaj = 0;
        let mut blokitaj = 0;
        for fadeno in self.fadenoj.iter().flatten().filter(|f| f.id != RIPAZA_ID) {
            match fadeno.stato {
                FadenoStato::Mortinta => mortintaj += 1,
                FadenoStato::Preta | FadenoStato::Kuranta => pretaj += 1,
                FadenoStato::Blokitaj => blokitaj += 1,
            }
        }
        (mortintaj, pretaj, blokitaj)
    }

    pub fn statistiko(&self) -> PlanistaStatistiko {
        PlanistaStatistiko {
            ŝaltoj: self.ŝaltoj,
            limdataj_blokoj: self.limdataj_blokoj,
            vekigoj: self.vekigoj,
            maks_vekprokrasto: self.maks_vekprokrasto,
        }
    }

    /// Kiom da fojoj ĉiu fadeno estis elektita; indico `0` estas la ripoza.
    pub fn veturoj(&self) -> [u64; MAKSAJ_FADENOJ + 1] {
        self.veturoj
    }

    unsafe fn ŝanĝi(&mut self, malnova: FadenoId, nova: FadenoId) {
        self.ŝaltoj = self.ŝaltoj.saturating_add(1);
        let proceza = self.procezo_de(nova);
        // La fadenaj kontekstoj vivas en la kerna bildo, kiun ĉiu proceza
        // spaco kunhavas, do la ŝalto povas okazi antaŭ la registra konservo.
        // SAFETY: `ŝanĝi` estas vokata nur kun interrompoj malŝaltitaj, kaj
        // la celo estas viva fadeno kun viva proceza aŭ kerna spaco.
        unsafe { super::procezo::aktivigi(proceza) };

        let malnova =
            &mut self.fadenoj[malnova as usize].as_mut().unwrap().konteksto as *mut Kunteksto;
        let nova = &self.fadenoj[nova as usize].as_ref().unwrap().konteksto as *const Kunteksto;
        unsafe { arkitekturo::ŝalti_kuntekston(malnova, nova) };
    }
}

static PLANISTO: Kunhavigita<Planisto> = Kunhavigita::nova(Planisto::nova());

fn planisto() -> &'static mut Planisto {
    PLANISTO.akiri()
}

/// La unua libera fadena identigilo por dinamika uzanta procezo.
pub fn libera_fadena_id() -> Option<FadenoId> {
    (UNUA_DINAMIKA..=MAKSAJ_FADENOJ as FadenoId).find(|id| {
        planisto().fadenoj[*id as usize]
            .as_ref()
            .is_none_or(|fadeno| fadeno.stato == FadenoStato::Mortinta)
    })
}

/// La procezo, al kiu apartenas la fadeno `id`; `0` por la ripoza fadeno kaj
/// la kernaj fadenoj.
pub fn procezo_de_fadeno(id: FadenoId) -> ProcezaId {
    planisto().procezo_de(id)
}

pub fn pretigi_fadenon(
    id: FadenoId,
    proceza_id: ProcezaId,
    stako_pinto: VirtuaAdreso,
    enirpunkto: FadenoEnirpunkto,
) -> Fadeno {
    let mut fadeno = Fadeno::nova(id, proceza_id, stako_pinto, enirpunkto);
    arkitekturo::prepari_kuntekston(&mut fadeno.konteksto, fadeno.stako_pinto.adreso());
    // La trampolino transdonas la unuan argumenton al Rust.
    fadeno.konteksto.agordi_unuan_argumenton(id);
    fadeno
}

pub fn aldoni_fadenon(fadeno: Fadeno) -> Result<(), PlanistaEraro> {
    planisto().aldoni_fadenon(fadeno)
}

/// Antaŭokupa hoko, vokata el la temporizila traktilo (IF jam malŝaltita).
pub fn planumi() {
    planisto().planumi();
}

/// Koopera cedo; interrompoj restas malŝaltitaj dum la ŝanĝo.
pub fn cedi() {
    let _gardo = InterrompaGardo::nova();
    planisto().planumi_kun_cedo(true);
}

/// Bloku la nunan fadenon. La vokanto tenas interrompojn malŝaltitaj.
pub fn bloki() {
    let id = nuna_id();
    if id != RIPAZA_ID {
        planisto().bloki(id);
    }
}

/// Bloku la nunan fadenon ĝis la absoluta temporizila tiko `limdato`. La
/// vokanto kalkulas la limdaton unufoje por la tuta voko, do ripeta blokado
/// post fremda veko ne plilongigas la atendadon. La vokanto tenas interrompojn
/// malŝaltitaj.
pub fn bloki_ĝis_absoluta(limdato: u64) {
    let id = nuna_id();
    if id != RIPAZA_ID {
        planisto().bloki_kun_limdato(id, limdato);
    }
}

/// Bloku la nunan fadenon almenaŭ `millisekundoj`. `false` se la vokanto ne
/// estas planebla fadeno (la ripoza kunteksto); `0` revenas tuj sen bloki.
pub fn dormi(millisekundoj: u64) -> bool {
    if millisekundoj == 0 {
        return true;
    }
    let _gardo = InterrompaGardo::nova();
    let id = nuna_id();
    if id == RIPAZA_ID {
        return false;
    }
    let tikoj = millisekundoj.div_ceil(10).max(1);
    let limdato = crate::arkitekturo::temporizilaj_tikoj().saturating_add(tikoj);
    loop {
        if crate::arkitekturo::temporizilaj_tikoj() >= limdato {
            break;
        }
        planisto().bloki_kun_limdato(id, limdato);
        // Fremda veko aŭ tempolimo lasas markon; ĝin konsumas ĉiu ripeto, do
        // la sekva IK-voko ne konfuzos ĝin kun propra limdato.
        let _ = planisto().preni_tempolimon(id);
    }
    true
}

/// Ĉu la antaŭa blokado finiĝis pro tempolimo? La marko konsumiĝas, do ĉiu
/// limdato raportiĝas unufoje. Vokenda kun interrompoj malŝaltitaj.
pub fn preni_tempolimon() -> bool {
    let id = nuna_id();
    id != RIPAZA_ID && planisto().preni_tempolimon(id)
}

pub fn veki(id: FadenoId) {
    planisto().veki(id);
}

/// Finu la fadenon de la procezo `proceza` kaj detruu la procezon. Redonas
/// `true` se tia vivanta fadeno troviĝis kaj la voko sukcesis; `false`, se la
/// procezo ne havas vivantan fadenon aŭ ĝi estas la nuna fadeno.
///
/// La IK-purigo okazas antaŭ la planista ŝanĝo, ĉar la revokaj vekoj bezonas
/// la pretan vicon. `super::procezo::detrui` mem presas la akceptan linion
/// `[procezo] N detruita: …`.
pub fn mortigi_procezon(proceza: ProcezaId) -> bool {
    let _gardo = InterrompaGardo::nova();
    if proceza == 0 || proceza as usize >= super::procezo::MAKSAJ_PROCEZOJ {
        return false;
    }
    let id = planisto().fadeno_de_procezo(proceza);
    let Some(id) = id else {
        return false;
    };
    if id == nuna_id() {
        return false;
    }
    super::ik::purigi_procezon(proceza);
    planisto().marki_mortinta_kaj_detruu(proceza, id);
    true
}

/// Ĉu la identigilo apartenas al registrita, ankoraŭ viva fadeno? La ripoza
/// kunteksto (`0`) ne estas mesaĝcelo. Vokata de IK kun interrompoj malŝaltitaj.
pub fn vivas(id: FadenoId) -> bool {
    if id == RIPAZA_ID || id as usize > MAKSAJ_FADENOJ {
        return false;
    }
    planisto().fadenoj[id as usize].is_some_and(|fadeno| fadeno.stato != FadenoStato::Mortinta)
}

/// Finu la nunan fadenon; la voko neniam revenas.
///
/// Vokata de `SYS_FINI` kaj de la escepta dispetĉo por mortiginda uzanta
/// fadeno. Se la fadeno apartenas al procezo, ĝia tuta adresa spaco estas
/// detruita. La ripoza kunteksto ne havas fadenan vivciklon, do voko de tie
/// haltigas la sistemon anstataŭ ŝajnigi sukceson.
pub fn morti() -> ! {
    let id = nuna_id();
    if id == RIPAZA_ID {
        crate::skribu!("[planisto] FINI sen fadeno; haltas\n");
        crate::arkitekturo::halti()
    }
    // La IK-purigo bezonas la pretan vicon (revokaj vekoj), do ĝi okazas
    // antaŭ ol `morti` prunteprenas la planiston.
    let proceza = {
        let planisto = planisto();
        planisto.procezo_de(id)
    };
    if proceza != 0 {
        super::ik::purigi_procezon(proceza);
    }
    planisto().morti(id)
}

pub fn nuna_id() -> FadenoId {
    planisto().nuna_id()
}

/// La procezo de la nuna fadeno; `0` por la nukleo.
pub fn nuna_procezo() -> ProcezaId {
    planisto().nuna_procezo()
}

pub fn lanĉi() {
    arkitekturo::malebligi_interrompojn();
    planisto().lanĉi();
}

pub fn raporto() -> (usize, usize, usize) {
    planisto().raporto()
}

pub fn statistiko() -> PlanistaStatistiko {
    planisto().statistiko()
}

pub fn veturoj() -> [u64; MAKSAJ_FADENOJ + 1] {
    planisto().veturoj()
}

pub fn plenumtikoj() -> [u64; MAKSAJ_FADENOJ + 1] {
    let _gardo = InterrompaGardo::nova();
    planisto().plenumtikoj
}

/// Unua Rust-funkcio de nova fadeno, vokata de la muntita trampolino.
#[unsafe(no_mangle)]
pub extern "C" fn fadeno_eniro(id: FadenoId) -> ! {
    let enirpunkto = planisto().fadenoj[id as usize]
        .as_ref()
        .and_then(|fadeno| fadeno.enirpunkto);
    if let Some(enirpunkto) = enirpunkto {
        enirpunkto(id);
    }
    planisto().morti(id)
}

fn testa_planisto<const N: usize>(prioritatoj: [Prioritato; N]) -> Planisto {
    let mut planisto = Planisto::nova();
    for (indekso, prioritato) in prioritatoj.into_iter().enumerate() {
        let id = indekso as u64 + 1;
        let mut fadeno = Fadeno::nova(id, 0, VirtuaAdreso::nova(0), |_| {});
        fadeno.prioritato = prioritato;
        planisto.aldoni_fadenon(fadeno).unwrap();
    }
    let unua = planisto.elekti_proksiman().unwrap();
    planisto.kurigu(unua);
    planisto
}

fn prioritata_provo() -> bool {
    let mut planisto = testa_planisto([Prioritato::Malalta, Prioritato::Normala, Prioritato::Alta]);
    let mut tikoj = [0u8; 3];
    for _ in 0..14 {
        tikoj[planisto.nuna_id() as usize - 1] += 1;
        if let Some(sekva) = planisto.elekti_por_tiko(false) {
            planisto.kurigu(sekva);
        }
    }
    tikoj == [2, 4, 8] && planisto.nuna_id() == 1 && planisto.restaj_tikoj == 1
}

fn maksimuma_atenda_provo() -> bool {
    let mut prioritatoj = [Prioritato::Alta; MAKSAJ_FADENOJ];
    prioritatoj[0] = Prioritato::Malalta;
    let mut planisto = testa_planisto(prioritatoj);
    let sekva = planisto.elekti_por_tiko(false).unwrap();
    planisto.kurigu(sekva);
    for _ in 0..MAKSA_PRETA_ATENDO {
        if planisto.nuna_id() == 1 {
            return false;
        }
        if let Some(sekva) = planisto.elekti_por_tiko(false) {
            planisto.kurigu(sekva);
        }
    }
    planisto.nuna_id() == 1
}

fn egaleca_provo() -> bool {
    let mut planisto = testa_planisto([Prioritato::Normala; 3]);
    for tiko in 0..12 {
        if planisto.nuna_id() != (tiko / 2 % 3 + 1) as u64 {
            return false;
        }
        if let Some(sekva) = planisto.elekti_por_tiko(false) {
            planisto.kurigu(sekva);
        }
    }
    planisto.nuna_id() == 1
}

fn ceda_provo() -> bool {
    let mut planisto = testa_planisto([Prioritato::Alta; 3]);
    for id in [2, 3, 1] {
        if planisto.elekti_por_tiko(true) != Some(id) {
            return false;
        }
        planisto.kurigu(id);
    }
    planisto.restaj_tikoj == 4
}

fn bloka_provo() -> bool {
    let mut planisto = testa_planisto([Prioritato::Alta; 3]);
    planisto.fadenoj[1].as_mut().unwrap().stato = FadenoStato::Blokitaj;
    let sekva = planisto.elekti_proksiman().unwrap();
    planisto.kurigu(sekva);
    planisto.veki_sen_limdato(1);
    planisto.veki_sen_limdato(1);
    for id in [3, 1] {
        if planisto.elekti_por_tiko(true) != Some(id) {
            return false;
        }
        planisto.kurigu(id);
    }
    planisto.fadenoj[1].as_mut().unwrap().stato = FadenoStato::Blokitaj;
    planisto.fadenoj[2].as_mut().unwrap().stato = FadenoStato::Blokitaj;
    planisto.fadenoj[3].as_mut().unwrap().stato = FadenoStato::Mortinta;
    if planisto.elekti_proksiman().is_some() {
        return false;
    }
    planisto.kurigu(RIPAZA_ID);
    if planisto.elekti_por_tiko(false).is_some() {
        return false;
    }
    planisto.veki_sen_limdato(1);
    planisto.veki_sen_limdato(1);
    if planisto.elekti_por_tiko(false) != Some(1) {
        return false;
    }
    planisto.kurigu(1);
    planisto.restaj_tikoj == 4 && planisto.preta_vico.pop().is_none()
}

pub fn testi_prioritatojn() {
    for (nomo, bone) in [
        ("prioritata rondo 1:2:4", prioritata_provo()),
        ("maksimuma preta atendo 28 tikoj", maksimuma_atenda_provo()),
        ("egalprioritata ordo", egaleca_provo()),
        ("tuja cedo", ceda_provo()),
        ("blokado, ripozo kaj veko", bloka_provo()),
    ] {
        crate::skribu!(
            "[planisto] {} ({}):\n",
            nomo,
            if bone { "bone" } else { "MALBONE" }
        );
    }
}

/// Tuta-tabela egaleco malvolviĝas al bajta `memcmp` kun pli da buklaj paŝoj
/// ol niaj Kani-limoj; jenaj naŭ skalaraj komparoj evitas tiun kompilan buklon
/// sen malŝalti iun ajn kontrolon.
fn samaj_plenumtikoj(
    mezuroj: &[u64; MAKSAJ_FADENOJ + 1],
    atenditaj: &[u64; MAKSAJ_FADENOJ + 1],
) -> bool {
    mezuroj[0] == atenditaj[0]
        && mezuroj[1] == atenditaj[1]
        && mezuroj[2] == atenditaj[2]
        && mezuroj[3] == atenditaj[3]
        && mezuroj[4] == atenditaj[4]
        && mezuroj[5] == atenditaj[5]
        && mezuroj[6] == atenditaj[6]
        && mezuroj[7] == atenditaj[7]
        && mezuroj[8] == atenditaj[8]
}

fn plenumtempa_provo() -> bool {
    let mut planisto = testa_planisto([Prioritato::Malalta, Prioritato::Normala, Prioritato::Alta]);
    for _ in 0..14 {
        if let Some(sekva) = planisto.elekti_por_tiko(false) {
            planisto.kurigu(sekva);
        }
    }
    samaj_plenumtikoj(&planisto.plenumtikoj, &[0, 2, 4, 8, 0, 0, 0, 0, 0])
}

fn ceda_kontada_provo() -> bool {
    let mut planisto = testa_planisto([Prioritato::Alta; 3]);
    planisto.elekti_por_tiko(false);
    for _ in 0..6 {
        let sekva = planisto.elekti_por_tiko(true).unwrap();
        planisto.kurigu(sekva);
    }
    samaj_plenumtikoj(&planisto.plenumtikoj, &[0, 1, 0, 0, 0, 0, 0, 0, 0])
        && planisto.veturoj[1] == 3
}

fn ripoza_kontada_provo() -> bool {
    let mut planisto = Planisto::nova();
    planisto.elekti_por_tiko(false);
    if planisto.plenumtikoj[0] != 1 {
        return false;
    }
    let fadeno = Fadeno::nova(1, 0, VirtuaAdreso::nova(0), |_| {});
    planisto.aldoni_fadenon(fadeno).unwrap();
    planisto.kurigu(RIPAZA_ID);
    let sekva = planisto.elekti_por_tiko(false).unwrap();
    planisto.kurigu(sekva);
    planisto.elekti_por_tiko(false);
    planisto.fadenoj[1].as_mut().unwrap().stato = FadenoStato::Blokitaj;
    planisto.kurigu(RIPAZA_ID);
    planisto.elekti_por_tiko(false);
    planisto.veki_sen_limdato(1);
    let sekva = planisto.elekti_por_tiko(false).unwrap();
    planisto.kurigu(sekva);
    planisto.elekti_por_tiko(false);
    planisto.fadenoj[1].as_mut().unwrap().stato = FadenoStato::Mortinta;
    planisto.kurigu(RIPAZA_ID);
    planisto.elekti_por_tiko(false);
    samaj_plenumtikoj(&planisto.plenumtikoj, &[5, 2, 0, 0, 0, 0, 0, 0, 0])
}

pub fn testi_plenumtempon() {
    for (nomo, bone) in [
        ("plenumtikoj laŭ tempopecoj", plenumtempa_provo()),
        ("cedo ne aldonas plenumtikojn", ceda_kontada_provo()),
        (
            "ripozo, blokado kaj morto konservas kontadon",
            ripoza_kontada_provo(),
        ),
    ] {
        crate::skribu!(
            "[planisto] {} ({})\n",
            nomo,
            if bone { "bone" } else { "MALBONE" }
        );
    }
}

#[cfg(kani)]
mod kani_plenumtempo {
    use super::*;

    #[kani::proof]
    #[kani::unwind(16)]
    fn tempopecoj_konservas_plenumtikojn() {
        assert!(plenumtempa_provo());
    }

    #[kani::proof]
    #[kani::unwind(12)]
    fn cedo_ne_kreas_tempon() {
        assert!(ceda_kontada_provo());
    }

    #[kani::proof]
    #[kani::unwind(12)]
    fn ripozo_kaj_blokado_apartigas_tempon() {
        assert!(ripoza_kontada_provo());
    }

    #[kani::proof]
    #[kani::unwind(12)]
    fn nur_la_nuna_nombrigilo_ŝanĝiĝas_kaj_saturiĝas() {
        let mut planisto = Planisto::nova();
        let id = kani::any::<u8>() as usize % (MAKSAJ_FADENOJ + 1);
        let antaŭ: [u64; MAKSAJ_FADENOJ + 1] = kani::any();
        planisto.plenumtikoj = antaŭ;
        if id != RIPAZA_ID as usize {
            planisto.fadenoj[id] = Some(Fadeno::nova(id as u64, 0, VirtuaAdreso::nova(0), |_| {}));
        }
        planisto.kurigu(id as u64);
        planisto.elekti_por_tiko(false);
        for (indekso, valoro) in antaŭ.into_iter().enumerate() {
            let atendita = if indekso == id {
                valoro.saturating_add(1)
            } else {
                valoro
            };
            assert_eq!(planisto.plenumtikoj[indekso], atendita);
        }
    }
}

#[cfg(kani)]
mod kani_prioritatoj {
    use super::*;

    #[kani::proof]
    #[kani::unwind(16)]
    fn pezaj_tempopecoj_sen_malsatigo() {
        assert!(prioritata_provo());
    }

    #[kani::proof]
    #[kani::unwind(32)]
    fn malalta_prioritato_atendas_maksimume_unu_rondon() {
        assert!(maksimuma_atenda_provo());
    }

    #[kani::proof]
    #[kani::unwind(14)]
    fn egalprioritataj_rotacias() {
        assert!(egaleca_provo());
    }

    #[kani::proof]
    #[kani::unwind(12)]
    fn cedo_liberigas_tempopecon() {
        assert!(ceda_provo());
    }

    #[kani::proof]
    #[kani::unwind(12)]
    fn blokitaj_ne_kuras_kaj_veko_ne_duobligas() {
        assert!(bloka_provo());
    }
}
