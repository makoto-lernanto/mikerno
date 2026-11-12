//! Arkitekture sendependa interrompa tavolo: la konservita kadro, la traktila
//! registrejo kaj la RAII-gardo por `RFLAGS.IF`.

use crate::kunhavigita::Kunhavigita;

pub use crate::arkitekturo::{ebligi_interrompojn, interrompoj_ebligitaj, malebligi_interrompojn};

pub const VEKTOROJ: usize = 256;

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
    fn trakta(&self, kadro: &InterrompaKadro);
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
    let kadro = unsafe { &*kadro };
    let vektoro = kadro.vektoro as usize;

    if let Some(traktilo) = TRAKTILOJ.akiri()[vektoro] {
        // EOI antaŭ la traktilo: ĝi rajtas ŝanĝi la kuntekston, kaj la venonta
        // tiko devas povi alveni ankaŭ dum la nova fadeno plenumas.
        if vektoro >= 0x20 && vektoro != 0xFF {
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
    crate::arkitekturo::konfirmi_interrompon();
}

fn trakti_escepton(kadro: &InterrompaKadro) -> ! {
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
