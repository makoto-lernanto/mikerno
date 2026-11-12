//! 256-vektora interrompa priskriba tabelo (IDT).
//!
//! Vektoroj kun erarkodo ne ricevas ŝajn-erarkodon. Ĉiuj enirpunktoj saltas al
//! `isr_komuna`, kiu konservas la registrojn, alvokas Rust-on per
//! `InterrompaKadro` kaj finas per `iretq`.

use core::arch::{asm, global_asm};
use core::mem::{offset_of, size_of};

use crate::interrompo::{InterrompaKadro, SISTEMVOKA_VEKTORO, VEKTOROJ};

const ERARKODAJ: [u8; 10] = [8, 10, 11, 12, 13, 14, 17, 21, 29, 30];
const STRIDO: u64 = 32;

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct IdtEro {
    offseto_malalta: u16,
    selektilo: u16,
    ist: u8,
    tipo_atributo: u8,
    offseto_meza: u16,
    offseto_alta: u32,
    rezervita: u32,
}

impl IdtEro {
    const fn mankanta() -> Self {
        Self {
            offseto_malalta: 0,
            selektilo: 0,
            ist: 0,
            tipo_atributo: 0,
            offseto_meza: 0,
            offseto_alta: 0,
            rezervita: 0,
        }
    }

    fn agordi(&mut self, traktilo: u64, dpl3: bool) {
        self.offseto_malalta = traktilo as u16;
        self.selektilo = 0x08;
        // Ĉeesta | DPL | 64-bita interrompa pordego. Interrompa pordego (ne
        // kapta) malŝaltas `RFLAGS.IF`, do temporizilo ne reeniras dum voko.
        self.tipo_atributo = if dpl3 { 0xEE } else { 0x8E };
        self.offseto_meza = (traktilo >> 16) as u16;
        self.offseto_alta = (traktilo >> 32) as u32;
    }
}

#[repr(C)]
pub struct InterrompaPriskribaTabelo {
    eroj: [IdtEro; VEKTOROJ],
}

#[repr(C, packed)]
struct IdtMontrilo {
    limo: u16,
    bazo: u64,
}

static mut IPT: InterrompaPriskribaTabelo = InterrompaPriskribaTabelo {
    eroj: [IdtEro::mankanta(); VEKTOROJ],
};

const _: () = assert!(size_of::<IdtEro>() == 16);
const _: () = assert!(size_of::<InterrompaKadro>() == 168);
const _: () = assert!(offset_of!(InterrompaKadro, vektoro) == 128);
const _: () = assert!(offset_of!(InterrompaKadro, erarkodo) == 136);
const _: () = assert!(offset_of!(InterrompaKadro, rip) == 144);
const _: () = assert!(offset_of!(InterrompaKadro, cs) == 152);
const _: () = assert!(offset_of!(InterrompaKadro, rflags) == 160);

global_asm!(
    r#"
    .section .text

    /* Vektoroj sen erarkodo: puŝu ŝajn-erarkodon 0 kaj la vektoron. */
    .p2align 5
    .global isr_sen_eraro
isr_sen_eraro:
    .set vektoro_sen, 0
    .rept 256
        .p2align 5
        push 0
        push vektoro_sen
        jmp isr_komuna
        .set vektoro_sen, vektoro_sen + 1
    .endr

    /* Vektoroj kun erarkodo: la CPU jam puŝis ĝin; puŝu nur la vektoron. */
    .p2align 5
    .global isr_kun_eraro
isr_kun_eraro:
    .set vektoro_kun, 0
    .rept 256
        .p2align 5
        push vektoro_kun
        jmp isr_komuna
        .set vektoro_kun, vektoro_kun + 1
    .endr

    /* La stako devas kongrui kun `InterrompaKadro`. */
    .global isr_komuna
isr_komuna:
    push r15
    push r14
    push r13
    push r12
    push r11
    push r10
    push r9
    push r8
    push rbp
    push rdi
    push rsi
    push rdx
    push rcx
    push rbx
    push rax
    mov rax, rsp
    add rax, 48            /* 8 (rax) + 16 (vektoro+erarkodo) + 24 (CPU) */
    push rax               /* konservita rsp */

    cld
    mov rdi, rsp
    mov r15, rsp           /* originalo jam estas en la stako */
    and rsp, -16
    call interrompa_dispetcho
    mov rsp, r15

    add rsp, 8             /* forĵetu la konservitan rsp */
    pop rax
    pop rbx
    pop rcx
    pop rdx
    pop rsi
    pop rdi
    pop rbp
    pop r8
    pop r9
    pop r10
    pop r11
    pop r12
    pop r13
    pop r14
    pop r15
    add rsp, 16            /* forĵetu vektoron kaj erarkodon */
    iretq
"#
);

unsafe extern "C" {
    static isr_sen_eraro: u8;
    static isr_kun_eraro: u8;
}

pub fn instali() {
    unsafe {
        let ipt = &mut *core::ptr::addr_of_mut!(IPT);
        let sen_eraro = &raw const isr_sen_eraro as u64;
        let kun_eraro = &raw const isr_kun_eraro as u64;

        for (vektoro, ero) in ipt.eroj.iter_mut().enumerate() {
            let bazo = if ERARKODAJ.contains(&(vektoro as u8)) {
                kun_eraro
            } else {
                sen_eraro
            };
            ero.agordi(
                bazo + STRIDO * vektoro as u64,
                vektoro as u8 == SISTEMVOKA_VEKTORO,
            );
        }

        let montrilo = IdtMontrilo {
            limo: (size_of::<InterrompaPriskribaTabelo>() - 1) as u16,
            bazo: &raw const IPT as u64,
        };
        asm!("lidt [{0}]", in(reg) &montrilo, options(nostack));
    }
}
