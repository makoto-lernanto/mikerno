//! x86_64 4-nivela paĝado: PML4 → PDPT → PD → PT, kun 4 KiB folioj.
//!
//! La praŝarĝa ŝelko jam mapis la unuajn 4 GiB per 2 MiB paĝoj; ĉi tiu modulo
//! konstruas pli detalan radikan tabelon kaj transdonas ĝin al CR3.

use core::arch::asm;
use core::ptr;

use crate::memoro::KADRA_GRANDO;
use crate::memoro::kadro::{KadraAlokilo, Kadro};
use crate::memoro::virtuala::{
    AdresaSpaco, FizikaAdreso, PaĝaEraro, PaĝaPermesoj, PaĝaTablo, UZANTA_BAZO, UZANTA_FINO,
    VirtuaAdreso,
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

/// Ŝaltu `EFER.NXE` (por la bito 63) kaj `CR0.WP` (por ke Ring 0 ankaŭ respektu
/// nurlegajn paĝojn). Sen `WP` nurlegaj mapoj estus nur konsilo, ne barilo.
pub fn pravalorizi() {
    // SAFETY: `rdmsr`/`wrmsr` kun `EFER_MSR` estas la dokumentita vojo por
    // ŝalti `NXE`; la aliaj bitoj de la MSR estas konservitaj.
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
    // SAFETY: `CR0.WP` (bito 16) estas la sola bito ŝanĝata; la ceteraj,
    // inkluzive `PG` kaj `PE`, restas kiel la praŝarĝa ŝelko lasis ilin.
    unsafe {
        let mut cr0: u64;
        asm!("mov {}, cr0", out(reg) cr0, options(nomem, nostack, preserves_flags));
        cr0 |= 1 << 16;
        asm!("mov cr0, {}", in(reg) cr0, options(nomem, nostack, preserves_flags));
    }
}

/// Po unu bito por ĉiu PML4-ero: `1` se la suba PDPT estis alokita nur por
/// ĉi tiu spaco kaj do rajtas esti liberigita dum `detruu`.
const PRIVATAJ_VORTOJ: usize = 512 / 64;

/// Adresa spaco de la nukleo: posedas la radikan paĝan tabelon.
pub struct X86AdresaSpaco {
    radiko: Kadro,
    privataj_pdptoj: [u64; PRIVATAJ_VORTOJ],
}

/// La fizika radiko, kiun `CR3` nun montras.
pub fn aktiva_radiko() -> FizikaAdreso {
    let cr3: u64;
    // SAFETY: legado de `CR3` ne havas kromefikojn.
    unsafe {
        asm!(
            "mov {}, cr3",
            out(reg) cr3,
            options(nomem, nostack, preserves_flags),
        );
    }
    FizikaAdreso::nova(cr3 & ADRESO_MASKO)
}

/// Ŝaltu `CR3` al la donita radika paĝtabelo.
///
/// # Sekureco
/// `radiko` devas esti fizika adreso de viva `PML4`, kies mapoj inkluzivas la
/// nunan kodon kaj la nunan stakon. La kernaj mapoj estas kunhavataj de ĉiuj
/// procezaj spacoj (`krei_kunhavantan`), do la ŝalto estas sekura ankaŭ meze
/// de kerna funkcio; tamen vokenda kun interrompoj malŝaltitaj, por ke neniu
/// traktilo uzu duonvalidan staton.
pub unsafe fn ŝalti_radikon(radiko: FizikaAdreso) {
    // SAFETY: la alvokanto garantias vivan `PML4` kun atingeblaj kernaj mapoj;
    // `mov cr3` estas la sola necesa operacio (MiKerno ne uzas PCID).
    unsafe {
        asm!(
            "mov cr3, {}",
            in(reg) radiko.adreso(),
            options(nostack, preserves_flags),
        );
    }
}

impl X86AdresaSpaco {
    fn nova(alokilo: &mut KadraAlokilo) -> Result<Self, PaĝaEraro> {
        let kadro = alokilo.aloki().ok_or(PaĝaEraro::Elĉerpita)?;
        nuligi(kadro);
        Ok(Self {
            radiko: kadro,
            privataj_pdptoj: [0; PRIVATAJ_VORTOJ],
        })
    }

    fn radiko(&self) -> FizikaAdreso {
        FizikaAdreso::nova(self.radiko.adreso())
    }

    fn ĉu_privata(&self, indekso: usize) -> bool {
        self.privataj_pdptoj[indekso / 64] & (1 << (indekso % 64)) != 0
    }
}

impl AdresaSpaco for X86AdresaSpaco {
    fn krei(alokilo: &mut KadraAlokilo) -> Result<Self, PaĝaEraro> {
        Self::nova(alokilo)
    }

    fn krei_kunhavantan(&self, alokilo: &mut KadraAlokilo) -> Result<Self, PaĝaEraro> {
        let mut nova = Self::nova(alokilo)?;
        if let Err(eraro) = kopii_kernajn(
            self.radiko(),
            nova.radiko(),
            &mut nova.privataj_pdptoj,
            alokilo,
        ) {
            forigi_kopion(nova.radiko(), self.radiko(), alokilo);
            return Err(eraro);
        }
        Ok(nova)
    }

    fn radika_adreso(&self) -> FizikaAdreso {
        self.radiko()
    }

    fn privataj_pdptoj(&self) -> usize {
        self.privataj_pdptoj
            .iter()
            .map(|vorto| vorto.count_ones() as usize)
            .sum()
    }

    fn ŝalti(&self) {
        // PAE, LME kaj CR0.PG jam estas ŝaltitaj de la praŝarĝa ŝelko.
        // SAFETY: la radika kadro apartenas al ĉi tiu viva spaco, kiu enhavas
        // la nunajn kernajn mapojn.
        unsafe { ŝalti_radikon(self.radiko()) };
    }

    fn detruu(self, alokilo: &mut KadraAlokilo) -> Result<(), PaĝaEraro> {
        // 1. La uzanta fenestro: folioj kaj la tabeloj, kiuj restas malplenaj.
        liberigi_intervalon(self.radiko(), UZANTA_BAZO, UZANTA_FINO, alokilo)?;

        // 2. La privataj PDPT-oj. `kopii_kernajn` alokis ilin nur por ĉi tiu
        // spaco, do ili ne estas kunhavataj kun la nukleo aŭ alia procezo.
        // Post la fenestra liberigo ili rajtas enhavi nur `U=0`-erojn; se
        // uzanta ero restus, ĝi kuŝus ekster la fenestro kaj la spaco estus
        // difektita — tiam ni rifuzas liberigi la PDPT-on anstataŭ riski
        // kernan mapon.
        for indekso in 0..512 {
            if !self.ĉu_privata(indekso) {
                continue;
            }
            let valoro = ero(self.radiko(), indekso);
            if valoro & PREZENTA == 0 {
                continue;
            }
            let pdpt = FizikaAdreso::nova(valoro & ADRESO_MASKO);
            if ĉu_uzanta_tabelo(pdpt) {
                return Err(PaĝaEraro::Konflikto);
            }
            agordi_eron(self.radiko(), indekso, 0);
            alokilo.liberigi(Kadro::el_adreso(pdpt.adreso()));
        }

        // 3. La radika kadro mem.
        alokilo.liberigi(self.radiko);
        Ok(())
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

        let pdpt = suba_tabelo(self.radiko(), indico(virtuala, 3), permesoj.uzanta, alokilo)?;
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
        if adreso < UZANTA_BAZO || fino > UZANTA_FINO {
            return false;
        }

        let mut paĝo = adreso & !(KADRA_GRANDO - 1);
        loop {
            if !folio_uzanta(self.radiko(), paĝo, skriba) {
                return false;
            }
            let sekva = paĝo + KADRA_GRANDO;
            if sekva >= fino {
                return true;
            }
            paĝo = sekva;
        }
    }

    fn permesoj(&self, virtuala: VirtuaAdreso) -> Option<PaĝaPermesoj> {
        let pml4 = ero(self.radiko(), indico(virtuala, 3));
        if pml4 & PREZENTA == 0 {
            return None;
        }
        let pdpt = ero(FizikaAdreso::nova(pml4 & ADRESO_MASKO), indico(virtuala, 2));
        if pdpt & PREZENTA == 0 {
            return None;
        }
        let pd = ero(FizikaAdreso::nova(pdpt & ADRESO_MASKO), indico(virtuala, 1));
        if pd & PREZENTA == 0 {
            return None;
        }
        if pd & GRANDA != 0 {
            return Some(permesoj_el_ero(pd));
        }
        let pt = ero(FizikaAdreso::nova(pd & ADRESO_MASKO), indico(virtuala, 0));
        if pt & PREZENTA == 0 {
            return None;
        }
        Some(permesoj_el_ero(pt))
    }

    fn traduki(&self, virtuala: VirtuaAdreso) -> Option<FizikaAdreso> {
        let pdpt = ero(self.radiko(), indico(virtuala, 3));
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
    // SAFETY: la kadro venas de la bitaro kaj kuŝas en la unuaj 4 GiB, kiuj
    // estas identece mapitaj; la kadro estas okupata de ni, do ni rajtas
    // nuligi ĝian tutan enhavon.
    unsafe {
        ptr::write_bytes(kadro.adreso() as *mut u8, 0, KADRA_GRANDO as usize);
    }
}

/// Kopiu la kernajn mapojn de `fonto` en freŝan `celon`.
///
/// Puraj kernaj subtableoj restas kunhavataj (la sama fizika kadro). PML4-ero
/// kun uzantaj eroj — aŭ kiu enhavas parton de la uzanta fenestro — ricevas
/// privatan PDPT-on, en kiu nur la `U=0`-eroj estas kopiitaj; la indekso estas
/// registrita en `privataj`, por ke `detruu` liberigu ĝuste tiujn kadrojn.
fn kopii_kernajn(
    fonto: FizikaAdreso,
    celo: FizikaAdreso,
    privataj: &mut [u64; PRIVATAJ_VORTOJ],
    alokilo: &mut KadraAlokilo,
) -> Result<(), PaĝaEraro> {
    for indekso in 0..512 {
        let valoro = ero(fonto, indekso);
        if valoro & PREZENTA == 0 {
            continue;
        }
        if valoro & GRANDA != 0 {
            // PML4-ero ne rajtas esti folio; rifuzi anstataŭ diveni.
            return Err(PaĝaEraro::Konflikto);
        }
        let pdpt = FizikaAdreso::nova(valoro & ADRESO_MASKO);
        if !ĉu_uzanta_tabelo(pdpt) && !fenestra_indekso(indekso) {
            // Pura kerna subtableo: kunhavigu la saman fizikan kadron.
            agordi_eron(celo, indekso, valoro);
            continue;
        }

        let kadro = alokilo.aloki().ok_or(PaĝaEraro::Elĉerpita)?;
        nuligi(kadro);
        let privata = FizikaAdreso::nova(kadro.adreso());
        for subindekso in 0..512 {
            let sub = ero(pdpt, subindekso);
            if sub & PREZENTA != 0 && sub & UZANTA == 0 {
                agordi_eron(privata, subindekso, sub);
            }
        }
        // La flagoj de la PML4-ero (inkluzive `U`, kiu malfermas la fenestron)
        // restas; nur la adreso montras al la privata PDPT.
        agordi_eron(celo, indekso, (valoro & !ADRESO_MASKO) | kadro.adreso());
        privataj[indekso / 64] |= 1 << (indekso % 64);
    }
    Ok(())
}

/// Liberigu la privatajn PDPT-kadrojn, kiujn `kopii_kernajn` jam kreis, kaj
/// la freŝan radikon. Kunhavataj eroj restas netuŝitaj.
fn forigi_kopion(celo: FizikaAdreso, fonto: FizikaAdreso, alokilo: &mut KadraAlokilo) {
    for indekso in 0..512 {
        let valoro = ero(celo, indekso);
        if valoro & PREZENTA == 0 {
            continue;
        }
        let fonta = ero(fonto, indekso);
        if (valoro & ADRESO_MASKO) == (fonta & ADRESO_MASKO) {
            continue;
        }
        alokilo.liberigi(Kadro::el_adreso(valoro & ADRESO_MASKO));
        agordi_eron(celo, indekso, 0);
    }
    alokilo.liberigi(Kadro::el_adreso(celo.adreso()));
}

/// Ĉu iu ero de la tabelo estas atingebla de Ring 3?
fn ĉu_uzanta_tabelo(tabelo: FizikaAdreso) -> bool {
    (0..512).any(|indekso| ero(tabelo, indekso) & (PREZENTA | UZANTA) == (PREZENTA | UZANTA))
}

/// Ĉu la PML4-ero `indekso` enhavas parton de la uzanta fenestro? Tia ero
/// neniam estas kunhavata: novaj uzantaj mapoj aperus ankaŭ en la fonta spaco.
fn fenestra_indekso(indekso: usize) -> bool {
    let bazo = indekso as u64 * nivela_amplekso(3);
    let fino = bazo + nivela_amplekso(3);
    bazo < UZANTA_FINO && fino > UZANTA_BAZO
}

/// Liberigu ĉiujn foliojn en `[komenco, fino)` kaj la paĝtabelojn, kiuj restas
/// malplenaj poste. Nur la donita intervalo estas tuŝata, do kernaj mapoj en
/// la sama radika tabelo restas sendifektaj.
fn liberigi_intervalon(
    radiko: FizikaAdreso,
    komenco: u64,
    fino: u64,
    alokilo: &mut KadraAlokilo,
) -> Result<(), PaĝaEraro> {
    if fino <= komenco {
        return Ok(());
    }
    liberigi_nivelon(radiko, 3, 0, komenco, fino, alokilo)
}

/// Amplekso de unu ero en la donita nivelo (0 = PT, 3 = PML4).
const fn nivela_amplekso(nivelo: u32) -> u64 {
    1u64 << (12 + nivelo * 9)
}

fn liberigi_nivelon(
    tabelo: FizikaAdreso,
    nivelo: u32,
    bazo: u64,
    komenco: u64,
    fino: u64,
    alokilo: &mut KadraAlokilo,
) -> Result<(), PaĝaEraro> {
    let amplekso = nivela_amplekso(nivelo);
    for indekso in 0..512 {
        let ero_bazo = bazo + indekso as u64 * amplekso;
        let ero_fino = ero_bazo + amplekso;
        if ero_fino <= komenco || ero_bazo >= fino {
            continue;
        }
        let valoro = ero(tabelo, indekso);
        if valoro & PREZENTA == 0 {
            continue;
        }
        let suba = FizikaAdreso::nova(valoro & ADRESO_MASKO);

        if nivelo == 0 {
            agordi_eron(tabelo, indekso, 0);
            alokilo.liberigi(Kadro::el_adreso(suba.adreso()));
            continue;
        }
        if valoro & GRANDA != 0 {
            // Grandegaj folioj ne aperas en la uzanta fenestro; liberigi ilin
            // kiel unuopajn kadrojn estus danĝera.
            return Err(PaĝaEraro::Konflikto);
        }

        liberigi_nivelon(suba, nivelo - 1, ero_bazo, komenco, fino, alokilo)?;
        // Forigu la suban tabelon nur se ĝi ne plu enhavas ion ajn: tabeloj
        // kun kernaj mapoj restas, eĉ kiam ili parte kuŝas en la intervalo.
        if ĉu_malplena(suba) {
            agordi_eron(tabelo, indekso, 0);
            alokilo.liberigi(Kadro::el_adreso(suba.adreso()));
        }
    }
    Ok(())
}

fn ĉu_malplena(tabelo: FizikaAdreso) -> bool {
    (0..512).all(|indekso| ero(tabelo, indekso) & PREZENTA == 0)
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

/// La permesoj, kiujn folia ero donas al la tradukita paĝo.
fn permesoj_el_ero(ero: u64) -> PaĝaPermesoj {
    PaĝaPermesoj {
        skribebla: ero & SKRIBEBA != 0,
        plenumebla: ero & NE_PLENUMEBLA == 0,
        kaŝebla: ero & NEKAŜEBLA == 0,
        uzanta: ero & UZANTA != 0,
    }
}
