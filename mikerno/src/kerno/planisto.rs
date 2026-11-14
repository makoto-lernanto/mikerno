//! Unuprocesora antaŭokupa planisto kun cirkla preta vico.
//!
//! Ĉiuj modifoj okazas kun interrompoj malŝaltitaj. La temporizila traktilo
//! vokas `planumi`; fadeno povas cedi per `cedi`, blokiĝi per `bloki` aŭ
//! finiĝi per `morti`. La ripoza fadeno (id 0) portas la praŝarĝan kuntekston.

use crate::arkitekturo::{self, Kunteksto};
use crate::interrompo::InterrompaGardo;
use crate::kunhavigita::Kunhavigita;
use crate::memoro::virtuala::VirtuaAdreso;

use super::objekto::{Fadeno, FadenoEnirpunkto, FadenoId, FadenoStato};
use super::procezo::{ProcezaEstado, ProcezaId, marki};

pub const MAKSAJ_FADENOJ: usize = 8;
const RIPAZA_ID: FadenoId = 0;

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

    fn pusxi(&mut self, id: FadenoId) {
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
}

impl Planisto {
    pub const fn nova() -> Self {
        let mut fadenoj = [None; MAKSAJ_FADENOJ + 1];
        fadenoj[RIPAZA_ID as usize] = Some(Fadeno::ripoza());
        Self {
            fadenoj,
            preta_vico: PretaVico::nova(),
            nuna_fadeno: None,
        }
    }

    pub fn aldoni_fadenon(&mut self, fadeno: Fadeno) -> Result<(), PlanistaEraro> {
        let indekso = fadeno.id as usize;
        if fadeno.id == RIPAZA_ID || indekso >= self.fadenoj.len() {
            return Err(PlanistaEraro::NevalidaIdentigilo);
        }
        if self.fadenoj[indekso].is_some() {
            return Err(PlanistaEraro::JamRegistrita);
        }
        self.preta_vico.pusxi(fadeno.id);
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
        }
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

    /// Vokata el la temporizila interrompo: rotaciu la nunan fadenon aŭ,
    /// se la ripoza kunteksto regas, planu vekitajn fadenojn.
    pub fn planumi(&mut self) {
        let Some(nuna) = self.nuna_fadeno else {
            return;
        };
        if nuna == RIPAZA_ID {
            if let Some(sekva) = self.elekti_proksiman() {
                self.kurigu(sekva);
                unsafe { self.switch(RIPAZA_ID, sekva) };
            }
            return;
        }
        let Some(fadeno) = self.fadenoj[nuna as usize].as_mut() else {
            return;
        };
        if fadeno.stato != FadenoStato::Kuranta {
            return;
        }

        fadeno.stato = FadenoStato::Preta;
        self.preta_vico.pusxi(nuna);
        self.stato_procezan(nuna, ProcezaEstado::Preta);

        let sekva = self.elekti_proksiman().unwrap_or(nuna);
        if sekva == nuna {
            self.kurigu(nuna);
            return;
        }
        self.kurigu(sekva);
        unsafe { self.switch(nuna, sekva) };
    }

    pub fn bloki(&mut self, id: FadenoId) {
        if let Some(fadeno) = self.fadenoj[id as usize].as_mut() {
            fadeno.stato = FadenoStato::Blokitaj;
        }
        // Procezo sen kuranta fadeno ne plu estas "kuranta".
        self.stato_procezan(id, ProcezaEstado::Preta);
        let sekva = self.elekti_proksiman().unwrap_or(RIPAZA_ID);
        self.kurigu(sekva);
        unsafe { self.switch(id, sekva) };
    }

    pub fn veki(&mut self, id: FadenoId) {
        let Some(fadeno) = self.fadenoj[id as usize].as_mut() else {
            return;
        };
        if fadeno.stato != FadenoStato::Blokitaj {
            return;
        }
        fadeno.stato = FadenoStato::Preta;
        self.preta_vico.pusxi(id);
    }

    pub fn morti(&mut self, id: FadenoId) -> ! {
        crate::arkitekturo::malebligi_interrompojn();
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
        unsafe { self.switch(id, sekva) };

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
        unsafe { self.switch(RIPAZA_ID, sekva) };
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

    unsafe fn switch(&mut self, malnova: FadenoId, nova: FadenoId) {
        let proceza = self.procezo_de(nova);
        // La fadenaj kontekstoj vivas en la kerna bildo, kiun ĉiu proceza
        // spaco kunhavas, do la ŝalto povas okazi antaŭ la registra konservo.
        // SAFETY: `switch` estas vokata nur kun interrompoj malŝaltitaj, kaj
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
    planisto().planumi();
}

/// Bloku la nunan fadenon. La vokanto tenas interrompojn malŝaltitaj.
pub fn bloki() {
    let id = nuna_id();
    if id != RIPAZA_ID {
        planisto().bloki(id);
    }
}

pub fn veki(id: FadenoId) {
    planisto().veki(id);
}

/// Ĉu la identigilo apartenas al registrita, ankoraŭ viva fadeno? La ripoza
/// kunteksto (`0`) ne estas mesaĝcelo. Vokata de IPC kun interrompoj malŝaltitaj.
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
    // La IPC-purigo bezonas la pretan vicon (revokaj vekoj), do ĝi okazas
    // antaŭ ol `morti` prunteprenas la planiston.
    let proceza = {
        let planisto = planisto();
        planisto.procezo_de(id)
    };
    if proceza != 0 {
        super::ipc::purigi_procezon(proceza);
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
