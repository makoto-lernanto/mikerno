//! Lokala APIC (xAPIC): MMIO-mapado, malŝalto de la malnova 8259-PIC,
//! ŝalto de la APIC kaj legado de la identigilo.

use core::arch::asm;
use core::ptr;
use core::sync::atomic::{AtomicU64, Ordering};

use crate::memoro::kadro::KadraAlokilo;
use crate::memoro::virtuala::{AdresaSpaco, FizikaAdreso, PaĝaEraro, PaĝaPermesoj, VirtuaAdreso};

const APIC_BAZA_MSR: u32 = 0x1B;
const APIC_GLOBALA_BITO: u64 = 1 << 11;
const APIC_BAZA_MASKO: u64 = 0x000F_FFFF_FFFF_F000;
const DEFAULTA_BAZO: u64 = 0xFEE0_0000;

pub(crate) const REGISTRO_ID: u64 = 0x020;
pub(crate) const REGISTRO_EOI: u64 = 0x0B0;
pub(crate) const REGISTRO_SVR: u64 = 0x0F0;
pub(crate) const REGISTRO_LVT_TEMPORIZILO: u64 = 0x320;
pub(crate) const REGISTRO_TEMPORIZILA_DIVIDO: u64 = 0x3E0;
pub(crate) const REGISTRO_TEMPORIZILA_KOMENCA: u64 = 0x380;
pub(crate) const REGISTRO_TEMPORIZILA_NUNA: u64 = 0x390;

const SPURIA_VEKTORO: u32 = 0xFF;
const SVR_AKTIVA_BITO: u32 = 1 << 8;

static mut APIC_BAZO: u64 = 0;
/// Nombrilo de la senditaj `EOI`-oj; la fina raporto komparas ĝin kun la
/// temporizilaj tikoj.
static KONFIRMOJ: AtomicU64 = AtomicU64::new(0);

fn legi_msr(msr: u32) -> u64 {
    let malalta: u32;
    let alta: u32;
    unsafe {
        asm!(
            "rdmsr",
            in("ecx") msr,
            out("eax") malalta,
            out("edx") alta,
            options(nomem, nostack, preserves_flags),
        );
    }
    ((alta as u64) << 32) | malalta as u64
}

fn skribi_msr(msr: u32, valoro: u64) {
    unsafe {
        asm!(
            "wrmsr",
            in("ecx") msr,
            in("eax") valoro as u32,
            in("edx") (valoro >> 32) as u32,
            options(nomem, nostack, preserves_flags),
        );
    }
}

fn bazo() -> u64 {
    unsafe { *ptr::addr_of!(APIC_BAZO) }
}

pub(crate) fn skribi(offset: u64, valoro: u32) {
    unsafe {
        ptr::write_volatile((bazo() + offset) as *mut u32, valoro);
    }
}

pub(crate) fn legi(offset: u64) -> u32 {
    unsafe { ptr::read_volatile((bazo() + offset) as *const u32) }
}

/// Preparu la lokan APIC-on kaj redonu ĝian identigilon.
pub fn pravalorizi(
    spaco: &mut dyn AdresaSpaco,
    alokilo: &mut KadraAlokilo,
) -> Result<u32, PaĝaEraro> {
    unsafe {
        // Malŝaltu la 8259-PIC-on: 0x21 kaj 0xA1 estas ĝiaj maskaj registroj.
        asm!("out dx, al", in("dx") 0x21u16, in("al") 0xFFu8, options(nomem, nostack, preserves_flags));
        asm!("out dx, al", in("dx") 0xA1u16, in("al") 0xFFu8, options(nomem, nostack, preserves_flags));
    }

    let msr = legi_msr(APIC_BAZA_MSR);
    let bazo = if msr & APIC_BAZA_MASKO != 0 {
        msr & APIC_BAZA_MASKO
    } else {
        DEFAULTA_BAZO
    };
    // La masko nuligas ankaŭ la x2APIC-biton; ni restas en MMIO-reĝimo.
    skribi_msr(APIC_BAZA_MSR, (bazo & APIC_BAZA_MASKO) | APIC_GLOBALA_BITO);

    let virtuala = VirtuaAdreso::nova(bazo);
    match spaco.traduki(virtuala) {
        Some(fizika) if fizika.adreso() == bazo => {}
        Some(_) => return Err(PaĝaEraro::Konflikto),
        None => spaco.mapigi(
            virtuala,
            FizikaAdreso::nova(bazo),
            PaĝaPermesoj::SKRIBA,
            alokilo,
        )?,
    }

    unsafe {
        *ptr::addr_of_mut!(APIC_BAZO) = bazo;
    }

    skribi(REGISTRO_SVR, SPURIA_VEKTORO | SVR_AKTIVA_BITO);

    Ok(legi(REGISTRO_ID) >> 24)
}

pub fn konfirmi() {
    if bazo() != 0 {
        skribi(REGISTRO_EOI, 0);
        KONFIRMOJ.fetch_add(1, Ordering::Relaxed);
    }
}

/// Kiom da `EOI`-oj estis senditaj ekde la starto?
pub fn konfirmoj() -> u64 {
    KONFIRMOJ.load(Ordering::Relaxed)
}
