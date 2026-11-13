//! Sistemvokoj: la `int 0x80`-interkonsento kaj ĝia kerna efektivigo.
//!
//! Registra ABI (vidu `libmikerno::syscall`): `rax` = numero,
//! `rdi`/`rsi`/`rdx`/`r10` = argumentoj, `rax` = rezulto. La arkitektura
//! tavolo legas la registrojn el `InterrompaKadro` kaj skribas la rezulton
//! reen; ĉi tiu modulo nur decidas, kion la voko signifas.

use core::mem::size_of;

use libmikerno::Mesago;
use libmikerno::syscall::{SYS_CEDI, SYS_FINI, SYS_RICEVI, SYS_SENDI, SYS_SKRIBI};

use crate::interrompo::{InterrompaKadro, InterrompaTraktilo, SISTEMVOKA_VEKTORO};
use crate::kunhavigita::Kunhavigita;
use crate::memoro::virtuala::{PaĝaTablo, UZANTA_STAKO_GRANDE, UZANTA_STAKO_SUPRO};

use super::{ipc, planisto};

/// `-1`: la argumentoj estas malvalidaj.
pub const MALĜUSTA_ARG: u64 = (-1i64) as u64;
/// `-2`: la numero ne apartenas al la interkonsento.
pub const NEKONATA_VOKO: u64 = (-2i64) as u64;
/// `-3`: la celo de mesaĝo ne estas fadeno.
pub const NEVALIDA_CELLOKO: u64 = (-3i64) as u64;
/// `-4`: la mesaĝujo de la celo estas plena.
pub const MESAGUJO_PLENA: u64 = (-4i64) as u64;

/// Maksimuma longo de unu `SYS_SKRIBI`-bufro.
const MAKS_SKRIBO: usize = 4096;

/// Kerna kopiujo por `SYS_SKRIBI`: la uzanta bufro estas legata nur dum la
/// mallonga `STAC`-fenestro, kaj la formatado poste legas ĉi tiun bufron. La
/// aliro estas unuprocesora kaj okazas kun interrompoj malŝaltitaj.
static SKRIBA_BUFRO: Kunhavigita<[u8; MAKS_SKRIBO]> = Kunhavigita::nova([0; MAKS_SKRIBO]);

/// Efektivigu unu sistemvokon.
///
/// `el_ring3` estas vera nur por vokoj el Ring 3: la kerno mem estas fidinda
/// kaj rajtas uzi kernajn montrilojn, dum uzantaj montriloj estas limigitaj al
/// la anoncitaj bildaj kaj stakaj regionoj.
pub fn voki(numero: usize, a0: u64, a1: u64, _a2: u64, _a3: u64, el_ring3: bool) -> u64 {
    match numero {
        SYS_SKRIBI => skribi(a0, a1, el_ring3),
        SYS_FINI => fini(),
        SYS_CEDI => cedi(),
        SYS_SENDI => sendi(a0, a1, el_ring3),
        SYS_RICEVI => ricevi(a0, a1, el_ring3),
        _ => NEKONATA_VOKO,
    }
}

fn skribi(montrilo: u64, longo: u64, el_ring3: bool) -> u64 {
    let Ok(longo) = usize::try_from(longo) else {
        return MALĜUSTA_ARG;
    };
    if !kontroli_bufron(montrilo, longo, false, el_ring3) {
        return MALĜUSTA_ARG;
    }
    // Ring 3-bufro estas kopiata en kernan bufron dum mallonga `STAC`-fenestro;
    // la formatado poste legas nur la kernan kopion. Ring 0-montriloj estas
    // fidindaj kaj ne bezonas fenestron.
    let bufro: &[u8] = if el_ring3 {
        let celo = SKRIBA_BUFRO.akiri();
        {
            let _gardo = crate::arkitekturo::StacGardo::nova();
            // SAFETY: Ring 3-montriloj trapasis la regionan kontrolon, do la
            // bufro estas ne-nula, maplitan kaj ene de la anoncita bildo aŭ
            // stako; `longo` estas limigita per `MAKS_SKRIBO`. La celo estas
            // privata kerna bufro de sama grando, kaj interrompoj estas
            // malŝaltitaj, do neniu alia kopio samtempe uzas ĝin.
            unsafe {
                core::ptr::copy_nonoverlapping(montrilo as *const u8, celo.as_mut_ptr(), longo);
            }
        }
        &celo[..longo]
    } else {
        // SAFETY: Ring 0-montriloj estas fidindaj kaj la longo estas limigita.
        unsafe { core::slice::from_raw_parts(montrilo as *const u8, longo) }
    };
    match core::str::from_utf8(bufro) {
        Ok(teksto) => {
            crate::skribu!("{}", teksto);
            0
        }
        Err(_) => MALĜUSTA_ARG,
    }
}

fn fini() -> u64 {
    // `morti` neniam revenas; post ĝi ne ekzistas resuriga kodo.
    planisto::morti()
}

fn cedi() -> u64 {
    planisto::cedi();
    0
}

fn sendi(destinacio: u64, montrilo: u64, el_ring3: bool) -> u64 {
    if !kontroli_mesagon(montrilo, false, el_ring3) {
        return MALĜUSTA_ARG;
    }
    let mesago = {
        let _gardo = el_ring3.then(crate::arkitekturo::StacGardo::nova);
        // SAFETY: la montrilo estas ne-nula kaj (por Ring 3) en la anoncita
        // regiono; `Mesago` estas `Copy`, do unufoja legado sufiĉas. La
        // `STAC`-fenestro kovras nur la legadon, ne la IPC-vokon.
        unsafe { core::ptr::read_unaligned(montrilo as *const Mesago) }
    };
    if mesago.longo() > ipc::MAKSA_MESAGA_LONGO {
        return MALĜUSTA_ARG;
    }
    match ipc::sendi_mesagon(destinacio, &mesago) {
        Ok(()) => 0,
        Err(ipc::IpcEraro::NevalidaCelloko) => NEVALIDA_CELLOKO,
        Err(ipc::IpcEraro::MesagujoPlena) => MESAGUJO_PLENA,
    }
}

fn ricevi(fonto: u64, celo: u64, el_ring3: bool) -> u64 {
    if fonto > planisto::MAKSAJ_FADENOJ as u64 || !kontroli_mesagon(celo, true, el_ring3) {
        return MALĜUSTA_ARG;
    }
    // `ricevi_mesagon` rajtas bloki kaj cedi la procesoron, do ĝi restas
    // ekster la `STAC`-fenestro.
    let mesago = ipc::ricevi_mesagon(fonto);
    {
        let _gardo = el_ring3.then(crate::arkitekturo::StacGardo::nova);
        // SAFETY: `celo` estas ne-nula kaj (por Ring 3) en la anoncita regiono;
        // `Mesago` estas `Copy` kaj la skribo okazas unufoje, sen referenco
        // restanta. La fenestro kovras nur la skribon.
        unsafe {
            core::ptr::write_unaligned(celo as *mut Mesago, mesago);
        }
    }
    0
}

/// Ne-nula, limigita kaj (por Ring 3) ene de la anoncita uzanta regiono.
fn kontroli_bufron(montrilo: u64, longo: usize, skriba: bool, el_ring3: bool) -> bool {
    if montrilo == 0 || longo == 0 || longo > MAKS_SKRIBO {
        return false;
    }
    kontroli_regionon(montrilo, longo, skriba, el_ring3)
}

fn kontroli_mesagon(montrilo: u64, skriba: bool, el_ring3: bool) -> bool {
    if montrilo == 0 {
        return false;
    }
    kontroli_regionon(montrilo, size_of::<Mesago>(), skriba, el_ring3)
}

/// Kontrolu Ring 3-montrilon: sen ciferpremo, ene de la anoncita bildo aŭ
/// stako, kaj kun la postulata folia permeso — `skriba` por skribo, alie por
/// legado. Ring 0-montriloj estas fidindaj kaj trapasas sen kontrolo.
fn kontroli_regionon(montrilo: u64, longo: usize, skriba: bool, el_ring3: bool) -> bool {
    let Some(fino) = montrilo.checked_add(longo as u64) else {
        return false;
    };
    if !el_ring3 {
        return true;
    }
    let Some(procezo) = super::procezo::akiri(super::planisto::nuna_procezo()) else {
        return false;
    };
    let id = super::planisto::nuna_procezo();
    let (bilda_komenco, bilda_fino) = procezo.bildaj_limoj();
    let en_bildo = bilda_fino > bilda_komenco && montrilo >= bilda_komenco && fino <= bilda_fino;
    let staka_komenco = UZANTA_STAKO_SUPRO - UZANTA_STAKO_GRANDE as u64;
    let en_stako = montrilo >= staka_komenco && fino <= UZANTA_STAKO_SUPRO;
    if !en_bildo && !en_stako {
        crate::skribu!(
            "[eraro] procezo {}: montrilo 0x{:x} (longo {}) estas ekster la anoncita regiono\n",
            id,
            montrilo,
            longo
        );
        return false;
    }
    if !procezo.spaco().kontroli_uzantan(montrilo, longo, skriba) {
        crate::skribu!(
            "[eraro] procezo {}: montrilo 0x{:x} (longo {}, skriba {}) ne havas la postulatan folian permeson\n",
            id,
            montrilo,
            longo,
            skriba
        );
        return false;
    }
    true
}

struct SistemvokaTraktilo;

impl InterrompaTraktilo for SistemvokaTraktilo {
    fn trakta(&self, kadro: &mut InterrompaKadro) {
        crate::arkitekturo::sistemvoka_dispetĉo(kadro);
    }
}

static TRAKTILO: SistemvokaTraktilo = SistemvokaTraktilo;

pub fn pravalorizi() {
    crate::interrompo::registri(SISTEMVOKA_VEKTORO, &TRAKTILO);
    crate::skribu!("[sistemvoko] pordego 0x80 (DPL3) instalita\n");
}

/// Ring 0-memtesto de la dispetĉo kaj de la argumenta kontrolo.
///
/// Vokata de la unua fadeno: `SYS_CEDI` tie vere cedas la procesoron, dum en la
/// praŝarĝa kunteksto ĝi nur revenus sen ŝanĝo.
pub fn testi_ring0() {
    let teksto = "saluton el Ring 0 tra int 0x80";
    let skribo = invoki(
        SYS_SKRIBI,
        [teksto.as_ptr() as u64, teksto.len() as u64, 0, 0],
    );
    let nekonata = invoki(0x7FFF, [0; 4]);
    let nula = invoki(SYS_SKRIBI, [0, 8, 0, 0]);
    let troa = invoki(
        SYS_SKRIBI,
        [teksto.as_ptr() as u64, (MAKS_SKRIBO + 1) as u64, 0, 0],
    );
    let cedo = invoki(SYS_CEDI, [0; 4]);
    crate::skribu!(
        "[sistemvoko] Ring 0-memtesto: skribo={}, nekonata={}, nula={}, troa={}, cedo={}\n",
        skribo as i64,
        nekonata as i64,
        nula as i64,
        troa as i64,
        cedo as i64
    );
}

fn invoki(numero: usize, argumentoj: [u64; 4]) -> u64 {
    let rezulto: u64;
    // SAFETY: `int 0x80` eniras la IDT-pordon instalitan de ĉi tiu modulo; la
    // registra ABI kongruas kun `libmikerno::syscall`. La voko ne tuŝas la
    // stakon (`nostack`).
    unsafe {
        core::arch::asm!(
            "int 0x80",
            inlateout("rax") numero => rezulto,
            in("rdi") argumentoj[0],
            in("rsi") argumentoj[1],
            in("rdx") argumentoj[2],
            in("r10") argumentoj[3],
            options(nostack),
        );
    }
    rezulto
}
