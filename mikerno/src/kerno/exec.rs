//! `SYS_EXEC`: anstataŭigo de la nuna procezo per ELF-bildo el SNDS.
//!
//! La dosiero estas legata en kontiguajn kadrojn, validigata de la sama
//! ŝargilo kiel la praŝarĝa `horizonto` kaj muntata en freŝan adresan spacon.
//! Post la ŝalto la malnova spaco (bildo, stako, kupo kaj sekretaj paĝoj) estas
//! detruata, do `exec` anstataŭigas la procezon (C17.5), ne kreas novan.

use core::fmt;
use core::slice;

use libmikerno::starto::REĜIMO_EXEC;

use crate::konservado::snds;
use crate::memoro::KADRA_GRANDO;
use crate::memoro::kadro::Kadro;
use crate::memoro::virtuala::{AdresaSpaco, UZANTA_STAKO_SUPRO, VirtuaAdreso};

use super::procezo::{self, ProcezaEraro};
use super::uzantspaco::{self, UzantaBildo, ŜargaEraro};

pub enum ExecEraro {
    NeniuProcezo,
    TroMalgranda,
    Elĉerpita,
    Dosiersistemo(snds::Eraro),
    Ŝargo(ŜargaEraro),
    Proceza(ProcezaEraro),
    Registro,
}

impl fmt::Display for ExecEraro {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NeniuProcezo => f.write_str("la nuna fadeno ne apartenas al procezo"),
            Self::TroMalgranda => f.write_str("la dosiero estas pli mallonga ol ELF-kapo"),
            Self::Elĉerpita => f.write_str("neniu kontigua kadro por la fonta bildo"),
            Self::Dosiersistemo(eraro) => write!(f, "SNDS: {eraro}"),
            Self::Ŝargo(eraro) => write!(f, "bilda ŝargo: {eraro}"),
            Self::Proceza(eraro) => write!(f, "proceza operacio: {eraro}"),
            Self::Registro => f.write_str("la planista ero ne registriĝis"),
        }
    }
}

impl From<ProcezaEraro> for ExecEraro {
    fn from(eraro: ProcezaEraro) -> Self {
        Self::Proceza(eraro)
    }
}

/// Anstataŭigu la procezon `id` per la bildo `vojo` kaj eniru ĝin je Ring 3.
///
/// Sukceso neniam revenas. La funkcio redonas nur la kaŭzon de malsukceso;
/// tiam la malnova procezo restas muntita kaj la alvokanto decidas la
/// erar-kodon.
///
/// # Sekureco
/// Vokenda en Ring 0 kun interrompoj malŝaltitaj, dum `id` estas la nuna
/// procezo kaj `vojo` estas valida UTF-8-sen-NUL-a teksto.
pub fn anstataŭigi(id: u64, vojo: &str) -> ExecEraro {
    let dosiero = match snds::malfermi(vojo) {
        Ok(dosiero) => dosiero,
        Err(eraro) => return ExecEraro::Dosiersistemo(eraro),
    };
    let grando = dosiero.grando();
    if grando < 64 {
        return ExecEraro::TroMalgranda;
    }
    let kadroj = (grando as usize).div_ceil(KADRA_GRANDO as usize);
    let fonto = {
        let alokilo = super::kadra_alokilo();
        match alokilo.aloki_kontiguajn(kadroj) {
            Some(kadro) => kadro,
            None => return ExecEraro::Elĉerpita,
        }
    };

    let rezulto = legi_kaj_munti(id, fonto, grando, &dosiero);
    liberigi_fonton(fonto, kadroj);

    let bildo = match rezulto {
        Ok(bildo) => bildo,
        Err(eraro) => return eraro,
    };
    crate::skribu!(
        "[kerno] exec: procezo {} anstataŭigita per {} ({} bajtoj, enirpunkto 0x{:x})\n",
        id,
        vojo,
        grando,
        bildo.enirpunkto
    );
    // SAFETY: la nova spaco kunhavas la kernajn mapojn (ĝin kreis
    // `krei_kunhavantan`), la nuna fadeno ankoraŭ plenumas sur la kerna stako
    // de `id`, kaj la bildo estis muntita kun la bito `U`. La interrompoj estas
    // malŝaltitaj en la sistemvoka vojo.
    unsafe { procezo::aktivigi(id) };
    // SAFETY: `munti_el_bufro` mapis la enirpunktan paĝon plenumebla kaj la
    // stakon skribebla; `id` estas la identigilo de la sama procezo.
    unsafe {
        crate::arkitekturo::eniri_uzanton(bildo.enirpunkto, UZANTA_STAKO_SUPRO, REĜIMO_EXEC, id)
    }
}

/// Kreu novan procezon el la ELF-bildo `vojo` kaj registru ĝin ĉe la plano.
///
/// La nova procezo heredas la kernajn mapojn de la nuna spaco, sed ricevas
/// propran uzantan fenestron, stakon kaj sekretan paĝon. Ĝi startas en
/// `REĜIMO_EXEC`, do la bildo raportas la sukceson anstataŭe ripeti la testojn.
/// Redonas la novan procezan identigilon.
///
/// # Sekureco
/// Vokenda en Ring 0 kun interrompoj malŝaltitaj; la nuna fadeno devas aparteni
/// al viva procezo (la ŝelo).
pub fn lanĉi(vojo: &str) -> Result<u64, ExecEraro> {
    let dosiero = snds::malfermi(vojo).map_err(ExecEraro::Dosiersistemo)?;
    let grando = dosiero.grando();
    if grando < 64 {
        return Err(ExecEraro::TroMalgranda);
    }
    let kadroj = (grando as usize).div_ceil(KADRA_GRANDO as usize);
    let fonto = {
        let alokilo = super::kadra_alokilo();
        match alokilo.aloki_kontiguajn(kadroj) {
            Some(kadro) => kadro,
            None => return Err(ExecEraro::Elĉerpita),
        }
    };
    let rezulto = munti_novan(fonto, grando, &dosiero, vojo);
    liberigi_fonton(fonto, kadroj);
    rezulto
}

fn munti_novan(
    fonto: Kadro,
    grando: u32,
    dosiero: &snds::Dosiero,
    vojo: &str,
) -> Result<u64, ExecEraro> {
    {
        // SAFETY: la fontaj kadroj estis ĵus alokitaj kontigue kaj kuŝas en la
        // identeca mapo; la bufro kovras ekzakte la dosieron.
        let bufro =
            unsafe { slice::from_raw_parts_mut(fonto.adreso() as *mut u8, grando as usize) };
        snds::legi(dosiero, bufro).map_err(ExecEraro::Dosiersistemo)?;
    }

    let mut nova = {
        let alokilo = super::kadra_alokilo();
        let nuna = super::planisto::nuna_procezo();
        let Some(procezo) = procezo::akiri(nuna) else {
            return Err(ExecEraro::NeniuProcezo);
        };
        procezo
            .spaco()
            .krei_kunhavantan(alokilo)
            .map_err(|eraro| ExecEraro::Ŝargo(ŜargaEraro::from(eraro)))?
    };

    let bildo = match uzantspaco::munti_el_bufro(
        &mut nova,
        super::kadra_alokilo(),
        fonto.adreso(),
        grando as u64,
    ) {
        Ok(bildo) => bildo,
        Err(eraro) => {
            let _ = nova.detruu(super::kadra_alokilo());
            return Err(ExecEraro::Ŝargo(eraro));
        }
    };

    let Some(id) = (1..super::procezo::MAKSAJ_PROCEZOJ as u64)
        .find(|&id| procezo::akiri(id).is_none() && !super::planisto::vivas(id))
    else {
        let _ = nova.detruu(super::kadra_alokilo());
        return Err(ExecEraro::Elĉerpita);
    };
    let id = match procezo::registri(id, nova) {
        Ok(id) => id,
        Err((eraro, spaco)) => {
            let _ = spaco.detruu(super::kadra_alokilo());
            return Err(ExecEraro::Proceza(eraro));
        }
    };

    if let Err(eraro) = preparu(id, &bildo, vojo) {
        forigu(id);
        return Err(eraro);
    }
    crate::skribu!(
        "[kerno] nova procezo {} lanĉita el SNDS: {} (enirpunkto 0x{:x})\n",
        id,
        vojo,
        bildo.enirpunkto
    );
    Ok(id)
}

/// Agordu la muntitan bildon, la sekretan paĝon kaj la planistan fadenon.
fn preparu(id: u64, bildo: &UzantaBildo, vojo: &str) -> Result<(), ExecEraro> {
    let kerna_pinto = {
        let Some(procezo) = procezo::akiri(id) else {
            return Err(ExecEraro::NeniuProcezo);
        };
        procezo.agordi_bildon(bildo.komenco, bildo.fino, bildo.enirpunkto);
        procezo.agordi_nomon(vojo.rsplit('/').next().unwrap_or(vojo));
        procezo.agordi_reĝimon(REĜIMO_EXEC);
        match uzantspaco::mapi_sekreton(procezo.spaco(), super::kadra_alokilo(), id, false) {
            Ok(adreso) => procezo.agordi_sekretan(adreso),
            Err(eraro) => return Err(ExecEraro::Ŝargo(eraro)),
        }
        procezo.kerna_stako_pinto()
    };

    let Some(fadeno_id) = super::planisto::libera_fadena_id() else {
        return Err(ExecEraro::Registro);
    };
    if super::ik::registri_fadenon(fadeno_id, id).is_err() {
        return Err(ExecEraro::Registro);
    }
    let fadeno = super::planisto::pretigi_fadenon(
        fadeno_id,
        id,
        VirtuaAdreso::nova(kerna_pinto),
        super::uzanta_fadeno,
    );
    if super::planisto::aldoni_fadenon(fadeno).is_err() {
        return Err(ExecEraro::Registro);
    }
    Ok(())
}

fn forigu(id: u64) {
    super::ik::purigi_procezon(id);
    let _ = procezo::detrui(id, super::kadra_alokilo());
}

fn legi_kaj_munti(
    id: u64,
    fonto: Kadro,
    grando: u32,
    dosiero: &snds::Dosiero,
) -> Result<UzantaBildo, ExecEraro> {
    {
        // SAFETY: la fontaj kadroj estis ĵus alokitaj kontigue kaj kuŝas en la
        // identeca mapo de la unuaj 4 GiB; la bufro kovras ekzakte la dosieron.
        let bufro =
            unsafe { slice::from_raw_parts_mut(fonto.adreso() as *mut u8, grando as usize) };
        snds::legi(dosiero, bufro).map_err(ExecEraro::Dosiersistemo)?;
    }

    let mut nova = {
        let alokilo = super::kadra_alokilo();
        let Some(procezo) = procezo::akiri(id) else {
            return Err(ExecEraro::NeniuProcezo);
        };
        procezo
            .spaco()
            .krei_kunhavantan(alokilo)
            .map_err(|eraro| ExecEraro::Ŝargo(ŜargaEraro::from(eraro)))?
    };

    let muntado = uzantspaco::munti_el_bufro(
        &mut nova,
        super::kadra_alokilo(),
        fonto.adreso(),
        grando as u64,
    );
    let bildo = match muntado {
        Ok(bildo) => bildo,
        Err(eraro) => {
            // La duonmuntita spaco ne rajtas resti en la kadro-alokilo.
            let _ = nova.detruu(super::kadra_alokilo());
            return Err(ExecEraro::Ŝargo(eraro));
        }
    };

    let Some(procezo) = procezo::akiri(id) else {
        let _ = nova.detruu(super::kadra_alokilo());
        return Err(ExecEraro::NeniuProcezo);
    };
    procezo.agordi_bildon(bildo.komenco, bildo.fino, bildo.enirpunkto);
    procezo.anstataŭigi_spacon(nova, super::kadra_alokilo())?;
    Ok(bildo)
}

fn liberigi_fonton(fonto: Kadro, kadroj: usize) {
    let alokilo = super::kadra_alokilo();
    for indekso in 0..kadroj {
        alokilo.liberigi(Kadro::el_adreso(
            fonto.adreso() + indekso as u64 * KADRA_GRANDO,
        ));
    }
}
