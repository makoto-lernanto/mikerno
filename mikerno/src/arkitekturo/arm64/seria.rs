//! PL011 UART-surmetilo, demandata reĝimo.
//!
//! QEMU `virt` metas la unuan PL011-on ĉe `0x0900_0000`. La adreso venas el
//! la FDT; la fiksa valoro restas nur kiel diagnoza rezerva agordo antaŭ ol la
//! FDT estas analizita.

use core::arch::asm;
use core::fmt::{self, Write};
use core::ptr::{read_volatile, write_volatile};

pub const DEFAŬLTA: usize = 0x0900_0000;

const FR: usize = 0x18;
const IBRD: usize = 0x24;
const FBRD: usize = 0x28;
const LCR_H: usize = 0x2C;
const CR: usize = 0x30;
const IMSC: usize = 0x38;
const ICR: usize = 0x44;

const FR_TXFF: u32 = 1 << 5;
const FR_RXFE: u32 = 1 << 4;

static mut BAZO: usize = DEFAŬLTA;

/// Pravalorizu la PL011-on ĉe `bazo`: 115200 8N1, FIFO ŝaltita.
pub fn pravalorizi(bazo: usize) {
    unsafe {
        BAZO = bazo;
        write_volatile((bazo + CR) as *mut u32, 0);
        write_volatile((bazo + ICR) as *mut u32, 0x7FF);
        // UARTCLK = 24 MHz: 24_000_000 / (16 * 115200) = 13.0208.
        write_volatile((bazo + IBRD) as *mut u32, 13);
        write_volatile((bazo + FBRD) as *mut u32, 1);
        write_volatile((bazo + LCR_H) as *mut u32, 0x70); // 8N1, FIFO
        write_volatile((bazo + IMSC) as *mut u32, 0);
        write_volatile((bazo + CR) as *mut u32, 0x301); // UARTEN | TXE | RXE
    }
    // La agordoj devas esti videblaj antaŭ la unua signo.
    unsafe { asm!("dsb sy", options(nomem, nostack)) };
}

pub fn skribi_bajton(bajto: u8) {
    if bajto == b'\n' {
        skribi_bajton(b'\r');
    }
    let bazo = unsafe { BAZO };
    while unsafe { read_volatile((bazo + FR) as *const u32) } & FR_TXFF != 0 {}
    unsafe { write_volatile(bazo as *mut u32, bajto as u32) };
}

/// Legu unubajton el la enigbufro, se ĝi jam enhavas unu.
pub fn legi_bajton() -> Option<u8> {
    let bazo = unsafe { BAZO };
    if unsafe { read_volatile((bazo + FR) as *const u32) } & FR_RXFE != 0 {
        return None;
    }
    Some(unsafe { read_volatile(bazo as *const u32) } as u8)
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
