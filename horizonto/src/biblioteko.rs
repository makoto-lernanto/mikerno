//! Uzanta baza biblioteko: maldikaj envolvaĵoj ĉirkaŭ la sistemvokoj.
//!
//! La protokolo venas el `libmikerno`; ĉi tiu modulo nur donas al la uzanta
//! kodo Esperantajn nomojn sen montri la registran aranĝon.

#![allow(dead_code)]

use core::fmt::{self, Write};

pub use libmikerno::kapablo::{Kapablo, Permesoj, Tenilo};
pub use libmikerno::mesago::Mesago;

/// Skribu tekston al la kerna konzolo.
pub fn skribi(teksto: &str) -> isize {
    libmikerno::syscall::skribi(teksto)
}

/// Sufiĉa por unu protokola linio; pli longa teksto estas simple ignorata.
const FORMATO_LONGO: usize = 256;

struct Bufro {
    bajtoj: [u8; FORMATO_LONGO],
    longo: usize,
}

impl Bufro {
    const fn nova() -> Self {
        Self {
            bajtoj: [0; FORMATO_LONGO],
            longo: 0,
        }
    }
}

impl Write for Bufro {
    fn write_str(&mut self, teksto: &str) -> fmt::Result {
        let fino = self.longo + teksto.len();
        if fino > self.bajtoj.len() {
            return Err(fmt::Error);
        }
        self.bajtoj[self.longo..fino].copy_from_slice(teksto.as_bytes());
        self.longo = fino;
        Ok(())
    }
}

/// Skribu formatitan tekston per unu `SYS_SKRIBI`. Formati aparte kaj sendi la
/// tutan linion unufoje tenas la eliron atomia eĉ kiam la planisto cedas al la
/// alia procezo inter du linioj.
pub fn skribi_formaton(argumentoj: fmt::Arguments<'_>) {
    let mut bufro = Bufro::nova();
    if bufro.write_fmt(argumentoj).is_ok()
        && let Ok(teksto) = core::str::from_utf8(&bufro.bajtoj[..bufro.longo])
    {
        skribi(teksto);
    }
}

/// Sendu mesaĝon tra la kapablo.
pub fn sendi(kapablo: Kapablo, mesago: &Mesago) -> isize {
    libmikerno::syscall::sendi(kapablo, mesago)
}

/// Ricevu mesaĝon tra la kapablo en `celo`.
pub fn ricevi(kapablo: Kapablo, celo: &mut Mesago) -> isize {
    libmikerno::syscall::ricevi(kapablo, celo)
}

/// Kreu finpunkton kun la donitaj permesoj; `> 0` estas la tenila nombro.
pub fn krei_finpunkton(permesoj: Permesoj) -> isize {
    libmikerno::syscall::krei_finpunkton(permesoj)
}

/// Fermu la lokan tenilon de finpunkto.
pub fn fermi(tenilo: Tenilo) -> isize {
    libmikerno::syscall::fermi(tenilo)
}

/// Cedu la procesoron al la planisto.
pub fn cedi() -> isize {
    libmikerno::syscall::cedi()
}

/// Finu la nunan procezon. Se la kerno rifuzas, la procezo restas blokita.
pub fn fini(kodo: usize) -> ! {
    let _ = libmikerno::syscall::fini(kodo);
    loop {
        core::hint::spin_loop();
    }
}
