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
use crate::memoro::virtuala::{AdresaSpaco, UZANTA_STAKO_SUPRO};

use super::procezo::{self, ProcezaEraro};
use super::uzantspaco::{self, UzantaBildo, ŜargaEraro};

pub enum ExecEraro {
    NeniuProcezo,
    TroMalgranda,
    Elĉerpita,
    Dosiersistemo(snds::Eraro),
    Ŝargo(ŜargaEraro),
    Proceza(ProcezaEraro),
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
