//! Praŝarĝa ŝelko: 32-bita eniro → PAE + longa reĝimo → 64-bita eniro.
//!
//! Du protokoloj kondukas ĉi tien:
//! - QEMU `-kernel`: Xen-PVH-noto (`XEN_ELFNOTE_PHYS32_ENTRY`), EBX montras al
//!   `hvm_start_info`.
//! - Limine (ISO): Multiboot2-kapo, EAX estas `0x36d76289`, EBX montras al la
//!   Multiboot2-informo.
//!
//! Ĉi tio estas kunmetita de `rustc` per `global_asm!`, do ekstera asemblero
//! ne necesas.

use core::arch::global_asm;

global_asm!(
    r#"
    .section .multiboot2_header, "a"
    .align 8
mb2_kapo:
    .long 0xE85250D6        /* magio */
    .long 0                 /* arkitekturo: x86 */
    .long mb2_fino - mb2_kapo
    .long -(0xE85250D6 + (mb2_fino - mb2_kapo))
    /* Petu la memormapon. */
    .short 1
    .short 0
    .long 16
    .long 6
    .long 0
    /* Petu 32-bitan grafikan framebuffer en la nuna reĝimo. */
    .short 5
    .short 0
    .long 20
    .long 0
    .long 0
    .long 32
    .align 8
    /* Fina etikedo. */
    .short 0
    .short 0
    .long 8
mb2_fino:

    .section .note.Xen, "a", @note
    .align 4
    .long 4                 /* namesz: "Xen\0" */
    .long 4                 /* descsz */
    .long 18                /* XEN_ELFNOTE_PHYS32_ENTRY */
    .asciz "Xen"
    .align 4
    .long _komenco
    .align 4

    .section .text
    .code32
    .global _komenco
_komenco:
    cli
    cld
    mov ebp, eax            /* MB2-magio; PVH lasas EAX nedifinita */
    mov esi, ebx            /* praŝarĝa informo */

    /* Nuligu la paĝtabelan areon: PML4 (1) + PDPT (1) + PD0..PD3 (4) = 6 paĝoj. */
    mov edi, offset _pml4
    xor eax, eax
    mov ecx, (6 * 4096) / 4
    rep stosd

    /* PML4[0] -> PDPT */
    mov edi, offset _pml4
    mov eax, offset _pdpt
    or eax, 0x3
    mov dword ptr [edi], eax

    /* PDPT[i] -> PD_i */
    mov edi, offset _pdpt
    mov eax, offset _pd0
    or eax, 0x3
    mov dword ptr [edi + 0], eax
    mov eax, offset _pd1
    or eax, 0x3
    mov dword ptr [edi + 8], eax
    mov eax, offset _pd2
    or eax, 0x3
    mov dword ptr [edi + 16], eax
    mov eax, offset _pd3
    or eax, 0x3
    mov dword ptr [edi + 24], eax

    /* Plenigu la 4 PD-ojn per 2 MiB da grandegaj paĝoj: identeca mapo de la
       unuaj 4 GiB. */
    xor ecx, ecx
    xor edx, edx
    mov edi, offset _pd0
.Lplenigu_pd:
    mov eax, edx
    or  eax, 0x83
    mov dword ptr [edi], eax
    add edi, 8
    add edx, 0x200000
    inc ecx
    cmp ecx, 512 * 4
    jne .Lplenigu_pd

    /* CR3 = PML4 */
    mov eax, offset _pml4
    mov cr3, eax

    /* CR4.PAE */
    mov eax, cr4
    or  eax, (1 << 5)
    mov cr4, eax

    /* EFER.LME */
    mov ecx, 0xC0000080
    rdmsr
    or  eax, (1 << 8)
    wrmsr

    /* CR0.PG | CR0.PE */
    mov eax, cr0
    or  eax, 0x80000001
    mov cr0, eax

    /* Ŝargu 64-bitan GDT kaj saltu en 64-bitan kodon. */
    lgdt [komenca_gdt_montrilo]
    .byte 0xEA
    .long .Lstarto64
    .word 0x08

    .code64
.Lstarto64:
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax

    /* Nuligu .bss */
    lea rdi, [rip + __bss_komenco]
    lea rcx, [rip + __bss_fino]
    sub rcx, rdi
    xor al, al
    rep stosb

    lea rsp, [rip + _staka_supro]

    /* Argumentoj: la informa montrilo kaj la protokola magio. */
    mov edi, esi
    mov esi, ebp
    xor ebp, ebp
    call kerna_eniro

.Lhalti64:
    cli
    hlt
    jmp .Lhalti64

    .section .rodata
    .align 8
komenca_gdt:
    .quad 0
    .quad 0x00209A0000000000
    .quad 0x0000920000000000
komenca_gdt_fino:
komenca_gdt_montrilo:
    .word komenca_gdt_fino - komenca_gdt - 1
    .quad komenca_gdt

    .section .boot_bss, "aw", @nobits
    .align 4096
_pml4:
    .space 4096
_pdpt:
    .space 4096
_pd0:
    .space 4096
_pd1:
    .space 4096
_pd2:
    .space 4096
_pd3:
    .space 4096

    .align 16
_stako:
    .space 16384
_staka_supro:
"#
);
