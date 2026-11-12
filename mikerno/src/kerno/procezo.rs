//! Proceza abstrakto: identigilo, privata adresa spaco, kerna stako kaj stato.
//!
//! La nukleo mem estas la procezo `0`; ĝi ne havas eron en la tabelo. Ĉiu
//! procezo posedas propran radikan paĝtabelon, kiu kunhavas la kernajn mapojn
//! sed havas privatan, malplenan uzantan fenestron, plus propran 16 KiB-an
//! kernan stakon por transiroj Ring 3 → Ring 0.

use core::fmt;

use crate::arkitekturo;
use crate::arkitekturo::Spaco;
use crate::kunhavigita::Kunhavigita;
use crate::memoro::kadro::KadraAlokilo;
use crate::memoro::virtuala::{AdresaSpaco, FizikaAdreso, PaĝaEraro};

pub type ProcezaId = u64;

const MAKSAJ_PROCEZOJ: usize = 8;
const KERNA_STAKO_GRANDE: usize = 16 * 1024;

#[repr(align(16))]
struct KernaStako(#[allow(dead_code)] [u8; KERNA_STAKO_GRANDE]);

// La grando estas multoblo de 16, do ankaŭ la pinto restas 16-bajte
// aliniiĝinta, kion postulas la x86-64-ABI kaj la interrompa kadro.
const _: () = assert!(KERNA_STAKO_GRANDE % 16 == 0);

/// La kernaj stakoj apartenas al la procezaj fenestroj de la tabelo: ili vivas
/// en `.bss` de la kerna bildo, kiun ĉiu proceza spaco kunhavas.
static mut KERNAJ_STAKOJ: [KernaStako; MAKSAJ_PROCEZOJ] =
    [const { KernaStako([0; KERNA_STAKO_GRANDE]) }; MAKSAJ_PROCEZOJ];

/// La pinto de la kerna stako de proceza fenestro. `id` jam validiĝis en
/// `krei`, do la indekso restas ene de la tabelo.
fn kerna_stako_pinto(id: ProcezaId) -> u64 {
    // SAFETY: la indekso estas ene de `KERNAJ_STAKOJ`; ni nur prenas la adreson
    // de la ero, sen krei referencon al la `static mut`.
    let bazo = unsafe { core::ptr::addr_of!(KERNAJ_STAKOJ[id as usize]) as u64 };
    bazo + KERNA_STAKO_GRANDE as u64
}

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
            kerna_stako_pinto: kerna_stako_pinto(id),
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

    pub fn radika_adreso(&self) -> FizikaAdreso {
        self.adresa_spaco.radika_adreso()
    }

    pub fn stato(&self) -> ProcezaEstado {
        self.estado
    }
}

static KERNA_RADIKO: Kunhavigita<u64> = Kunhavigita::nova(0);

/// Registru la radikan `PML4`-on de la nukleo (procezo `0`) post la unua
/// `CR3`-ŝalto. Antaŭ tio la nukleo uzas la praŝarĝilan tabularon.
pub fn agordi_kernan_spacon(radiko: FizikaAdreso) {
    *KERNA_RADIKO.akiri() = radiko.adreso();
}

fn kerna_radiko() -> Option<FizikaAdreso> {
    let radiko = *KERNA_RADIKO.akiri();
    (radiko != 0).then_some(FizikaAdreso::nova(radiko))
}

/// Aktivigu la adresan spacon kaj la kernan stakon de la donita procezo.
/// `0` estas la nukleo mem, kiu revenas al la kerna radiko kaj al la defaŭlta
/// `RSP0`.
///
/// # Sekureco
/// Vokenda en Ring 0 kun interrompoj malŝaltitaj. La proceza spaco devas esti
/// vivanta kaj devas kunhavigi la kernajn mapojn de la aktiva spaco; tion
/// garantiis `krei_kunhavantan`.
pub unsafe fn aktivigi(id: ProcezaId) {
    let mut pinto = arkitekturo::defaŭlta_kernostako();
    let radiko = if id == 0 {
        kerna_radiko()
    } else {
        akiri(id).map(|procezo| {
            pinto = procezo.kerna_stako_pinto;
            procezo.radika_adreso()
        })
    };
    if let Some(radiko) = radiko {
        if arkitekturo::aktiva_adreso_spaco() != radiko.adreso() {
            // SAFETY: la alvokanto garantias Ring 0 kun interrompoj malŝaltitaj;
            // la celo estas viva kerna aŭ proceza `PML4` kun kunhavataj mapoj.
            unsafe { arkitekturo::ŝalti_adresan_spacon(radiko.adreso()) };
        }
    }
    arkitekturo::agordi_kernostakon(pinto);
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
