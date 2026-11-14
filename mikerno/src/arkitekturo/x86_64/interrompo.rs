//! La platforma interrompa kadro kaj la klasifiko de esceptoj.
//!
//! `idt.rs` muntas la eniran ŝelkon, kiu konstruas ĉi tiun strukturon sur la
//! kerna stako antaŭ ol voki Rust. La komuna tavolo `interrompo` aliras ĝin nur
//! per `SistemvokaKadro` kaj per la funkcioj de ĉi tiu modulo.

use crate::arkitekturo::komuna::{EsceptoTipo, SistemvokaKadro};

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

impl SistemvokaKadro for InterrompaKadro {
    fn numero(&self) -> usize {
        self.rax as usize
    }

    fn argumento(&self, indekso: usize) -> u64 {
        match indekso {
            0 => self.rdi,
            1 => self.rsi,
            2 => self.rdx,
            3 => self.r10,
            _ => 0,
        }
    }

    fn agordi_rezulton(&mut self, rezulto: u64) {
        self.rax = rezulto;
    }

    fn el_uzanto(&self) -> bool {
        self.cs & 3 == 3
    }

    fn vektoro(&self) -> u64 {
        self.vektoro
    }

    fn erarkodo(&self) -> u64 {
        self.erarkodo
    }

    fn instrukcipunkto(&self) -> u64 {
        self.rip
    }

    fn stakmontrilo(&self) -> u64 {
        self.rsp
    }

    fn stato(&self) -> u64 {
        self.rflags
    }
}

/// Klasifiku la vektoron laŭ la komuna escepta speco.
pub fn escepto_speco(vektoro: u64) -> EsceptoTipo {
    match vektoro {
        13 => EsceptoTipo::ProtektaEraro,
        14 => EsceptoTipo::PaĝaEraro,
        alia => EsceptoTipo::Nekonata(alia),
    }
}
