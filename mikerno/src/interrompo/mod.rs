//! Arkitekture sendependa interrompa tavolo: la konservita kadro, la traktila
//! registrejo kaj la RAII-gardo por `RFLAGS.IF`.

use crate::kunhavigita::Kunhavigita;

pub use crate::arkitekturo::{ebligi_interrompojn, interrompoj_ebligitaj, malebligi_interrompojn};

pub const VEKTOROJ: usize = 256;
/// La vektoro de `int 0x80`; ĝia pordego estas la sola DPL3-pordego.
pub const SISTEMVOKA_VEKTORO: u8 = 0x80;

/// La stako, kiun la enira ŝelko konservas antaŭ ol voki Rust.
#[repr(C)]
#[derive(Clone, Copy)]
#[allow(dead_code)]
pub struct InterrompaKadro {
    pub rsp: u64,
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rbp: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    pub vektoro: u64,
    pub erarkodo: u64,
    pub rip: u64,
    pub cs: u64,
    pub rflags: u64,
}

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
    // `#GP` kaj `#PF` el Ring 3 apartenas al la uzanta procezo, ne al la
    // kerno: la procezo estas mortigata kaj la planisto daŭrigas. Ĉiuj aliaj
    // esceptoj — ankaŭ la samaj vektoroj el Ring 0 — restas fatalaj.
    if kadro.cs & 3 == 3 && matches!(kadro.vektoro, 13 | 14) {
        let nomo = if kadro.vektoro == 14 {
            "paĝa eraro"
        } else {
            "ĝenerala protekta eraro"
        };
        crate::skribu!(
            "\n[eraro] procezo {}: {} ĉe rip=0x{:016x}, erarkodo=0x{:x}",
            crate::kerno::planisto::nuna_procezo(),
            nomo,
            kadro.rip,
            kadro.erarkodo
        );
        if kadro.vektoro == 14 {
            crate::skribu!(", CR2=0x{:016x}", crate::arkitekturo::fuŝa_adreso());
        }
        crate::skribu!("; la procezo estas mortigata\n");
        // `morti` detruas la procezan adresan spacon, elektas alian fadenon
        // kaj neniam revenas al ĉi tiu kadro.
        crate::kerno::planisto::morti()
    }

    crate::skribu!(
        "\n[kerno] escepto {} (erarkodo 0x{:x}) rip=0x{:016x} rsp=0x{:016x} rflags=0x{:x}\n",
        kadro.vektoro,
        kadro.erarkodo,
        kadro.rip,
        kadro.rsp,
        kadro.rflags
    );
    if kadro.vektoro == 14 {
        crate::skribu!(
            "[kerno] paĝa eraro ĉe 0x{:016x}\n",
            crate::arkitekturo::fuŝa_adreso()
        );
    }
    crate::arkitekturo::halti()
}
