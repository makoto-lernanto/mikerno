//! Arkitekture sendependa interrompa tavolo: la traktila registrejo, la
//! komuna escepta politiko kaj la RAII-gardo por la interrompa flago.

use crate::arkitekturo::{EsceptoTipo, InterrompaKadro, SistemvokaKadro};
use crate::kunhavigita::Kunhavigita;
use crate::memoro::virtuala::{UZANTA_BAZO, UZANTA_FINO};

pub use crate::arkitekturo::{ebligi_interrompojn, interrompoj_ebligitaj, malebligi_interrompojn};

pub const VEKTOROJ: usize = 256;
/// La vektoro de `int 0x80`; ĝia pordego estas la sola DPL3-pordego.
pub const SISTEMVOKA_VEKTORO: u8 = 0x80;
/// `RFLAGS.AC` (bito 18): kun `CR4.SMAP` ŝaltita, Ring 0 povas aliri uzantajn
/// paĝojn nur kiam ĉi tiu bito estas 1.
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

/// Vokata de la arkitektura enira ŝelko.
#[unsafe(no_mangle)]
pub extern "C" fn interrompa_dispetcho(kadro: *mut InterrompaKadro) {
    // Ring 3 rajtas ŝalti `RFLAGS.AC` per `popfq`, kaj la interrompa pordego
    // konservas ĝin. La kerno devas malŝalti ĝin antaŭ ol tuŝi ajnan uzantan
    // montrilon; `iretq` restarigas la konservitan kredaron, do la uzanto
    // revidas sian propran `AC`.
    crate::arkitekturo::malŝalti_ac();

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
fn estas_apic_vektoro(vektoro: usize) -> bool {
    (0x20..0x80).contains(&vektoro)
}

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
