//! x86_64 4-nivela paĝado: PML4 → PDPT → PD → PT, kun 4 KiB folioj.
//!
//! La praŝarĝa ŝelko jam mapis la unuajn 4 GiB per 2 MiB paĝoj; ĉi tiu modulo
//! konstruas pli detalan radikan tabelon kaj transdonas ĝin al CR3.

use core::arch::asm;
use core::ptr;

use crate::memoro::KADRA_GRANDO;
use crate::memoro::kadro::{KadraAlokilo, Kadro};
use crate::memoro::virtuala::{
    AdresaSpaco, FizikaAdreso, PaĝaEraro, PaĝaPermesoj, PaĝaTablo, UZANTA_FINO, VirtuaAdreso,
};

const PREZENTA: u64 = 1 << 0;
const SKRIBEBA: u64 = 1 << 1;
const UZANTA: u64 = 1 << 2;
const NEKAŜEBLA: u64 = 1 << 4;
const GRANDA: u64 = 1 << 7;
const NE_PLENUMEBLA: u64 = 1 << 63;

/// Bitoj 51:12 — la fizika kadro de folia ero.
const ADRESO_MASKO: u64 = 0x000F_FFFF_FFFF_F000;
/// Bitoj 51:21 — la bazo de 2 MiB grandega paĝo.
const DU_MIB_BAZA_MASKO: u64 = 0x000F_FFFF_FFE0_0000;

const EFER_MSR: u32 = 0xC000_0080;
const NXE_BITO: u32 = 1 << 11;

/// Ŝaltu `EFER.NXE`, por ke mapoj rajtas uzi la biton 63.
pub fn pravalorizi() {
    unsafe {
        let malalta: u32;
        let alta: u32;
        asm!(
            "rdmsr",
            in("ecx") EFER_MSR,
            out("eax") malalta,
            out("edx") alta,
            options(nostack, preserves_flags),
        );
        asm!(
            "wrmsr",
            in("ecx") EFER_MSR,
            in("eax") malalta | NXE_BITO,
            in("edx") alta,
            options(nostack, preserves_flags),
        );
    }
}

/// Adresa spaco de la nukleo: posedas la radikan paĝan tabelon.
pub struct X86AdresaSpaco {
    radiko: FizikaAdreso,
}

impl X86AdresaSpaco {
    fn nova(alokilo: &mut KadraAlokilo) -> Result<Self, PaĝaEraro> {
        let kadro = alokilo.aloki().ok_or(PaĝaEraro::Elĉerpita)?;
        nuligi(kadro);
        Ok(Self {
            radiko: FizikaAdreso::nova(kadro.adreso()),
        })
    }
}

impl AdresaSpaco for X86AdresaSpaco {
    fn krei(alokilo: &mut KadraAlokilo) -> Result<Self, PaĝaEraro> {
        Self::nova(alokilo)
    }

    fn ŝalti(&self) {
        // PAE, LME kaj CR0.PG jam estas ŝaltitaj de la praŝarĝa ŝelko.
        unsafe {
            asm!(
                "mov cr3, {}",
                in(reg) self.radiko.adreso(),
                options(nostack, preserves_flags),
            );
        }
    }
}

impl PaĝaTablo for X86AdresaSpaco {
    fn mapigi(
        &mut self,
        virtuala: VirtuaAdreso,
        fizika: FizikaAdreso,
        permesoj: PaĝaPermesoj,
        alokilo: &mut KadraAlokilo,
    ) -> Result<(), PaĝaEraro> {
        if virtuala.adreso() % KADRA_GRANDO != 0 || fizika.adreso() % KADRA_GRANDO != 0 {
            return Err(PaĝaEraro::NevalidaAdreso);
        }

        let pdpt = suba_tabelo(self.radiko, indico(virtuala, 3), permesoj.uzanta, alokilo)?;
        let pd = suba_tabelo(pdpt, indico(virtuala, 2), permesoj.uzanta, alokilo)?;
        let pt = suba_tabelo(pd, indico(virtuala, 1), permesoj.uzanta, alokilo)?;

        let indekso = indico(virtuala, 0);
        if ero(pt, indekso) & PREZENTA != 0 {
            return Err(PaĝaEraro::Konflikto);
        }
        agordi_eron(
            pt,
            indekso,
            fizika.adreso() | PREZENTA | permesaj_eroj(permesoj),
        );
        flushi(virtuala);
        Ok(())
    }

    fn kontroli_uzantan(&self, adreso: u64, longo: usize, skriba: bool) -> bool {
        if longo == 0 {
            return true;
        }
        let Some(fino) = adreso.checked_add(longo as u64) else {
            return false;
        };
        if fino > UZANTA_FINO {
            return false;
        }

        let mut paĝo = adreso & !(KADRA_GRANDO - 1);
        loop {
            if !folio_uzanta(self.radiko, paĝo, skriba) {
                return false;
            }
            let sekva = paĝo + KADRA_GRANDO;
            if sekva >= fino {
                return true;
            }
            paĝo = sekva;
        }
    }

    fn traduki(&self, virtuala: VirtuaAdreso) -> Option<FizikaAdreso> {
        let pdpt = ero(self.radiko, indico(virtuala, 3));
        if pdpt & PREZENTA == 0 {
            return None;
        }
        let pd = ero(FizikaAdreso::nova(pdpt & ADRESO_MASKO), indico(virtuala, 2));
        if pd & PREZENTA == 0 {
            return None;
        }
        if pd & GRANDA != 0 {
            return Some(FizikaAdreso::nova(
                (pd & DU_MIB_BAZA_MASKO) | (virtuala.adreso() & 0x1F_FFFF),
            ));
        }

        let pt = ero(FizikaAdreso::nova(pd & ADRESO_MASKO), indico(virtuala, 1));
        if pt & PREZENTA == 0 {
            return None;
        }
        let folio = ero(FizikaAdreso::nova(pt & ADRESO_MASKO), indico(virtuala, 0));
        if folio & PREZENTA == 0 {
            return None;
        }
        Some(FizikaAdreso::nova(
            (folio & ADRESO_MASKO) | (virtuala.adreso() & 0xFFF),
        ))
    }
}

/// Indekso en la tabelo de la donita nivelo (0 = PT, 3 = PML4).
fn indico(virtuala: VirtuaAdreso, nivelo: u32) -> usize {
    ((virtuala.adreso() >> (12 + nivelo * 9)) & 0x1FF) as usize
}

fn ero(tabelo: FizikaAdreso, indekso: usize) -> u64 {
    unsafe { ptr::read_volatile((tabelo.adreso() + indekso as u64 * 8) as *const u64) }
}

fn agordi_eron(tabelo: FizikaAdreso, indekso: usize, valoro: u64) {
    unsafe {
        ptr::write_volatile((tabelo.adreso() + indekso as u64 * 8) as *mut u64, valoro);
    }
}

fn nuligi(kadro: Kadro) {
    unsafe {
        ptr::write_bytes(kadro.adreso() as *mut u8, 0, KADRA_GRANDO as usize);
    }
}

fn flushi(virtuala: VirtuaAdreso) {
    unsafe {
        asm!(
            "invlpg [{}]",
            in(reg) virtuala.adreso(),
            options(nostack, preserves_flags),
        );
    }
}

/// Liveru la sekvan tabelon; kreu kaj nuligu ĝin se mankas. La bito `U`
/// sekvas la folion: se la folio estas uzanta, ankaŭ la tabeloj estas.
fn suba_tabelo(
    tabelo: FizikaAdreso,
    indekso: usize,
    uzanta: bool,
    alokilo: &mut KadraAlokilo,
) -> Result<FizikaAdreso, PaĝaEraro> {
    let valoro = ero(tabelo, indekso);
    if valoro & PREZENTA == 0 {
        let kadro = alokilo.aloki().ok_or(PaĝaEraro::Elĉerpita)?;
        nuligi(kadro);
        let mut nova = kadro.adreso() | PREZENTA | SKRIBEBA;
        if uzanta {
            nova |= UZANTA;
        }
        agordi_eron(tabelo, indekso, nova);
        return Ok(FizikaAdreso::nova(kadro.adreso()));
    }
    if valoro & GRANDA != 0 {
        return Err(PaĝaEraro::Konflikto);
    }
    if uzanta && valoro & UZANTA == 0 {
        // La tabelo jam ekzistas pro kerna mapo; malfermu nur ĉi tiun vojon.
        agordi_eron(tabelo, indekso, valoro | UZANTA);
    }
    Ok(FizikaAdreso::nova(valoro & ADRESO_MASKO))
}

/// Ĉu unu paĝo estas atingebla de Ring 3? La bito `U` devas esti ĉe ĉiuj
/// niveloj; skribo postulas ankaŭ la biton `W` ĉe la folio.
fn folio_uzanta(radiko: FizikaAdreso, adreso: u64, skriba: bool) -> bool {
    let virtuala = VirtuaAdreso::nova(adreso);
    let pml4 = ero(radiko, indico(virtuala, 3));
    if pml4 & (PREZENTA | UZANTA) != (PREZENTA | UZANTA) {
        return false;
    }
    let pdpt = ero(FizikaAdreso::nova(pml4 & ADRESO_MASKO), indico(virtuala, 2));
    if pdpt & (PREZENTA | UZANTA) != (PREZENTA | UZANTA) {
        return false;
    }
    let pd = ero(FizikaAdreso::nova(pdpt & ADRESO_MASKO), indico(virtuala, 1));
    if pd & (PREZENTA | UZANTA) != (PREZENTA | UZANTA) {
        return false;
    }
    if pd & GRANDA != 0 {
        return !skriba || pd & SKRIBEBA != 0;
    }
    let pt = ero(FizikaAdreso::nova(pd & ADRESO_MASKO), indico(virtuala, 0));
    if pt & (PREZENTA | UZANTA) != (PREZENTA | UZANTA) {
        return false;
    }
    !skriba || pt & SKRIBEBA != 0
}

fn permesaj_eroj(permesoj: PaĝaPermesoj) -> u64 {
    let mut valoro = 0;
    if permesoj.skribebla {
        valoro |= SKRIBEBA;
    }
    if permesoj.uzanta {
        valoro |= UZANTA;
    }
    if !permesoj.plenumebla {
        valoro |= NE_PLENUMEBLA;
    }
    if !permesoj.kaŝebla {
        valoro |= NEKAŜEBLA;
    }
    valoro
}
