//! KASLR: elektas hazardan kernan bazadreson, kopias la bildon tien kaj
//! aplikas la `R_X86_64_RELATIVE`-relokojn de la pozicio-sendependa bildo.
//!
//! Ĉi tiu modulo ruliĝas ankoraŭ ĉe la liga bazo (1 MiB), kun la identeca
//! paĝtabelo de la praŝarĝa ŝelko. Post la kopio kaj la relokado ĝi saltas en
//! la kopion; se la kondiĉoj mankas, ĝi daŭrigas ĉe la liga bazo kaj la
//! normala enirpunkto raportas, ke KASLR restis malŝaltita.

use core::ptr;

use crate::memoro::EniraInformo;
use crate::memoro::kadro::MAKSA_FIZIKA_MEMORO;

/// La bazo, ĉe kiu la bildo estas ligita kaj kie la praŝarĝilo metas ĝin.
const LIGA_BAZO: u64 = 0x10_0000;
/// Alinio de la hazarda bazo (2 MiB, la paĝa grandeco de la ŝelko).
const ALINIO: u64 = 0x20_0000;
/// Sub ĉi tiu adreso kuŝas BIOS/legacy-strukturoj.
const MALALTA_LIMO: u64 = 0x20_0000;

const R_X86_64_RELATIVE: u32 = 8;

unsafe extern "C" {
    static __kerna_komenco: u8;
    static __kerna_fino: u8;
    static __rela_komenco: Elf64Rela;
    static __rela_fino: Elf64Rela;
    fn kerna_salto_al_kopio(starta_informo: *const u8, magio: u32, bazo: u64, origino: u64) -> !;
}

/// `Elf64_Rela`: `r_offset`, `r_info`, `r_addend`.
#[repr(C)]
struct Elf64Rela {
    offset: u64,
    info: u64,
    addend: i64,
}

/// La adreso de la kuranta kerna bildo.
pub fn bazo() -> u64 {
    &raw const __kerna_komenco as u64
}

/// La unua kerna enirpunkto: provu KASLR kaj neniam revenu.
pub fn hazardigi(starta_informo: *const u8, magio: u32) -> ! {
    let origino = bazo();
    let bildo = bilda_grando();
    if let Ok(informo) = crate::arkitekturo::analizi_eniron(starta_informo, magio) {
        if let Some(bazo) = elekti_bazon(&informo, bildo, origino) {
            kopii(origino, bazo, bildo);
            if relokigi(bazo, bildo) {
                // SAFETY: `bazo` estas 2 MiB-aliniiĝinta regiono tute ene de
                // uzebla RAM, ekster la bildo, la moduloj kaj la praŝarĝaj
                // strukturoj; la kopio kaj la relokado finiĝis, do la salto en
                // la kopion trovas plenan bildon kun la ĝusta stako.
                unsafe { kerna_salto_al_kopio(starta_informo, magio, bazo, origino) }
            }
        }
    }
    // Ankaŭ sen kopio la relokoj devas esti aplikataj: la ligilo lasas la celajn
    // fakojn nulaj kaj metis la valorojn nur en `.rela.dyn` (RELA havas la
    // aldonaĵon en la tabelo, ne en la bildo). Ĉe la liga bazo la deŝovo estas 0.
    relokigi(origino, bildo);
    crate::kerna_eniro_kopio(starta_informo, magio, origino, origino)
}

/// Longo de la kerna bildo (`__kerna_fino - __kerna_komenco`).
fn bilda_grando() -> u64 {
    (&raw const __kerna_fino as u64) - (&raw const __kerna_komenco as u64)
}

/// Elektu bazadreson el la uzeblaj regionoj; `None` se ne ekzistas almenaŭ du
/// kandidatoj (tiam hazardigo ne eblas).
fn elekti_bazon(informo: &EniraInformo, bildo: u64, origino: u64) -> Option<u64> {
    let mut nombro: u64 = 0;
    trakandidatoj(informo, bildo, origino, &mut |_| {
        nombro = nombro.saturating_add(1);
    });
    if nombro < 2 {
        return None;
    }
    let celo = crate::arkitekturo::hazardo() % nombro;
    let mut indekso: u64 = 0;
    let mut elektita = None;
    trakandidatoj(informo, bildo, origino, &mut |bazo| {
        if indekso == celo {
            elektita = Some(bazo);
        }
        indekso += 1;
    });
    elektita
}

/// Voku `ago` por ĉiu 2 MiB-aliniiĝinta bazo, kiu entenas la tutan bildon ene
/// de uzebla regiono kaj ne tuŝas praŝarĝajn strukturojn.
fn trakandidatoj(informo: &EniraInformo, bildo: u64, origino: u64, ago: &mut dyn FnMut(u64)) {
    for regiono in informo
        .memora_mapo
        .regionoj()
        .iter()
        .filter(|regiono| regiono.uzebla())
    {
        let mut bazo = alinii_supre(regiono.komenco.max(MALALTA_LIMO));
        let fino = regiono.fino().min(MAKSA_FIZIKA_MEMORO);
        while let Some(bildo_fino) = bazo.checked_add(bildo) {
            if bildo_fino > fino {
                break;
            }
            if libera(bazo, bildo, origino, informo) {
                ago(bazo);
            }
            bazo += ALINIO;
        }
    }
}

fn alinii_supre(adreso: u64) -> u64 {
    adreso.div_ceil(ALINIO) * ALINIO
}

fn intersekcias(a: u64, b: u64, c: u64, d: u64) -> bool {
    a < d && c < b
}

/// Ĉu `[bazo, bazo + bildo)` ne tuŝas la ligebildon, la modulojn, la
/// praŝarĝan informon aŭ la framebufferon?
fn libera(bazo: u64, bildo: u64, origino: u64, informo: &EniraInformo) -> bool {
    let bildo_fino = bazo + bildo;
    if bildo_fino > MAKSA_FIZIKA_MEMORO {
        return false;
    }
    if intersekcias(bazo, bildo_fino, origino, origino + bildo) {
        return false;
    }
    for regiono in informo.rezervindaj {
        if intersekcias(bazo, bildo_fino, regiono.komenco, regiono.fino()) {
            return false;
        }
    }
    for modulo in informo.moduloj {
        if intersekcias(bazo, bildo_fino, modulo.komenco, modulo.fino()) {
            return false;
        }
    }
    match informo.framebuffer {
        Some(fb) => {
            let fino = fb.bazo + fb.paŝo as u64 * fb.alto as u64;
            !intersekcias(bazo, bildo_fino, fb.bazo, fino)
        }
        None => true,
    }
}

/// Kopiu `bildo` bajtojn de `origino` al `bazo` per `rep movsb`.
fn kopii(origino: u64, bazo: u64, bildo: u64) {
    let fonto = origino as *const u8;
    let celo = bazo as *mut u8;
    // SAFETY: ambaŭ intervaloj estas mapitaj de la praŝarĝa identeca mapo kaj
    // ne interkovriĝas (la kandidatoj ekskludas la ligebildon).
    unsafe {
        core::arch::asm!(
            "cld",
            "rep movsb",
            inout("rsi") fonto as u64 => _,
            inout("rdi") celo as u64 => _,
            inout("rcx") bildo => _,
            options(nostack),
        );
    }
}

/// Apliku la RELATIVE-relokojn al la kopio. Redonas `false` se la tabelo
/// enhavas ion, kion ĉi tiu ŝelko ne komprenas; tiam la bildo restas ĉe la
/// liga bazo.
fn relokigi(bazo: u64, bildo: u64) -> bool {
    let deŝovo = bazo.wrapping_sub(LIGA_BAZO);
    let komenco = &raw const __rela_komenco as u64;
    let fino = &raw const __rela_fino as u64;
    let erogrando = core::mem::size_of::<Elf64Rela>() as u64;
    if fino < komenco || (fino - komenco) % erogrando != 0 {
        return false;
    }
    let nombro = ((fino - komenco) / erogrando) as usize;
    let eroj = komenco as *const Elf64Rela;

    // Kontrolu la tutan tabelon antaŭ la unua skribo, por ne lasi duonrelokitan
    // bildon se iu ero estas nekonata.
    for indekso in 0..nombro {
        // SAFETY: `eroj` montras al la reloka tabelo de la ligebildo, kiu havas
        // `nombro` erojn laŭ la ligilaj simboloj.
        let ero = unsafe { &*eroj.add(indekso) };
        if (ero.info & 0xffff_ffff) as u32 != R_X86_64_RELATIVE {
            return false;
        }
        if ero.offset < LIGA_BAZO || ero.offset >= LIGA_BAZO + bildo {
            return false;
        }
    }

    for indekso in 0..nombro {
        // SAFETY: sama tabelo kiel supre.
        let ero = unsafe { &*eroj.add(indekso) };
        let celo = (ero.offset.wrapping_add(deŝovo)) as *mut u64;
        let valoro = (ero.addend as u64).wrapping_add(deŝovo);
        // SAFETY: la kontrolo supre garantiis, ke `celo` kuŝas ene de la kopio;
        // la kopio estas identece mapita kaj ankoraŭ ne uzata de aliulo.
        unsafe { ptr::write_volatile(celo, valoro) };
    }
    true
}
