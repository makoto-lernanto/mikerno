//! Uzanta baza biblioteko: maldikaj envolvaĵoj ĉirkaŭ la sistemvokoj.
//!
//! La protokolo venas el `libmikerno`; ĉi tiu modulo nur donas al la uzanta
//! kodo Esperantajn nomojn sen montri la registran aranĝon.

#![allow(dead_code)]

use core::fmt::{self, Write};

pub use libmikerno::kapablo::{Permesoj, Tenilo};
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

/// Sendu mesaĝon tra la tenilo.
pub fn sendi(tenilo: Tenilo, mesago: &Mesago) -> isize {
    libmikerno::syscall::sendi(tenilo, mesago)
}

/// Ricevu mesaĝon tra la tenilo en `celo`.
pub fn ricevi(tenilo: Tenilo, celo: &mut Mesago) -> isize {
    libmikerno::syscall::ricevi(tenilo, celo)
}

/// Ricevu mesaĝon kun fonta filtro (`0` = ajna) kaj tempolimo.
pub fn ricevi_kun_tempolimo(
    tenilo: Tenilo,
    celo: &mut Mesago,
    fonto: u64,
    tempolimo_ms: u64,
) -> isize {
    libmikerno::syscall::ricevi_kun_tempolimo(tenilo, celo, fonto, tempolimo_ms)
}

/// Sendu mesaĝon tra la tenilo kun tempolimo.
pub fn sendi_kun_tempolimo(tenilo: Tenilo, mesago: &Mesago, tempolimo_ms: u64) -> isize {
    libmikerno::syscall::sendi_kun_tempolimo(tenilo, mesago, tempolimo_ms)
}

/// Donu la tenilon al la procezo `celo` kun la permesoj `permesoj`.
pub fn doni(fonto: Tenilo, celo: u64, permesoj: Permesoj) -> isize {
    libmikerno::syscall::doni(fonto, celo, permesoj)
}

/// Kreu finpunkton kun la donitaj permesoj; `> 0` estas la tenila nombro.
pub fn krei_finpunkton(permesoj: Permesoj) -> isize {
    libmikerno::syscall::krei_finpunkton(permesoj)
}

/// Fermu la lokan tenilon de finpunkto.
pub fn fermi(tenilo: Tenilo) -> isize {
    libmikerno::syscall::fermi(tenilo)
}

/// Revoku la tutan finpunkton de la tenilo.
pub fn revoki(tenilo: Tenilo) -> isize {
    libmikerno::syscall::revoki(tenilo)
}

/// Cedu la procesoron al la planisto.
pub fn cedi() -> isize {
    libmikerno::syscall::cedi()
}

/// Dormu almenaŭ la donitajn milisekundojn.
pub fn dormi(millisekundoj: u64) -> isize {
    libmikerno::syscall::dormi(millisekundoj)
}

/// Finu la nunan procezon. Se la kerno rifuzas, la procezo restas blokita.
pub fn fini(kodo: usize) -> ! {
    let _ = libmikerno::syscall::fini(kodo);
    loop {
        core::hint::spin_loop();
    }
}
