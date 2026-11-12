//! 16550 UART-surmetilo sur COM1 (0x3F8), demandata reĝimo.
//!
//! Ĉi tiu estas la arkitektura konzolo; la kerna nukleo vidas nur
//! `arkitekturo::skribi` kaj `arkitekturo::skribi_formaton`.

use core::arch::asm;
use core::fmt::{self, Write};

const COM1: u16 = 0x3F8;

#[inline]
unsafe fn eligi(porto: u16, valoro: u8) {
    unsafe {
        asm!("out dx, al", in("dx") porto, in("al") valoro, options(nomem, nostack, preserves_flags));
    }
}

#[inline]
unsafe fn enigi(porto: u16) -> u8 {
    let valoro: u8;
    unsafe {
        asm!("in al, dx", out("al") valoro, in("dx") porto, options(nomem, nostack, preserves_flags));
    }
    valoro
}

pub fn pravalorizi() {
    unsafe {
        eligi(COM1 + 1, 0x00); // malŝaltu interrompojn
        eligi(COM1 + 3, 0x80); // ŝaltu DLAB
        eligi(COM1, 0x01); // divido malalta: 1 => 115200 baudoj
        eligi(COM1 + 1, 0x00); // divido alta
        eligi(COM1 + 3, 0x03); // 8N1, DLAB malŝaltita
        eligi(COM1 + 2, 0xC7); // FIFO ŝaltita, malplenigita, sojlo 14 bajtoj
        eligi(COM1 + 4, 0x0B); // sen IRQ, RTS/DSR ŝaltitaj
    }
}

fn atendi_elsendon() {
    unsafe { while enigi(COM1 + 5) & 0x20 == 0 {} }
}

pub fn skribi_bajton(bajto: u8) {
    if bajto == b'\n' {
        atendi_elsendon();
        unsafe { eligi(COM1, b'\r') };
    }
    atendi_elsendon();
    unsafe { eligi(COM1, bajto) };
}

pub struct Konzolo;

impl Write for Konzolo {
    fn write_str(&mut self, teksto: &str) -> fmt::Result {
        for bajto in teksto.bytes() {
            skribi_bajton(bajto);
        }
        Ok(())
    }
}

pub fn skribi_formaton(argumentoj: fmt::Arguments) {
    let _ = Konzolo.write_fmt(argumentoj);
}
