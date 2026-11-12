//! Uzanta baza biblioteko: maldikaj envolvaĵoj ĉirkaŭ la sistemvokoj.
//!
//! La protokolo venas el `libmikerno`; ĉi tiu modulo nur donas al la uzanta
//! kodo Esperantajn nomojn sen montri la registran aranĝon.

#![allow(dead_code)]

pub use libmikerno::kapablo::Kapablo;
pub use libmikerno::mesago::Mesago;

/// Skribu tekston al la kerna konzolo.
pub fn skribi(teksto: &str) -> isize {
    libmikerno::syscall::skribi(teksto)
}

/// Sendu mesaĝon tra la kapablo.
pub fn sendi(kapablo: Kapablo, mesago: &Mesago) -> isize {
    libmikerno::syscall::sendi(kapablo, mesago)
}

/// Ricevu mesaĝon tra la kapablo en `celo`.
pub fn ricevi(kapablo: Kapablo, celo: &mut Mesago) -> isize {
    libmikerno::syscall::ricevi(kapablo, celo)
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
