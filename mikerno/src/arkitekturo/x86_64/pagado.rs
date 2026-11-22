//! x86_64 4-nivela paĝado: PML4 → PDPT → PD → PT, kun 4 KiB folioj.
//!
//! La praŝarĝa ŝelko jam mapis la unuajn 4 GiB per 2 MiB paĝoj; ĉi tiu modulo
//! konstruas pli detalan radikan tabelon kaj transdonas ĝin al CR3.

use core::arch::asm;
use core::ptr;

use kernologiko::pagado::x86_64::{
    ADRESO_MASKO, DU_MIB_BAZA_MASKO, GRANDA, PREZENTA, PRIVATAJ_VORTOJ, SKRIBEBA, UZANTA,
    permesaj_eroj, permesoj_el_ero,
};
use kernologiko::pagado::{
    PrivatajSubtabeloj, aliniiĝinta, indeksa_bazo, nivela_amplekso, paĝa_indekso,
};

use crate::memoro::KADRA_GRANDO;
use crate::memoro::kadro::{KadraAlokilo, Kadro};
use crate::memoro::virtuala::{
    AdresaSpaco, FizikaAdreso, PaĝaEraro, PaĝaPermesoj, PaĝaTablo, UZANTA_BAZO, UZANTA_FINO,
    VirtuaAdreso,
};

use super::procesoro;

/// `CR4.PCIDE` (bito 17): proces-kuntekstaj identigiloj.
const PCIDE_BITO: u64 = 1 << 17;

const EFER_MSR: u32 = 0xC000_0080;
const NXE_BITO: u32 = 1 << 11;

/// Ŝaltu `EFER.NXE` (por la bito 63) kaj `CR0.WP` (por ke Ring 0 ankaŭ respektu
/// nurlegajn paĝojn). Sen `WP` nurlegaj mapoj estus nur konsilo, ne barilo.
/// Ankaŭ malŝaltu `CR4.PCIDE`: la praŝarĝila ŝelko (ekz. Limine) povas lasi
/// ĝin ŝaltita, sed la `CR3`-modelo de MiKerno skribas nudan radikon.
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
    // Se `CR3` portas ne-nulan PCID-on, tiuj bitoj fariĝus rezervitaj tuj post
    // la malŝalto de `PCIDE`; do unue reŝargu `CR3` kun nula PCID, poste
    // forigu la biton. La dua paŝo ankaŭ purigas la tutan TLB-on.
    // SAFETY: nur `CR4.PCIDE` estas forigata; CPL estas 0 kaj `CR0.PG=1`.
    unsafe {
        let cr4: u64;
        asm!("mov {}, cr4", out(reg) cr4, options(nomem, nostack, preserves_flags));
        if cr4 & PCIDE_BITO != 0 {
            let cr3: u64;
            asm!("mov {}, cr3", out(reg) cr3, options(nomem, nostack, preserves_flags));
            asm!(
                "mov cr3, {}",
                in(reg) cr3 & ADRESO_MASKO,
                options(nostack, preserves_flags),
            );
            asm!(
                "mov cr4, {}",
                in(reg) cr4 & !PCIDE_BITO,
                options(nostack, preserves_flags),
            );
        }
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

/// Adresa spaco de la nukleo: posedas la radikan paĝan tabelon.
pub struct X86AdresaSpaco {
    radiko: Kadro,
    /// Po unu bito por ĉiu PML4-ero: la suba PDPT estis alokita nur por ĉi
    /// tiu spaco kaj do rajtas esti liberigita dum `detruu`.
    privataj_pdptoj: PrivatajSubtabeloj<PRIVATAJ_VORTOJ>,
}

/// La radika paĝtabelo enhavita en `CR3`-valoro.
///
/// MiKerno tenas `CR4.PCIDE` malŝaltita, do la malaltaj 12 bitoj estas
/// rezervitaj kaj nulo (`ŝalti_radikon` haltigas la kernon alie). La masko
/// ankaŭ igas la komparon en `procezo::aktivigi` sendependa de eventualaj
/// flagaj bitoj.
pub const fn radiko_ero(cr3: u64) -> u64 {
    cr3 & ADRESO_MASKO
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
    FizikaAdreso::nova(radiko_ero(cr3))
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
    // La tuta modelo de `aktivigi` (kompari la aktivan radikon, ŝalti nur ĉe
    // diferenco) supozas, ke `mov cr3` ŝanĝas unu TLB-spacon. Kun PCID la
    // malaltaj 12 bitoj estus etikedo, do ni rifuzas daŭrigi anstataŭ silente
    // ŝanĝi la signifon.
    if !procesoro::ne_pcide() {
        crate::skribu!("\n[kerno] CR3: CR4.PCID estas ŝaltita; MiKerno postulas PCID=0; haltas\n");
        crate::arkitekturo::halti();
    }
    // SAFETY: la alvokanto garantias vivan `PML4` kun atingeblaj kernaj mapoj;
    // `mov cr3` estas la sola necesa operacio (PCID estas malŝaltita, vidu
    // supre).
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
            privataj_pdptoj: PrivatajSubtabeloj::nova(),
        })
    }

    fn radiko(&self) -> FizikaAdreso {
        FizikaAdreso::nova(self.radiko.adreso())
    }

    /// La folia ero, kiu tradukas `virtuala`, kaj ĝia nivelo (1 = 2 MiB-folio,
    /// 0 = 4 KiB-folio).
    fn folia_ero(&self, virtuala: VirtuaAdreso) -> Option<(u64, u32)> {
        let mut tabelo = self.radiko();
        for nivelo in (1..=3).rev() {
            let valoro = ero(tabelo, paĝa_indekso(virtuala.adreso(), nivelo));
            if valoro & PREZENTA == 0 {
                return None;
            }
            if nivelo == 1 && valoro & GRANDA != 0 {
                return Some((valoro, 1));
            }
            tabelo = FizikaAdreso::nova(valoro & ADRESO_MASKO);
        }
        let folio = ero(tabelo, paĝa_indekso(virtuala.adreso(), 0));
        (folio & PREZENTA != 0).then_some((folio, 0))
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

    fn privataj_subtabeloj(&self) -> usize {
        self.privataj_pdptoj.nombro()
    }

    fn ŝalti(&self) {
        // PAE, LME kaj CR0.PG jam estas ŝaltitaj de la praŝarĝa ŝelko.
        // SAFETY: la radika kadro apartenas al ĉi tiu viva spaco, kiu enhavas
        // la nunajn kernajn mapojn.
        unsafe { ŝalti_radikon(self.radiko()) };
    }

    fn detruu(mut self, alokilo: &mut KadraAlokilo) -> Result<(), PaĝaEraro> {
        // 1. La uzanta fenestro: folioj kaj la tabeloj, kiuj restas malplenaj.
        liberigi_intervalon(self.radiko(), UZANTA_BAZO, UZANTA_FINO, alokilo)?;

        // 2. La privataj PDPT-oj. `kopii_kernajn` alokis ilin nur por ĉi tiu
        // spaco, do ili ne estas kunhavataj kun la nukleo aŭ alia procezo.
        // Post la fenestra liberigo ili rajtas enhavi nur `U=0`-erojn; se
        // uzanta ero restus, ĝi kuŝus ekster la fenestro kaj la spaco estus
        // difektita — tiam ni rifuzas liberigi la PDPT-on anstataŭ riski
        // kernan mapon.
        let privataj = self.privataj_pdptoj;
        for indekso in privataj.indeksoj() {
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
            self.privataj_pdptoj.malregistri(indekso);
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
        if !aliniiĝinta(virtuala.adreso()) || !aliniiĝinta(fizika.adreso()) {
            return Err(PaĝaEraro::NevalidaAdreso);
        }

        let pdpt = suba_tabelo(
            self.radiko(),
            paĝa_indekso(virtuala.adreso(), 3),
            permesoj.uzanta,
            alokilo,
        )?;
        let pd = suba_tabelo(
            pdpt,
            paĝa_indekso(virtuala.adreso(), 2),
            permesoj.uzanta,
            alokilo,
        )?;
        let pt = suba_tabelo(
            pd,
            paĝa_indekso(virtuala.adreso(), 1),
            permesoj.uzanta,
            alokilo,
        )?;

        let indekso = paĝa_indekso(virtuala.adreso(), 0);
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
        self.folia_ero(virtuala)
            .map(|(ero, _)| permesoj_el_ero(ero))
    }

    fn traduki(&self, virtuala: VirtuaAdreso) -> Option<FizikaAdreso> {
        let (ero, nivelo) = self.folia_ero(virtuala)?;
        if nivelo == 1 {
            return Some(FizikaAdreso::nova(
                (ero & DU_MIB_BAZA_MASKO) | (virtuala.adreso() & 0x1F_FFFF),
            ));
        }
        Some(FizikaAdreso::nova(
            (ero & ADRESO_MASKO) | (virtuala.adreso() & 0xFFF),
        ))
    }
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
    privataj: &mut PrivatajSubtabeloj<PRIVATAJ_VORTOJ>,
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
        privataj.registri(indekso);
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
    let bazo = indeksa_bazo(3, indekso);
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
    let pml4 = ero(radiko, paĝa_indekso(adreso, 3));
    if pml4 & (PREZENTA | UZANTA) != (PREZENTA | UZANTA) {
        return false;
    }
    let pdpt = ero(
        FizikaAdreso::nova(pml4 & ADRESO_MASKO),
        paĝa_indekso(adreso, 2),
    );
    if pdpt & (PREZENTA | UZANTA) != (PREZENTA | UZANTA) {
        return false;
    }
    let pd = ero(
        FizikaAdreso::nova(pdpt & ADRESO_MASKO),
        paĝa_indekso(adreso, 1),
    );
    if pd & (PREZENTA | UZANTA) != (PREZENTA | UZANTA) {
        return false;
    }
    if pd & GRANDA != 0 {
        return !skriba || pd & SKRIBEBA != 0;
    }
    let pt = ero(
        FizikaAdreso::nova(pd & ADRESO_MASKO),
        paĝa_indekso(adreso, 0),
    );
    if pt & (PREZENTA | UZANTA) != (PREZENTA | UZANTA) {
        return false;
    }
    !skriba || pt & SKRIBEBA != 0
}

#[cfg(kani)]
mod kani_proofs {
    //! Pruvoj por la `CR3`-disciplino: la radiko estas paĝ-aliniiĝinta, kaj por
    //! fizika `CR3`-valoro la masko forprenas ĝuste la malaltajn 12 bitojn.

    use super::{ADRESO_MASKO, PREZENTA, radiko_ero};
    use crate::memoro::virtuala::PaĝaPermesos;
    use kernologiko::pagado::x86_64::{
        NE_PLENUMEBLA, PRIVATAJ_VORTOJ, SKRIBEBA, permesaj_eroj, permesoj_el_ero,
    };
    use kernologiko::pagado::{
        EROJ, PrivatajSubtabeloj, aliniiĝinta, indeksa_bazo, nivela_amplekso, paĝa_indekso,
    };

    #[kani::proof]
    fn cr3_radiko_estas_paĝe_aliniita() {
        let cr3: u64 = kani::any();
        let radiko = radiko_ero(cr3);
        assert_eq!(radiko & 0xFFF, 0);
        assert_eq!(radiko, cr3 & ADRESO_MASKO);
        assert!(radiko <= ADRESO_MASKO);
    }

    #[kani::proof]
    fn cr3_radiko_forprenas_nur_la_malaltajn_bitojn() {
        let cr3: u64 = kani::any();
        // Nur fizika adreso: bitoj 63:52 estas nulaj, do la supra maskoparto
        // ne perdas informon.
        kani::assume(cr3 >> 52 == 0);
        let radiko = radiko_ero(cr3);
        assert_eq!(radiko, cr3 & !0xFFF_u64);
        assert_eq!(radiko >> 12, cr3 >> 12);
        if cr3 & 0xFFF == 0 {
            assert_eq!(radiko, cr3);
        }
    }

    fn kontrolu_malkomponon(nivelo: u32, adreso: u64) {
        let indekso = paĝa_indekso(adreso, nivelo);
        assert!(indekso < EROJ);
        let bazo = indeksa_bazo(nivelo, indekso);
        let amplekso = nivela_amplekso(nivelo);
        assert_eq!(bazo, adreso & (0x1FFu64 << (12 + 9 * nivelo)));
        let fenestro = amplekso * EROJ as u64;
        assert_eq!(adreso & (fenestro - 1), bazo + (adreso & (amplekso - 1)));
    }

    #[kani::proof]
    fn adreso_malkomponiĝas_en_kvar_nivelojn() {
        let adreso: u64 = kani::any();
        // La kvar niveloj kovras la kanonikan 48-bitan virtualan spacon.
        kani::assume(adreso < (1u64 << 48));
        kontrolu_malkomponon(0, adreso);
        kontrolu_malkomponon(1, adreso);
        kontrolu_malkomponon(2, adreso);
        kontrolu_malkomponon(3, adreso);
    }

    #[kani::proof]
    fn indekso_kaj_bazo_estas_inversaj() {
        let indekso: u16 = kani::any();
        kani::assume(indekso < 512);
        let indekso = indekso as usize;
        assert_eq!(paĝa_indekso(indeksa_bazo(0, indekso), 0), indekso);
        assert_eq!(paĝa_indekso(indeksa_bazo(1, indekso), 1), indekso);
        assert_eq!(paĝa_indekso(indeksa_bazo(2, indekso), 2), indekso);
        assert_eq!(paĝa_indekso(indeksa_bazo(3, indekso), 3), indekso);
    }

    fn iuj_permesoj() -> PaĝaPermesos {
        PaĝaPermesos {
            skribebla: kani::any(),
            plenumebla: kani::any(),
            kaŝebla: kani::any(),
            uzanta: kani::any(),
        }
    }

    #[kani::proof]
    fn permesoj_kaj_eroj_estas_reverseblaj() {
        let permesoj = iuj_permesoj();
        let eroj = permesaj_eroj(permesoj);
        let malkodita = permesoj_el_ero(eroj);
        assert_eq!(malkodita.skribebla, permesoj.skribebla);
        assert_eq!(malkodita.plenumebla, permesoj.plenumebla);
        assert_eq!(malkodita.kaŝebla, permesoj.kaŝebla);
        assert_eq!(malkodita.uzanta, permesoj.uzanta);
        assert_eq!(eroj & SKRIBEBA != 0, permesoj.skribebla);
        assert_eq!(eroj & NE_PLENUMEBLA != 0, !permesoj.plenumebla);
    }

    fn sen_wx(permesoj: PaĝaPermesos) {
        assert!(!(permesoj.skribebla && permesoj.plenumebla));
        let eroj = permesaj_eroj(permesoj);
        assert!(!(eroj & SKRIBEBA != 0 && eroj & NE_PLENUMEBLA == 0));
    }

    #[kani::proof]
    fn politikaj_permesoj_respektas_wx() {
        sen_wx(PaĝaPermesos::SKRIBA);
        sen_wx(PaĝaPermesos::SKRIBA_SENKAŜE);
        sen_wx(PaĝaPermesos::KERNA_KODO);
        sen_wx(PaĝaPermesos::KERNA_LEGA);
        sen_wx(PaĝaPermesos::KERNA_DATUMOJ);
        sen_wx(PaĝaPermesos::UZANTA_DATUMOJ);
        sen_wx(PaĝaPermesos::UZANTA_LEGA);
        sen_wx(PaĝaPermesos::UZANTA_KODO);
    }

    #[kani::proof]
    fn ero_konservas_la_aliniiĝon_de_la_fizika_adreso() {
        let fizika: u64 = kani::any();
        kani::assume(fizika < (1u64 << 52));
        kani::assume(aliniiĝinta(fizika));
        let ero = fizika | PREZENTA | permesaj_eroj(iuj_permesoj());
        assert_eq!(ero & ADRESO_MASKO, fizika);
        assert!(aliniiĝinta(ero & ADRESO_MASKO));
    }

    #[kani::proof]
    fn privataj_subtabeloj_estas_plene_liberigitaj() {
        let mut registro = PrivatajSubtabeloj::<PRIVATAJ_VORTOJ>::nova();
        let indekso: u16 = kani::any();
        kani::assume(indekso < 512);
        let indekso = indekso as usize;
        registro.registri(indekso);
        registro.registri(indekso);
        assert!(registro.estas(indekso));
        assert_eq!(registro.nombro(), 1);
        registro.malregistri(indekso);
        assert!(!registro.estas(indekso));
        assert_eq!(registro.nombro(), 0);
        assert!(registro.malplena());
    }
}
