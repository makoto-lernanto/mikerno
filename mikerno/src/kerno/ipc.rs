//! Minimuma mesaĝa IPC: finpunktaj objektoj, kapabloj, teniloj.
//!
//! Ekde Fazo 2 de M14 la mesaĝvico apartenas al la finpunkto, ne al la fadeno.
//! Ĉiu fadeno havas defaŭltan finpunkton (kreitan ĉe ĝia registrado), kaj la
//! malnova vojo `sendi_mesagon(fadeno, …)` / `ricevi_mesagon(fonto)` uzas ĝin
//! ĝis Fazo 3 transdonos veran `Tenilo`-n en `rdi`.
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
pub use libmikerno::{FinaPunkto, Kapablo, MAKSA_MESAGA_LONGO, Mesago, Permesoj};

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

const MAKSAJ_MESAGOJ: usize = 16;
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
    TabeloPlena,
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

    fn pusxi(&mut self, ero: Ero) -> Result<(), IpcEraro> {
        if self.nombro >= MAKSAJ_MESAGOJ {
            return Err(IpcEraro::MesagujoPlena);
        }
        self.eroj[self.nombro] = Some(ero);
        self.nombro += 1;
        Ok(())
    }

    fn preni_de(&mut self, fonto: FadenoId) -> Option<Mesago> {
        let indekso = (0..self.nombro).find(|&i| self.eroj[i].is_some_and(|e| e.fonto == fonto))?;
        let ero = self.eroj[indekso].take()?;
        for movo in indekso..self.nombro - 1 {
            self.eroj[movo] = self.eroj[movo + 1];
        }
        self.nombro -= 1;
        self.eroj[self.nombro] = None;
        Some(ero.mesago)
    }
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

/// Kapabla fendo de unu procezo: kiu finpunkto kaj kun kiuj permesoj.
#[derive(Clone, Copy)]
struct KapabloEniro {
    tenilo: Tenilo,
    /// Legata ekde la permes-kontroloj de Fazo 3.
    #[allow(dead_code)]
    permesoj: Permesoj,
    finpunkto: u8,
    uzata: bool,
}

impl KapabloEniro {
    const fn libera() -> Self {
        Self {
            tenilo: Tenilo::MALVALIDA,
            permesoj: Permesoj::ĈIUJ,
            finpunkto: 0,
            uzata: false,
        }
    }
}

#[derive(Clone, Copy)]
struct Kapablotabelo {
    eroj: [KapabloEniro; MAKSAJ_KAPABLOJ],
}

impl Kapablotabelo {
    const fn nova() -> Self {
        Self {
            eroj: [KapabloEniro::libera(); MAKSAJ_KAPABLOJ],
        }
    }

    fn unua_libera(&self) -> Option<usize> {
        self.eroj.iter().position(|eniro| !eniro.uzata)
    }
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
    let Some(tabelo) = kapabloj(procezo) else {
        finpunktoj()[fendo as usize].liberigu();
        return Err(IpcEraro::TabeloPlena);
    };
    if tabelo.eroj[0].uzata {
        finpunktoj()[fendo as usize].liberigu();
        return Err(IpcEraro::TabeloPlena);
    }
    let Some(tenilo) = Tenilo::nova(0, finpunktoj()[fendo as usize].generacio) else {
        finpunktoj()[fendo as usize].liberigu();
        return Err(IpcEraro::TabeloPlena);
    };
    tabelo.eroj[0] = KapabloEniro {
        tenilo,
        permesoj: Permesoj::PLENAJ,
        finpunkto: fendo,
        uzata: true,
    };
    finpunktoj()[fendo as usize].referencoj += 1;
    Ok(())
}

/// Sendu mesaĝon al la defaŭlta finpunkto de fadeno; la fonto estas la nuna
/// fadeno.
///
/// La mesaĝo estas kopiata, kaj `sendinto`/`ricevonto` estas fiksitaj de la
/// kerno: la uzanto ne povas mensogi pri ili. `ricevonto` estas la finpunkta
/// identigilo, ne plu la celo-fadeno (C1).
///
/// # Eraroj
/// `NevalidaCelloko` se la celo ne estas viva fadeno (C16); `MesagujoPlena`
/// se la vico de la finpunkto jam enhavas `MAKSAJ_MESAGOJ` mesaĝojn;
/// `TabeloPlena` se la defaŭlta finpunkto mankas kaj ne plu estas libera fendo.
pub fn sendi_mesagon(destinacio: FadenoId, mesago: &Mesago) -> Result<(), IpcEraro> {
    let _gardo = InterrompaGardo::nova();
    if !planisto::vivas(destinacio) {
        return Err(IpcEraro::NevalidaCelloko);
    }
    let indekso = defaŭlta_indekso(destinacio).ok_or(IpcEraro::TabeloPlena)?;
    let nuna = planisto::nuna_id();
    let mut mesago = *mesago;
    mesago.sendinto = nuna;
    mesago.ricevonto = finpunktoj()[indekso].id;
    finpunktoj()[indekso].vico.pusxi(Ero {
        fonto: nuna,
        mesago,
    })?;
    planisto::veki(destinacio);
    Ok(())
}

/// Ricevu mesaĝon de la donita fonto el la defaŭlta finpunkto de la nuna
/// fadeno; bloku se mankas.
pub fn ricevi_mesagon(fonto: FadenoId) -> Mesago {
    loop {
        let _gardo = InterrompaGardo::nova();
        let nuna = planisto::nuna_id();
        if nuna == 0 {
            // La ripoza kunteksto ne havas fadenan vivciklon; ne bloku ĝin.
            return Mesago::malplena();
        }
        if let Some(indekso) = defaŭlta_indekso(nuna)
            && let Some(mesago) = finpunktoj()[indekso].vico.preni_de(fonto)
        {
            return mesago;
        }
        planisto::bloki();
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
    let Some(tenilo) = Tenilo::nova(
        fendo_kapabla as u8,
        finpunktoj()[fendo_finpunkta as usize].generacio,
    ) else {
        finpunktoj()[fendo_finpunkta as usize].liberigu();
        return Err(IpcEraro::TabeloPlena);
    };
    tabelo.eroj[fendo_kapabla] = KapabloEniro {
        tenilo,
        permesoj,
        finpunkto: fendo_finpunkta,
        uzata: true,
    };
    Ok(tenilo)
}

/// Fermu unu lokan kapablofendon. La finpunkto restas viva se aliaj referencoj
/// ekzistas; ĝi nur malaperas kiam la lasta tenilo fermiĝas (`FERMI` neniam
/// revokas, C14).
pub fn fermi(procezo: ProcezaId, tenilo: Tenilo) -> Result<(), IpcEraro> {
    let _gardo = InterrompaGardo::nova();
    let tabelo = kapabloj(procezo).ok_or(IpcEraro::NevalidaTenilo)?;
    let eniro = &mut tabelo.eroj[tenilo.indekso() as usize];
    if !eniro.uzata || eniro.tenilo != tenilo {
        return Err(IpcEraro::NevalidaTenilo);
    }
    let fendo = eniro.finpunkto as usize;
    *eniro = KapabloEniro::libera();
    let objekto = &mut finpunktoj()[fendo];
    if !objekto.aktiva() || objekto.generacio != tenilo.generacio() {
        return Err(IpcEraro::NevalidaTenilo);
    }
    objekto.referencoj = forprenu_referencon(objekto.referencoj);
    if objekto.referencoj == 0 {
        objekto.liberigu();
    }
    Ok(())
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

#[cfg(kani)]
mod kani_proofs {
    //! Kani-pruvoj por la mesaĝujo.
    //!
    //! `sendi_mesagon` kaj `ricevi_mesagon` dependas de la planisto, de
    //! interrompoj kaj de tutmonda stato, kiuj ne estas modeligeblaj per Kani
    //! sen eliri la no_std-medion. La pruvoj do celas la enan `Mesagujo`-kodon,
    //! kiun ambaŭ funkcioj rekte uzas.
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

        let ricevita = ujo.preni_de(fonto).unwrap();
        assert_eq!(ricevita.tipo, 1);
        assert_eq!(ricevita.longo(), unua.longo());

        let ricevita = ujo.preni_de(fonto).unwrap();
        assert_eq!(ricevita.tipo, 2);
        assert_eq!(ricevita.longo(), dua.longo());

        assert_eq!(ujo.nombro, 0);
        assert!(ujo.preni_de(fonto).is_none());
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

        assert_eq!(ujo.preni_de(fonto_a).unwrap().tipo, 1);
        assert_eq!(ujo.preni_de(fonto_b).unwrap().tipo, 2);
        assert_eq!(ujo.preni_de(fonto_a).unwrap().tipo, 3);
        assert!(ujo.preni_de(fonto_a).is_none());
        assert!(ujo.preni_de(fonto_b).is_none());
        assert_eq!(ujo.nombro, 0);
    }

    /// Malplena mesaĝujo redonas `None` sen subtrahi de nulo.
    #[kani::proof]
    fn malplena_mesagujo_ricevas_nenion() {
        let mut ujo = Mesagujo::nova();
        let fonto: FadenoId = kani::any();

        assert!(ujo.preni_de(fonto).is_none());
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

        let _ = ujo.preni_de(fonto);
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

    use super::{
        FinaPunktaStato, MAKSAJ_FINPUNKTOJ, elekti_liberan, forprenu_referencon, pliigita_generacio,
    };

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
