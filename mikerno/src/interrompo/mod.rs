//! Arkitekture sendependa interrompa tavolo: la traktila registrejo, la
//! komuna escepta politiko kaj la RAII-gardo por la interrompa flago.

use crate::arkitekturo::InterrompaKadro;
#[cfg(target_arch = "x86_64")]
use crate::arkitekturo::{EsceptoTipo, SistemvokaKadro};
use crate::kunhavigita::Kunhavigita;
#[cfg(target_arch = "x86_64")]
use crate::memoro::virtuala::{UZANTA_BAZO, UZANTA_FINO};

pub use crate::arkitekturo::{ebligi_interrompojn, interrompoj_ebligitaj, malebligi_interrompojn};

pub const VEKTOROJ: usize = 256;
/// La vektoro de `int 0x80`; ĝia pordego estas la sola DPL3-pordego.
#[cfg(target_arch = "x86_64")]
pub const SISTEMVOKA_VEKTORO: u8 = 0x80;
/// `RFLAGS.AC` (bito 18): kun `CR4.SMAP` ŝaltita, Ring 0 povas aliri uzantajn
/// paĝojn nur kiam ĉi tiu bito estas 1.
#[cfg(target_arch = "x86_64")]
const FLAGO_AC: u64 = 1 << 18;

pub trait InterrompaTraktilo {
    /// La kadro estas ŝanĝebla, ĉar sistemvoko skribas sian rezulton en `rax`.
    fn trakta(&self, kadro: &mut InterrompaKadro);
}

static TRAKTILOJ: Kunhavigita<[Option<&'static dyn InterrompaTraktilo>; VEKTOROJ]> =
    Kunhavigita::nova([None; VEKTOROJ]);

/// Registru traktilon por unu vektoro. La traktilo devas vivi por ĉiam.
pub fn registri(vektoro: u8, traktilo: &'static dyn InterrompaTraktilo) {
    TRAKTILOJ.akiri()[vektoro as usize] = Some(traktilo);
}

/// Malŝaltu interrompojn provizore; `Drop` restarigas la antaŭan staton.
pub struct InterrompaGardo {
    antaŭa: bool,
}

impl InterrompaGardo {
    pub fn nova() -> Self {
        let antaŭa = interrompoj_ebligitaj();
        malebligi_interrompojn();
        Self { antaŭa }
    }
}

impl Drop for InterrompaGardo {
    fn drop(&mut self) {
        if self.antaŭa {
            ebligi_interrompojn();
        }
    }
}

/// Voku la registritan traktilon por `vektoro`; `false` se neniu ekzistas.
/// La platforma dispetĉo alvokas ĝin post la aparata konfirmo.
#[cfg(target_arch = "aarch64")]
pub fn trakti_vektoron(vektoro: usize, kadro: &mut InterrompaKadro) -> bool {
    if let Some(traktilo) = TRAKTILOJ.akiri()[vektoro] {
        traktilo.trakta(kadro);
        true
    } else {
        false
    }
}

/// Vokata de la arkitektura enira ŝelko.
#[cfg(target_arch = "x86_64")]
#[unsafe(no_mangle)]
pub extern "C" fn interrompa_dispetcho(kadro: *mut InterrompaKadro) {
    // Ring 3 rajtas ŝalti `RFLAGS.AC` per `popfq`, kaj la interrompa pordego
    // konservas ĝin. La kerno devas malŝalti ĝin antaŭ ol tuŝi ajnan uzantan
    // montrilon; `iretq` restarigas la konservitan kredaron, do la uzanto
    // revidas sian propran `AC`.
    crate::arkitekturo::malŝalti_ac();
    // La disciplino havas unu sencon nur se `CLAC` vere fermas la fenestron.
    // Se ĝi restus malfermita, la tuta sistemvoka kopiado perdus la SMAP-barilon.
    if crate::arkitekturo::smap_aktiva() && crate::arkitekturo::ac_aktiva() {
        crate::skribu!("\n[kerno] AC-restaĵo: la SMAP-fenestro restis malfermita; haltas\n");
        crate::arkitekturo::halti();
    }

    let kadro = unsafe { &mut *kadro };
    let vektoro = kadro.vektoro as usize;

    if let Some(traktilo) = TRAKTILOJ.akiri()[vektoro] {
        // EOI antaŭ la traktilo: ĝi rajtas ŝanĝi la kuntekston, kaj la venonta
        // tiko devas povi alveni ankaŭ dum la nova fadeno plenumas.
        if estas_apic_vektoro(vektoro) {
            crate::arkitekturo::konfirmi_interrompon();
        }
        traktilo.trakta(kadro);
        return;
    }

    if vektoro < 0x20 {
        trakti_escepton(kadro);
    }
    if vektoro == 0xFF {
        // Spuria APIC-interrompo: nek traktilo nek EOI necesas.
        return;
    }

    crate::skribu!("\n[kerno] neatendita interrompo: vektoro {}\n", vektoro);
    if estas_apic_vektoro(vektoro) {
        crate::arkitekturo::konfirmi_interrompon();
    }
}

/// Nur aparataj lokaj APIC-vektoroj (`0x20..0x80`) postulas EOI.
///
/// `int 0x80` estas programa interrompo: la APIC-ISR ne havas ĝian biton, do
/// EOI estus mislokita. Ankaŭ `int3` kaj la spuria `0xFF` restas ekstere.
#[cfg(target_arch = "x86_64")]
pub(crate) const fn estas_apic_vektoro(vektoro: usize) -> bool {
    vektoro >= 0x20 && vektoro < 0x80
}

#[cfg(target_arch = "x86_64")]
fn trakti_escepton(kadro: &mut InterrompaKadro) -> ! {
    let speco = crate::arkitekturo::escepto_speco(kadro.vektoro());
    // `#GP` kaj `#PF` el Ring 3 apartenas al la uzanta procezo, ne al la
    // kerno: la procezo estas mortigata kaj la planisto daŭrigas. Ĉiuj aliaj
    // esceptoj — ankaŭ la samaj vektoroj el Ring 0 — restas fatalaj.
    if kadro.el_uzanto() && matches!(speco, EsceptoTipo::PaĝaEraro | EsceptoTipo::ProtektaEraro) {
        let nomo = if speco == EsceptoTipo::PaĝaEraro {
            "paĝa eraro"
        } else {
            "ĝenerala protekta eraro"
        };
        crate::skribu!(
            "\n[eraro] procezo {}: {} ĉe rip=0x{:016x}, erarkodo=0x{:x}",
            crate::kerno::planisto::nuna_procezo(),
            nomo,
            kadro.instrukcipunkto(),
            kadro.erarkodo()
        );
        if speco == EsceptoTipo::PaĝaEraro {
            crate::skribu!(", CR2=0x{:016x}", crate::arkitekturo::fuŝa_adreso());
        }
        crate::skribu!("; la procezo estas mortigata\n");
        // `morti` detruas la procezan adresan spacon, elektas alian fadenon
        // kaj neniam revenas al ĉi tiu kadro.
        crate::kerno::planisto::morti()
    }

    // Ring 0 `#PF` ĉe uzanta adreso estas firmiga malobservo. SMEP rifuzas
    // plenumon (`I=1`); SMAP rifuzas datumaliron (`I=0`) dum `RFLAGS.AC=0`.
    // `U/S` estas 0 en ambaŭ kazoj, ĉar la fuŝo okazis en Ring 0.
    if speco == EsceptoTipo::PaĝaEraro && !kadro.el_uzanto() {
        let adreso = crate::arkitekturo::fuŝa_adreso();
        if (UZANTA_BAZO..UZANTA_FINO).contains(&adreso) {
            if kadro.erarkodo() & 0x10 != 0 && crate::arkitekturo::smep_aktiva() {
                crate::skribu!(
                    "\n[eraro] SMEP-malobservo: Ring 0 provis plenumi uzantan paĝon ĉe 0x{:016x} (erarkodo=0x{:x})\n",
                    adreso,
                    kadro.erarkodo()
                );
                crate::arkitekturo::halti();
            }
            if kadro.erarkodo() & 0x10 == 0
                && kadro.stato() & FLAGO_AC == 0
                && crate::arkitekturo::smap_aktiva()
            {
                crate::skribu!(
                    "\n[eraro] procezo {}: SMAP-malobservo ĉe 0x{:016x} (erarkodo=0x{:x})\n",
                    crate::kerno::planisto::nuna_procezo(),
                    adreso,
                    kadro.erarkodo()
                );
                crate::arkitekturo::halti();
            }
        }
    }

    crate::skribu!(
        "\n[kerno] escepto {} (erarkodo 0x{:x}) rip=0x{:016x} rsp=0x{:016x} rflags=0x{:x}\n",
        kadro.vektoro(),
        kadro.erarkodo(),
        kadro.instrukcipunkto(),
        kadro.stakmontrilo(),
        kadro.stato()
    );
    if speco == EsceptoTipo::PaĝaEraro {
        crate::skribu!(
            "[kerno] paĝa eraro ĉe 0x{:016x}\n",
            crate::arkitekturo::fuŝa_adreso()
        );
    }
    crate::arkitekturo::halti()
}

#[cfg(kani)]
mod kani_proofs {
    //! Pruvo por la EOI-politiko: EOI apartenas nur al la aparataj vektoroj.

    use super::estas_apic_vektoro;

    #[kani::proof]
    fn eoi_nur_por_apic_vektoroj() {
        let vektoro: usize = kani::any();
        if estas_apic_vektoro(vektoro) {
            assert!(vektoro >= 0x20 && vektoro < 0x80);
        } else {
            assert!(vektoro < 0x20 || vektoro >= 0x80);
        }
        assert!(!estas_apic_vektoro(0x0E));
        assert!(!estas_apic_vektoro(super::SISTEMVOKA_VEKTORO as usize));
        assert!(!estas_apic_vektoro(0xFF));
        assert!(estas_apic_vektoro(0x20));
        assert!(estas_apic_vektoro(0x7F));
    }
}
