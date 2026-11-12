//! x86_64-kuntekstŝanĝo kaj la unua eniro de nova fadeno.
//!
//! `sxalti_kuntekston_asm` konservas ĉiujn ĝeneralajn registrojn de la nuna
//! fadeno, same kiel `rsp`, `rip` kaj `rflags`, kaj poste restarigas la
//! konservitan staton de la sekva. La reveno okazas per `ret`, do la konservita
//! `rsp` montras al la revenadreso de la alvoko.
//!
//! Nova fadeno komenciĝas per artefarita stako: `[stako_pinto - 8]` enhavas la
//! adreson de `fadeno_trampolino`, kaj `rax` portas la fadenan identigilon. La
//! trampolino ŝaltas interrompojn kaj transdonas la identigilon al Rust.

use core::arch::global_asm;
use core::mem::{offset_of, size_of};

use crate::kerno::objekto::Kunteksto;

const O_RAX: usize = offset_of!(Kunteksto, rax);
const O_RBX: usize = offset_of!(Kunteksto, rbx);
const O_RCX: usize = offset_of!(Kunteksto, rcx);
const O_RDX: usize = offset_of!(Kunteksto, rdx);
const O_RSI: usize = offset_of!(Kunteksto, rsi);
const O_RDI: usize = offset_of!(Kunteksto, rdi);
const O_RBP: usize = offset_of!(Kunteksto, rbp);
const O_RSP: usize = offset_of!(Kunteksto, rsp);
const O_RIP: usize = offset_of!(Kunteksto, rip);
const O_RFLAGS: usize = offset_of!(Kunteksto, rflags);
const O_R8: usize = offset_of!(Kunteksto, r8);
const O_R9: usize = offset_of!(Kunteksto, r9);
const O_R10: usize = offset_of!(Kunteksto, r10);
const O_R11: usize = offset_of!(Kunteksto, r11);
const O_R12: usize = offset_of!(Kunteksto, r12);
const O_R13: usize = offset_of!(Kunteksto, r13);
const O_R14: usize = offset_of!(Kunteksto, r14);
const O_R15: usize = offset_of!(Kunteksto, r15);

const _: () = assert!(size_of::<Kunteksto>() == 18 * 8);

global_asm!(
    r#"
    .section .text

    /* rdi = malnova kunteksto, rsi = nova kunteksto */
    .p2align 4
    .global sxalti_kuntekston_asm
sxalti_kuntekston_asm:
    mov [rdi + {o_rax}], rax
    mov [rdi + {o_rbx}], rbx
    mov [rdi + {o_rcx}], rcx
    mov [rdi + {o_rdx}], rdx
    mov [rdi + {o_rsi}], rsi
    mov [rdi + {o_rbp}], rbp
    mov [rdi + {o_r8}], r8
    mov [rdi + {o_r9}], r9
    mov [rdi + {o_r10}], r10
    mov [rdi + {o_r11}], r11
    mov [rdi + {o_r12}], r12
    mov [rdi + {o_r13}], r13
    mov [rdi + {o_r14}], r14
    mov [rdi + {o_r15}], r15

    /* `rsp` montras al la revenadreso; `rip` estas tiu adreso mem. */
    mov [rdi + {o_rsp}], rsp
    mov rax, [rsp]
    mov [rdi + {o_rip}], rax
    pushfq
    pop rax
    mov [rdi + {o_rflags}], rax
    mov [rdi + {o_rdi}], rdi

    mov r8,  [rsi + {o_r8}]
    mov r9,  [rsi + {o_r9}]
    mov r10, [rsi + {o_r10}]
    mov r11, [rsi + {o_r11}]
    mov r12, [rsi + {o_r12}]
    mov r13, [rsi + {o_r13}]
    mov r14, [rsi + {o_r14}]
    mov r15, [rsi + {o_r15}]
    mov rbp, [rsi + {o_rbp}]
    mov rbx, [rsi + {o_rbx}]
    mov rcx, [rsi + {o_rcx}]
    mov rdx, [rsi + {o_rdx}]
    mov rax, [rsi + {o_rax}]

    mov rsp, [rsi + {o_rsp}]
    push qword ptr [rsi + {o_rflags}]
    popfq
    mov rdi, [rsi + {o_rdi}]
    mov rsi, [rsi + {o_rsi}]
    ret

    /* Unua eniro: rax = fadeno-id, rsp = pinto de la fadena stako. */
    .p2align 4
    .global fadeno_trampolino
fadeno_trampolino:
    sti
    mov rdi, rax
    xor ebp, ebp
    call fadeno_eniro
    ud2
    "#,
    o_rax = const O_RAX,
    o_rbx = const O_RBX,
    o_rcx = const O_RCX,
    o_rdx = const O_RDX,
    o_rsi = const O_RSI,
    o_rdi = const O_RDI,
    o_rbp = const O_RBP,
    o_rsp = const O_RSP,
    o_rip = const O_RIP,
    o_rflags = const O_RFLAGS,
    o_r8 = const O_R8,
    o_r9 = const O_R9,
    o_r10 = const O_R10,
    o_r11 = const O_R11,
    o_r12 = const O_R12,
    o_r13 = const O_R13,
    o_r14 = const O_R14,
    o_r15 = const O_R15,
);

unsafe extern "C" {
    fn sxalti_kuntekston_asm(malnova: *mut Kunteksto, nova: *const Kunteksto);
    static fadeno_trampolino: u8;
}

/// Konservu la nunan kuntekston kaj restarigu la donitan.
///
/// # Sekureco
/// Ambaŭ montriloj devas montri al vivantaj, ĝuste aliniiĝintaj `Kunteksto`
/// valoroj. La funkcio neniam revenas per ordinara vojo se la celo estas
/// neniam planita; alie ĝi revenas, kiam la nuna kunteksto restariĝas.
pub unsafe fn ŝalti_kuntekston(malnova: *mut Kunteksto, nova: *const Kunteksto) {
    unsafe { sxalti_kuntekston_asm(malnova, nova) }
}

/// Pretigu kuntekston tiel, ke la unua `ret` eniros `fadeno_trampolino`.
pub fn prepari_kuntekston(konteksto: &mut Kunteksto, stako_pinto: u64) {
    *konteksto = Kunteksto::nova();
    let rsp = stako_pinto - 8;
    unsafe {
        core::ptr::write_volatile(rsp as *mut u64, &raw const fadeno_trampolino as u64);
    }
    konteksto.rsp = rsp;
    konteksto.rip = &raw const fadeno_trampolino as u64;
    // `iretq`/`popfq` restarigas ĉi tion; la trampolino mem ŝaltas IF.
    konteksto.rflags = 0x2;
}
