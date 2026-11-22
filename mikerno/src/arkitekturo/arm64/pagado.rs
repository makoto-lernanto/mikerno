//! MMU de `arm64`: frua identeca mapo kaj la per-proceza `TTBR0`-spaco.
//!
//! La frua mapo (`pravalorizi`) konstruas du statikajn tabelojn, kiuj mapas la
//! unuajn 4 GiB per 1 GiB-blokoj kaj permesas al la unua Rust-kodo ruliĝi.
//! Post kiam la kadro-alokilo pretas, `ArmAdresaSpaco` konstruas dinamikan
//! spacon: la kernaj identecaj mapoj restas kunhavataj inter ĉiuj procezoj,
//! dum la uzanta fenestro `[UZANTA_BAZO, UZANTA_FINO)` ricevas privatajn
//! `L2`/`L3`-tabelojn. Unu `TTBR0_EL1` apartenas al unu procezo, same kiel unu
//! `CR3` sur `x86_64`.

use core::arch::asm;
use core::ptr::{read_volatile, write_bytes, write_volatile};

use kernologiko::pagado::arm64::{
    ADRESA_MASKO, AF, ATTR_APARATO, BLOKA_MASKO, BLOKO, DU_MIB_MASKO, PAĜO, PXN, SH_ENA, TABELO,
    UXN, atributo, el_descriptor,
};
use kernologiko::pagado::{aliniiĝinta, paĝa_indekso};

use super::procesoro;
use crate::memoro::kadro::{KadraAlokilo, Kadro};
use crate::memoro::virtuala::{
    AdresaSpaco, FizikaAdreso, PaĝaEraro, PaĝaPermesoj, PaĝaTablo, UZANTA_BAZO, VirtuaAdreso,
    kernaj_limoj,
};
use crate::memoro::{KADRA_GRANDO, MemoraMapo};

const TTBR_MASKO: u64 = 0x0000_FFFF_FFFF_FFFF;

/// La bazo de la kerna `Image`; la frua ligilo fiksas ĝin kaj `build.rs`
/// liveras `boot/linker-arm64.ld`, kiu asertas la saman valoron.
pub const KERNA_BAZO: u64 = 0x4008_0000;

/// La unua nivelo de la frua mapo: la unuaj 4 GiB per 1 GiB-blokoj.
static mut L0: Tabelo = Tabelo([0; 512]);
static mut L1: Tabelo = Tabelo([0; 512]);

#[repr(C, align(4096))]
struct Tabelo([u64; 512]);

const fn normala(bazo: u64) -> u64 {
    bazo | BLOKO | AF | SH_ENA | UXN
}

const fn aparato(bazo: u64) -> u64 {
    bazo | BLOKO | AF | ATTR_APARATO | PXN | UXN
}

/// Instalu la frunan identecan mapon; la MMU ekfunkcias antaŭ ol la unua
/// Rust-kodo tuŝas la stakon aŭ la seriajn registrojn.
pub fn pravalorizi() {
    let tabelo0 = &raw mut L0 as *mut u64;
    let tabelo1 = &raw mut L1 as *mut u64;
    let bazo0 = tabelo0 as u64;
    let bazo1 = tabelo1 as u64;
    // SAFETY: `L0` kaj `L1` estas niaj propraj, 4 KiB-aliniiĝintaj statikaj
    // tabeloj; la skriboj okazas antaŭ ol la MMU estas ŝaltita, do ili estas
    // fizikaj.
    unsafe {
        write_volatile(tabelo0, bazo1 | TABELO); // L0[0] → L1
        write_volatile(tabelo1, aparato(0));
        for indekso in 1..4u64 {
            write_volatile(tabelo1.add(indekso as usize), normala(indekso << 30));
        }

        // MAIR: atributo 0 = normala WB, atributo 1 = Device-nGnRnE.
        asm!("msr mair_el1, {}", in(reg) 0xffu64, options(nomem, nostack));

        // TCR: 48-bitaj adresoj (L0), 4 KiB-granuloj, inner WB, EPD1 (sen
        // TTBR1). IPS=2: 40-bitaj fizikaj adresoj, kiujn cortex-a53 havas.
        let tcr = 16u64 | (1 << 8) | (1 << 10) | (3 << 12) | (1 << 23) | (2 << 32);
        asm!("msr tcr_el1, {}", in(reg) tcr, options(nomem, nostack));
        asm!("msr ttbr0_el1, {}", in(reg) bazo0, options(nomem, nostack));
        asm!("dsb ish", "isb", options(nomem, nostack));
        asm!("tlbi vmalle1", "dsb ish", "isb", options(nomem, nostack));

        let mut sctlr: u64;
        asm!("mrs {}, sctlr_el1", out(reg) sctlr, options(nomem, nostack));
        sctlr |= (1 << 0) | (1 << 2) | (1 << 12); // M | C | I
        // SPAN = 0: ĉiu escepta eniro al `EL1` fermas la `PAN`-fenestron, do
        // la kerno legas uzantajn bufrojn nur per `StacGardo`.
        sctlr &= !(1 << 23);
        asm!("msr sctlr_el1, {}", in(reg) sctlr, options(nomem, nostack));
        asm!("isb", options(nomem, nostack));
    }
    // Antaŭ la unua escepto `PAN` ankoraŭ havas sian pravaloran staton; fermu
    // la fenestron ankaŭ ĉi tie.
    if procesoro::pan_subtenata() {
        procesoro::fermi_pan();
    }
}

/// La dinamika identeca spaco de la kerno kaj de ĉiu uzanta procezo.
pub struct ArmAdresaSpaco {
    radiko: u64,
    privataj: usize,
}

impl ArmAdresaSpaco {
    fn nova_tabelo(alokilo: &mut KadraAlokilo, privataj: &mut usize) -> Result<u64, PaĝaEraro> {
        let kadro = alokilo.aloki().ok_or(PaĝaEraro::Elĉerpita)?;
        let bazo = kadro.adreso();
        // SAFETY: la kadro estas freŝe alokita, 4 KiB-granda kaj identece
        // mapita; neniq alia referenco ekzistas.
        unsafe { write_bytes(bazo as *mut u8, 0, KADRA_GRANDO as usize) };
        *privataj += 1;
        Ok(bazo)
    }

    /// Redonu la subtableon, kiun montras la ero `parent`; se la ero estas
    /// malplena, aloku kaj registru novan tabelon. Bloko aŭ folio konfliktas.
    fn subtableo(
        parent: *mut u64,
        alokilo: &mut KadraAlokilo,
        privataj: &mut usize,
    ) -> Result<*mut u64, PaĝaEraro> {
        // SAFETY: `parent` montras al ero de nia propra tabelo.
        let ero = unsafe { read_volatile(parent) };
        if ero & TABELO == TABELO {
            return Ok((ero & ADRESA_MASKO) as *mut u64);
        }
        if ero & TABELO != 0 {
            return Err(PaĝaEraro::Konflikto);
        }
        let tabelo = Self::nova_tabelo(alokilo, privataj)?;
        // SAFETY: `parent` estas viva ero; la nova tabelo restas vivanta dum
        // la tuta spaco.
        unsafe { write_volatile(parent, tabelo | TABELO) };
        Ok(tabelo as *mut u64)
    }
}

fn legi_eron(ero: *const u64) -> u64 {
    // SAFETY: la montrilo montras al ero de la radika tabelo aŭ de unu el ĝiaj
    // subtabeloj, kiuj vivas dum la spaco.
    unsafe { read_volatile(ero) }
}

fn skribi_eron(ero: *mut u64, valoro: u64) {
    // SAFETY: sama kiel `legi_eron`; la skribo okazas nur en la koncerna spaco.
    unsafe { write_volatile(ero, valoro) };
}

fn ero(bazo: *const u64, indekso: usize) -> u64 {
    // SAFETY: la bazmontrilo montras al nia tabelo kaj `indekso` estas en 0..512.
    legi_eron(unsafe { bazo.add(indekso) })
}

fn ero_adreso(bazo: *mut u64, indekso: usize) -> *mut u64 {
    // SAFETY: sama kiel `ero`; la montrilo restas ene de la tabelo.
    unsafe { bazo.add(indekso) }
}

fn skribi_eron_en(bazo: *mut u64, indekso: usize, valoro: u64) {
    // SAFETY: sama kiel `ero`; la skribo okazas nur en la koncerna spaco.
    skribi_eron(unsafe { bazo.add(indekso) }, valoro);
}

impl PaĝaTablo for ArmAdresaSpaco {
    fn mapigi(
        &mut self,
        virtuala: VirtuaAdreso,
        fizika: FizikaAdreso,
        permesoj: PaĝaPermesoj,
        alokilo: &mut KadraAlokilo,
    ) -> Result<(), PaĝaEraro> {
        let va = virtuala.adreso();
        let pa = fizika.adreso();
        if !aliniiĝinta(va) || !aliniiĝinta(pa) {
            return Err(PaĝaEraro::NevalidaAdreso);
        }
        let radiko = self.radiko;
        let l0 = radiko as *mut u64;
        let l1 = Self::subtableo(
            ero_adreso(l0, paĝa_indekso(va, 3)),
            alokilo,
            &mut self.privataj,
        )?;
        let l2 = Self::subtableo(
            ero_adreso(l1, paĝa_indekso(va, 2)),
            alokilo,
            &mut self.privataj,
        )?;
        let l3 = Self::subtableo(
            ero_adreso(l2, paĝa_indekso(va, 1)),
            alokilo,
            &mut self.privataj,
        )?;
        let folio = ero_adreso(l3, paĝa_indekso(va, 0));
        if legi_eron(folio) & 1 != 0 {
            return Err(PaĝaEraro::Konflikto);
        }
        skribi_eron(folio, pa | PAĜO | atributo(permesoj));
        Ok(())
    }

    fn traduki(&self, virtuala: VirtuaAdreso) -> Option<FizikaAdreso> {
        let va = virtuala.adreso();
        let l0 = self.radiko as *const u64;
        let e0 = ero(l0, paĝa_indekso(va, 3));
        if e0 & TABELO != TABELO {
            return None;
        }
        let l1 = (e0 & ADRESA_MASKO) as *const u64;
        let e1 = ero(l1, paĝa_indekso(va, 2));
        if e1 & BLOKO == BLOKO && e1 & TABELO != TABELO {
            return Some(FizikaAdreso::nova(
                (e1 & BLOKA_MASKO) + (va & ((1 << 30) - 1)),
            ));
        }
        if e1 & TABELO != TABELO {
            return None;
        }
        let l2 = (e1 & ADRESA_MASKO) as *const u64;
        let e2 = ero(l2, paĝa_indekso(va, 1));
        if e2 & BLOKO == BLOKO && e2 & TABELO != TABELO {
            return Some(FizikaAdreso::nova(
                (e2 & DU_MIB_MASKO) + (va & ((1 << 21) - 1)),
            ));
        }
        if e2 & TABELO != TABELO {
            return None;
        }
        let l3 = (e2 & ADRESA_MASKO) as *const u64;
        let e3 = ero(l3, paĝa_indekso(va, 0));
        if e3 & 1 == 0 {
            return None;
        }
        Some(FizikaAdreso::nova(
            (e3 & ADRESA_MASKO) + (va & (KADRA_GRANDO - 1)),
        ))
    }

    fn permesoj(&self, virtuala: VirtuaAdreso) -> Option<PaĝaPermesoj> {
        let va = virtuala.adreso();
        let l0 = self.radiko as *const u64;
        let e0 = ero(l0, paĝa_indekso(va, 3));
        if e0 & TABELO != TABELO {
            return None;
        }
        let l1 = (e0 & ADRESA_MASKO) as *const u64;
        let e1 = ero(l1, paĝa_indekso(va, 2));
        if e1 & TABELO != TABELO {
            return (e1 & 1 != 0).then(|| el_descriptor(e1));
        }
        let l2 = (e1 & ADRESA_MASKO) as *const u64;
        let e2 = ero(l2, paĝa_indekso(va, 1));
        if e2 & TABELO != TABELO {
            return (e2 & 1 != 0).then(|| el_descriptor(e2));
        }
        let l3 = (e2 & ADRESA_MASKO) as *const u64;
        let e3 = ero(l3, paĝa_indekso(va, 0));
        (e3 & 1 != 0).then(|| el_descriptor(e3))
    }

    fn kontroli_uzantan(&self, adreso: u64, longo: usize, skriba: bool) -> bool {
        if longo == 0 {
            return true;
        }
        let Some(fino) = adreso.checked_add(longo as u64) else {
            return false;
        };
        let mut paĝo = adreso & !(KADRA_GRANDO - 1);
        while paĝo < fino {
            let virtuala = VirtuaAdreso::nova(paĝo);
            let Some(permesoj) = self.permesoj(virtuala) else {
                return false;
            };
            if !permesoj.uzanta || (skriba && !permesoj.skribebla) {
                return false;
            }
            paĝo += KADRA_GRANDO;
        }
        true
    }
}

impl AdresaSpaco for ArmAdresaSpaco {
    fn krei(alokilo: &mut KadraAlokilo) -> Result<Self, PaĝaEraro> {
        let mut privataj = 0;
        let radiko = Self::nova_tabelo(alokilo, &mut privataj)?;
        Ok(Self { radiko, privataj })
    }

    fn krei_kunhavantan(&self, alokilo: &mut KadraAlokilo) -> Result<Self, PaĝaEraro> {
        let mut privataj = 0;
        let radiko = Self::nova_tabelo(alokilo, &mut privataj)?;
        let l1 = Self::nova_tabelo(alokilo, &mut privataj)?;
        // La radika ero `0` montras al la dua nivelo, kiu kovras la unuajn
        // 512 GiB. El ĝi la unuaj kvar `L1`-eroj kovras `0..4 GiB`, la kernan
        // identecan mapon. Ili estas tabeloj aŭ blokoj de la kerna spaco kaj
        // restas kunhavataj; la uzanta fenestro (`L1[4..]`) restas malplena.
        let fonto = ero(self.radiko as *const u64, 0);
        if fonto & TABELO != TABELO {
            return Err(PaĝaEraro::Konflikto);
        }
        let fonto = (fonto & ADRESA_MASKO) as *const u64;
        let celo = l1 as *mut u64;
        for indekso in 0..4 {
            let ero = ero(fonto, indekso);
            skribi_eron_en(celo, indekso, ero);
        }
        skribi_eron(radiko as *mut u64, l1 | TABELO);
        Ok(Self { radiko, privataj })
    }

    fn radika_adreso(&self) -> FizikaAdreso {
        FizikaAdreso::nova(self.radiko)
    }

    fn privataj_subtabeloj(&self) -> usize {
        self.privataj
    }

    fn ŝalti(&self) {
        // SAFETY: `radiko` estas 4 KiB-aliniiĝinta fizika radiko, kiu restas
        // vivanta dum la spaco; la barieroj certigas, ke la tabeloj videblas
        // antaŭ ol la MMU uzas ilin.
        unsafe {
            asm!("msr ttbr0_el1, {}", in(reg) self.radiko, options(nomem, nostack));
            asm!(
                "dsb ish",
                "tlbi vmalle1",
                "dsb ish",
                "isb",
                options(nomem, nostack)
            );
        }
    }

    fn detruu(self, alokilo: &mut KadraAlokilo) -> Result<(), PaĝaEraro> {
        let l0 = self.radiko as *mut u64;
        let e0 = legi_eron(l0);
        if e0 & TABELO == TABELO {
            let l1 = (e0 & ADRESA_MASKO) as *mut u64;
            for indekso in 4..512 {
                let e1 = ero(l1, indekso);
                if e1 & TABELO == TABELO {
                    liberigi_l2((e1 & ADRESA_MASKO) as *mut u64, alokilo);
                } else if e1 & BLOKO == BLOKO && e1 & TABELO != TABELO {
                    liberigi_kadrojn(e1 & BLOKA_MASKO, 1 << 30, alokilo);
                }
            }
            alokilo.liberigi(Kadro::el_adreso(l1 as u64));
        }
        alokilo.liberigi(Kadro::el_adreso(self.radiko));
        Ok(())
    }

    fn idente_mapigi(
        &mut self,
        mapo: &MemoraMapo,
        alokilo: &mut KadraAlokilo,
    ) -> Result<(), PaĝaEraro> {
        self.mapigi_intervalon(
            VirtuaAdreso::nova(0),
            VirtuaAdreso::nova(0x10_0000),
            PaĝaPermesoj::KERNA_DATUMOJ,
            alokilo,
        )?;
        let limoj = kernaj_limoj();
        self.mapigi_intervalon(
            VirtuaAdreso::nova(limoj.antaŭkodo.0),
            VirtuaAdreso::nova(limoj.antaŭkodo.1),
            PaĝaPermesoj::KERNA_LEGA,
            alokilo,
        )?;
        self.mapigi_intervalon(
            VirtuaAdreso::nova(limoj.kodo.0),
            VirtuaAdreso::nova(limoj.kodo.1),
            PaĝaPermesoj::KERNA_KODO,
            alokilo,
        )?;
        self.mapigi_intervalon(
            VirtuaAdreso::nova(limoj.rodata.0),
            VirtuaAdreso::nova(limoj.rodata.1),
            PaĝaPermesoj::KERNA_LEGA,
            alokilo,
        )?;
        self.mapigi_intervalon(
            VirtuaAdreso::nova(limoj.datumoj.0),
            VirtuaAdreso::nova(limoj.datumoj.1),
            PaĝaPermesoj::KERNA_DATUMOJ,
            alokilo,
        )?;

        for regiono in mapo.regionoj().iter().filter(|regiono| regiono.uzebla()) {
            self.mapigi_intervalon(
                VirtuaAdreso::nova(regiono.komenco),
                VirtuaAdreso::nova(regiono.fino()),
                PaĝaPermesoj::KERNA_DATUMOJ,
                alokilo,
            )?;
        }

        // MMIO de QEMU `virt`: GIC-distribuilo, GIC-CPU-interfaco, PL011 kaj
        // `fw_cfg` (la adreso venas el la FDT, kun fiksa rezerva valoro).
        let fw_cfg = super::fw_cfg::bazo();
        for (komenco, fino) in [
            (0x0800_0000u64, 0x0800_1000u64),
            (0x0801_0000u64, 0x0801_2000u64),
            (0x0900_0000u64, 0x0900_1000u64),
            (fw_cfg, fw_cfg + 0x1000),
        ] {
            self.mapigi_intervalon(
                VirtuaAdreso::nova(komenco),
                VirtuaAdreso::nova(fino),
                PaĝaPermesoj::SKRIBA_SENKAŜE,
                alokilo,
            )?;
        }
        Ok(())
    }
}

fn liberigi_l2(l2: *mut u64, alokilo: &mut KadraAlokilo) {
    for indekso in 0..512 {
        // SAFETY: `l2` estas privata, viva `L2`-tabelo de la spaco.
        let ero = unsafe { read_volatile(l2.add(indekso)) };
        if ero & TABELO == TABELO {
            liberigi_l3((ero & ADRESA_MASKO) as *mut u64, alokilo);
        } else if ero & BLOKO == BLOKO && ero & TABELO != TABELO {
            liberigi_kadrojn(ero & DU_MIB_MASKO, 1 << 21, alokilo);
        }
    }
    alokilo.liberigi(Kadro::el_adreso(l2 as u64));
}

fn liberigi_l3(l3: *mut u64, alokilo: &mut KadraAlokilo) {
    for indekso in 0..512 {
        // SAFETY: `l3` estas privata, viva `L3`-tabelo de la spaco.
        let ero = unsafe { read_volatile(l3.add(indekso)) };
        if ero & 1 != 0 {
            alokilo.liberigi(Kadro::el_adreso(ero & ADRESA_MASKO));
        }
    }
    alokilo.liberigi(Kadro::el_adreso(l3 as u64));
}

fn liberigi_kadrojn(komenco: u64, longo: u64, alokilo: &mut KadraAlokilo) {
    let mut adreso = komenco;
    while adreso < komenco + longo {
        alokilo.liberigi(Kadro::el_adreso(adreso));
        adreso += KADRA_GRANDO;
    }
}

/// La fizika radiko, kiun `TTBR0_EL1` nun montras.
pub fn aktiva_radiko() -> u64 {
    let valoro: u64;
    // SAFETY: legado de `TTBR0_EL1` ne havas kromefikojn.
    unsafe { asm!("mrs {}, ttbr0_el1", out(reg) valoro, options(nomem, nostack)) };
    valoro & TTBR_MASKO & ADRESA_MASKO
}

/// Ŝaltu `TTBR0_EL1` al la donita radika tabelo.
///
/// # Sekureco
/// `radiko` devas esti viva, 4 KiB-aliniiĝinta radika tabelo, kiu mapas la
/// nunan kodon, stakon kaj la renkontotajn datumojn.
pub unsafe fn ŝalti_radikon(radiko: u64) {
    // SAFETY: la alvokanto garantias la antaŭkondiĉojn; la barieroj certigas
    // videblon de la tabeloj kaj malplenigas la TLB-on.
    unsafe {
        asm!("msr ttbr0_el1, {}", in(reg) radiko, options(nomem, nostack));
        asm!(
            "dsb ish",
            "tlbi vmalle1",
            "dsb ish",
            "isb",
            options(nomem, nostack)
        );
    }
}

/// La uzanta fenestro devas kuŝi super la kerna identeca mapo.
const _: () = assert!(UZANTA_BAZO >= 0x1_0000_0000);

#[cfg(kani)]
mod kani_proofs {
    //! Pruvoj por la arm64-priskriboj: permesoj estas reverseblaj, la politiko
    //! respektas W^X, kaj la folia adreso restas aliniiĝinta.

    use super::{ADRESA_MASKO, PAĜO, atributo, el_descriptor};
    use crate::memoro::virtuala::PaĝaPermesos;
    use kernologiko::pagado::aliniiĝinta;

    fn iuj_permesoj() -> PaĝaPermesos {
        PaĝaPermesos {
            skribebla: kani::any(),
            plenumebla: kani::any(),
            kaŝebla: kani::any(),
            uzanta: kani::any(),
        }
    }

    fn samaj(dekstra: PaĝaPermesos, maldekstra: PaĝaPermesos) -> bool {
        dekstra.skribebla == maldekstra.skribebla
            && dekstra.plenumebla == maldekstra.plenumebla
            && dekstra.kaŝebla == maldekstra.kaŝebla
            && dekstra.uzanta == maldekstra.uzanta
    }

    #[kani::proof]
    fn arm_permesoj_kaj_priskriboj_estas_reverseblaj() {
        let permesoj = iuj_permesoj();
        assert!(samaj(permesoj, el_descriptor(atributo(permesoj) | PAĜO)));
    }

    fn sen_wx(permesoj: PaĝaPermesos) {
        assert!(!(permesoj.skribebla && permesoj.plenumebla));
        let priskribo = el_descriptor(atributo(permesoj) | PAĜO);
        assert!(!(priskribo.skribebla && priskribo.plenumebla));
    }

    #[kani::proof]
    fn arm_politikaj_permesoj_respektas_wx() {
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
    fn arm_folio_konservas_la_aliniiĝon() {
        let fizika: u64 = kani::any();
        kani::assume(fizika < (1u64 << 48));
        kani::assume(aliniiĝinta(fizika));
        let priskribo = fizika | PAĜO | atributo(iuj_permesoj());
        assert_eq!(priskribo & ADRESA_MASKO, fizika);
        assert!(aliniiĝinta(priskribo & ADRESA_MASKO));
    }
}
