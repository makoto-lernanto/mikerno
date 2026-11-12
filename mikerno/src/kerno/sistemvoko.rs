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

// M11 anoncas la regionojn por la montrila kontrolo; Task 4 mapos ilin.
const BILDO_BAZO: u64 = 0x0000_0001_0000_0000;
const BILDO_MAKSO: u64 = 4 * 1024 * 1024;
const STAKO_SUPRO: u64 = BILDO_BAZO + 16 * 1024 * 1024;
const STAKO_GRANDE: u64 = 64 * 1024;

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
    if !kontroli_bufron(montrilo, longo, el_ring3) {
        return MALĜUSTA_ARG;
    }
    // SAFETY: Ring 0-montriloj estas fidindaj; Ring 3-montriloj trapasis la
    // regionan kontrolon, do la bufro estas ne-nula kaj ene de la anoncita
    // bildo aŭ stako. `longo` estas limigita per `MAKS_SKRIBO`.
    let bufro = unsafe { core::slice::from_raw_parts(montrilo as *const u8, longo) };
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
    if !kontroli_mesagon(montrilo, el_ring3) {
        return MALĜUSTA_ARG;
    }
    // SAFETY: la montrilo estas ne-nula kaj (por Ring 3) en la anoncita
    // regiono; `Mesago` estas `Copy`, do unufoja legado sufiĉas.
    let mesago = unsafe { core::ptr::read_unaligned(montrilo as *const Mesago) };
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
    if fonto > planisto::MAKSAJ_FADENOJ as u64 || !kontroli_mesagon(celo, el_ring3) {
        return MALĜUSTA_ARG;
    }
    let mesago = ipc::ricevi_mesagon(fonto);
    // SAFETY: `celo` estas ne-nula kaj (por Ring 3) en la anoncita regiono;
    // `Mesago` estas `Copy` kaj la skribo okazas unufoje, sen referenco restanta.
    unsafe {
        core::ptr::write_unaligned(celo as *mut Mesago, mesago);
    }
    0
}

/// Ne-nula, limigita kaj (por Ring 3) ene de la anoncita uzanta regiono.
fn kontroli_bufron(montrilo: u64, longo: usize, el_ring3: bool) -> bool {
    if montrilo == 0 || longo == 0 || longo > MAKS_SKRIBO {
        return false;
    }
    let Some(fino) = montrilo.checked_add(longo as u64) else {
        return false;
    };
    !el_ring3 || en_uzanta_regiono(montrilo, fino)
}

fn kontroli_mesagon(montrilo: u64, el_ring3: bool) -> bool {
    if montrilo == 0 {
        return false;
    }
    let Some(fino) = montrilo.checked_add(size_of::<Mesago>() as u64) else {
        return false;
    };
    !el_ring3 || en_uzanta_regiono(montrilo, fino)
}

fn en_uzanta_regiono(komenco: u64, fino: u64) -> bool {
    let en_bildo = komenco >= BILDO_BAZO && fino <= BILDO_BAZO + BILDO_MAKSO;
    let en_stako = komenco >= STAKO_SUPRO - STAKO_GRANDE && fino <= STAKO_SUPRO;
    en_bildo || en_stako
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
