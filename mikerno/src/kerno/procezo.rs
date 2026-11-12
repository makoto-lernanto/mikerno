//! Proceza abstrakto: identigilo, privata adresa spaco, kerna stako kaj stato.
//!
//! La nukleo mem estas la procezo `0`; ĝi ne havas eron en la tabelo. Ĉiu
//! procezo posedas propran radikan paĝtabelon, kiu kunhavas la kernajn mapojn
//! sed havas privatan, malplenan uzantan fenestron.

use core::fmt;

use crate::arkitekturo::Spaco;
use crate::kunhavigita::Kunhavigita;
use crate::memoro::kadro::KadraAlokilo;
use crate::memoro::virtuala::{AdresaSpaco, PaĝaEraro};

pub type ProcezaId = u64;

const MAKSAJ_PROCEZOJ: usize = 8;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ProcezaEstado {
    Preta,
    Kuranta,
    Mortinta,
}

#[derive(Debug)]
pub enum ProcezaEraro {
    NevalidaIdentigilo,
    JamRegistrita,
    Paĝa(PaĝaEraro),
}

impl fmt::Display for ProcezaEraro {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NevalidaIdentigilo => f.write_str("proceza identigilo ekster la tabelo"),
            Self::JamRegistrita => f.write_str("procezo kun tiu identigilo jam ekzistas"),
            Self::Paĝa(eraro) => write!(f, "paĝa eraro: {eraro}"),
        }
    }
}

impl From<PaĝaEraro> for ProcezaEraro {
    fn from(eraro: PaĝaEraro) -> Self {
        Self::Paĝa(eraro)
    }
}

pub struct Procezo {
    id: ProcezaId,
    adresa_spaco: Spaco,
    kerna_stako_pinto: u64,
    estado: ProcezaEstado,
}

impl Procezo {
    /// Kreu procezon, kies adresa spaco heredas la kernajn mapojn de
    /// `kerna_spaco` kaj komencas kun malplena uzanta fenestro.
    fn nova(
        id: ProcezaId,
        kerna_spaco: &Spaco,
        alokilo: &mut KadraAlokilo,
    ) -> Result<Self, PaĝaEraro> {
        Ok(Self {
            id,
            adresa_spaco: kerna_spaco.krei_kunhavantan(alokilo)?,
            // La pinton de la kerna stako muntos la transiro Ring 3 → Ring 0
            // (`TSS.RSP0`) en Fazo 3.
            kerna_stako_pinto: 0,
            estado: ProcezaEstado::Preta,
        })
    }

    pub fn id(&self) -> ProcezaId {
        self.id
    }

    pub fn spaco(&mut self) -> &mut Spaco {
        &mut self.adresa_spaco
    }

    pub fn kerna_stako_pinto(&self) -> u64 {
        self.kerna_stako_pinto
    }

    pub fn stato(&self) -> ProcezaEstado {
        self.estado
    }
}

static PROCEZOJ: Kunhavigita<[Option<Procezo>; MAKSAJ_PROCEZOJ]> =
    Kunhavigita::nova([const { None }; MAKSAJ_PROCEZOJ]);

fn tabelo() -> &'static mut [Option<Procezo>; MAKSAJ_PROCEZOJ] {
    PROCEZOJ.akiri()
}

/// Registru novan procezon en la tabelo. `id == 0` estas la nukleo mem, do
/// ĝi estas rifuzata; same identigilo ekster la tabelo.
pub fn krei(
    id: ProcezaId,
    kerna_spaco: &Spaco,
    alokilo: &mut KadraAlokilo,
) -> Result<ProcezaId, ProcezaEraro> {
    if id == 0 || id as usize >= MAKSAJ_PROCEZOJ {
        return Err(ProcezaEraro::NevalidaIdentigilo);
    }
    if tabelo()[id as usize].is_some() {
        return Err(ProcezaEraro::JamRegistrita);
    }
    let procezo = Procezo::nova(id, kerna_spaco, alokilo)?;
    tabelo()[id as usize] = Some(procezo);
    Ok(id)
}

pub fn akiri(id: ProcezaId) -> Option<&'static mut Procezo> {
    let indekso = id as usize;
    if indekso >= MAKSAJ_PROCEZOJ {
        return None;
    }
    tabelo()[indekso].as_mut()
}

/// Ŝanĝu la staton de la donita procezo; nekonata `id` estas senefika.
pub fn marki(id: ProcezaId, stato: ProcezaEstado) {
    if let Some(procezo) = akiri(id) {
        procezo.estado = stato;
    }
}
