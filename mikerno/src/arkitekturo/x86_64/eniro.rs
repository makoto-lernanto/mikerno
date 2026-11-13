//! Praŝarĝa ŝelko: 32-bita eniro → PAE + longa reĝimo → 64-bita eniro.
//!
//! Du protokoloj kondukas ĉi tien:
//! - QEMU `-kernel`: Xen-PVH-noto (`XEN_ELFNOTE_PHYS32_ENTRY`), EBX montras al
//!   `hvm_start_info`.
//! - Limine (ISO): Multiboot2-kapo, EAX estas `0x36d76289`, EBX montras al la
//!   Multiboot2-informo.
//!
//! La bildo estas pozicio-sendependa (ET_DYN): absolutaj 32-bitaj simbolaj
//! relokoj estas malpermesitaj. Tial la ŝelko trovas sian kernan bazon per
//! `call`/`pop` (`.Lbaza` sekvas la `call` sen plenigaĵo, do la korekto estas
//! la nombra longo de la prologo), kaj la paĝtabeloj, la GDT kaj la
//! Multiboot2-kapo havas fiksajn deŝovojn ene de `.boot_text`:
//!
//! | deŝovo  | enhavo                                  |
//! |---------|-----------------------------------------|
//! | `0x200` | GDT (3 eroj)                            |
//! | `0x220` | GDT-montrilo (skribata dum la starto)   |
//! | `0x280` | Multiboot2-kapo                         |
//! | `0x1000`| PML4, PDPT, PD0..PD3                    |
//! | `0x7000`| 64-bita eniro kaj la salto al la kopio  |
//!
//! Post la unua kernfunkcio la bildo eble estas kopiita al hazarda bazo;
//! `kerna_salto_al_kopio` faras tiun transiron sen reveni.

use core::arch::global_asm;

global_asm!(
    r#"
    .section .boot_text, "ax"
    .code32
    .global _komenco
_komenco:
    mov ebp, eax            /* MB2-magio; PVH lasas EAX nedifinita */
    mov esi, ebx            /* praŝarĝa informo */
    call .Lbaza

    /* `call` ĵetas la adreson de la sekva instrukcio; `.Lbaza` estas tiu
       adreso mem. La prologo estas 9 bajtoj (`mov ebp,eax` 2 + `mov esi,ebx`
       2 + `call rel32` 5), do la subtraho donas la bildan bazon. */
.Lbaza:
    pop ebx
    sub ebx, 9              /* EBX = adreso de _komenco */
    cli
    cld

    /* Nuligu la paĝtabelan areon: PML4 (1) + PDPT (1) + PD0..PD3 (4) = 6 paĝoj. */
    lea edi, [ebx + 0x1000]
    xor eax, eax
    mov ecx, (6 * 4096) / 4
    rep stosd

    /* PML4[0] -> PDPT */
    lea edi, [ebx + 0x1000]
    lea eax, [ebx + 0x2000]
    or  eax, 0x3
    mov dword ptr [edi], eax

    /* PDPT[i] -> PD_i */
    lea edi, [ebx + 0x2000]
    lea eax, [ebx + 0x3000]
    or  eax, 0x3
    mov dword ptr [edi + 0], eax
    lea eax, [ebx + 0x4000]
    or  eax, 0x3
    mov dword ptr [edi + 8], eax
    lea eax, [ebx + 0x5000]
    or  eax, 0x3
    mov dword ptr [edi + 16], eax
    lea eax, [ebx + 0x6000]
    or  eax, 0x3
    mov dword ptr [edi + 24], eax

    /* Plenigu la 4 PD-ojn per 2 MiB da grandegaj paĝoj: identeca mapo de la
       unuaj 4 GiB. */
    lea edi, [ebx + 0x3000]
    xor ecx, ecx
    xor edx, edx
.Lplenigu_pd:
    mov eax, edx
    or  eax, 0x83
    mov dword ptr [edi], eax
    add edi, 8
    add edx, 0x200000
    inc ecx
    cmp ecx, 512 * 4
    jne .Lplenigu_pd

    /* Konstruu la GDT-montrilon en la skribebla fendo ĉe 0x220. */
    lea edi, [ebx + 0x220]
    mov word ptr [edi], 0x17             /* 3 eroj * 8 - 1 */
    lea eax, [ebx + 0x200]
    mov dword ptr [edi + 2], eax
    lgdt [edi]

    /* CR3 = PML4 */
    lea eax, [ebx + 0x1000]
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

    /* Fora salto en longan reĝimon: `push`/`retf` anstataŭ la absoluta
       `ljmp`-operando, kiu postulus 32-bitan relokon. */
    push 0x08
    lea eax, [ebx + 0x7000]
    push eax
    .byte 0xCB              /* retf */

    /* GDT: nula ero, 64-bita koda ero, 64-bita datumera. */
    .org 0x200
    .quad 0
    .quad 0x00209A0000000000
    .quad 0x0000920000000000

    .org 0x220
    .space 8                /* GDT-montrilo, skribata supre */

    /* Multiboot2-kapo. La protokolo nur postulas la magion en la unuaj 32 KiB
       de la dosiero, 8-bajte aliniiĝintan. */
    .org 0x280
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

    /* La unuaj paĝtabeloj: nur dum la ŝelko. Ili kuŝas ene de la bildo, do
       ankaŭ la kopio havas ilin, sed post la unua kernfunkcio ili ne plu
       estas uzataj. */
    .org 0x1000
    .space 4096             /* PML4 */
    .space 4096             /* PDPT */
    .space 4096             /* PD0: 0 .. 1 GiB */
    .space 4096             /* PD1: 1 .. 2 GiB */
    .space 4096             /* PD2: 2 .. 3 GiB */
    .space 4096             /* PD3: 3 .. 4 GiB */

    .org 0x7000
    .code64
.Lstarto64:
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax

    /* Nuligu .bss ankaŭ en la liga bildo: la unua kernfunkcio legas la
       statikojn de la praŝarĝa analizo. */
    lea rdi, [rip + __bss_komenco]
    lea rcx, [rip + __bss_fino]
    sub rcx, rdi
    xor eax, eax
    rep stosb

    lea rsp, [rip + _staka_supro]

    /* Apliku la RELATIVE-relokojn antaŭ la unua Rust-voko: la ligilo lasas la
       celajn fakojn nulaj (la aldonaĵo estas en `.rela.dyn`), do eĉ ĉe la liga
       bazo la GOT-slotoj estas malplenaj. La deŝovo estas la efektiva pozicio
       de la bildo malpli la liga bazo, do la rutino validas ankaŭ se la
       praŝarĝilo mem delokigis la ELF-on. Ĉi tiu rutino ne uzas GOT-on. */
    lea rdi, [rip + _komenco]
    sub rdi, 0x100000
    call kerna_relokigi

    /* Argumentoj: la informa montrilo kaj la protokola magio. */
    mov edi, esi
    mov esi, ebp
    xor ebp, ebp
    call kerna_eniro

.Lhalti64:
    cli
    hlt
    jmp .Lhalti64

    /* Apliku la RELATIVE-relokojn por la deŝovo en RDI. La tabelo konsistas el
       `Elf64_Rela`-eroj (24 bajtoj: r_offset, r_info, r_addend); la tipo estis
       kontrolita de la ligilo, do la ŝelko nur aldonas la deŝovon al la celo
       kaj al la aldonaĵo. RSI (la praŝarĝa informo) restas netuŝita. */
    .global kerna_relokigi
kerna_relokigi:
    lea r10, [rip + __rela_komenco]
    lea r11, [rip + __rela_fino]
.Lrelok_ciklo:
    cmp r10, r11
    jae .Lrelok_fino
    mov r8, [r10]           /* r_offset */
    mov r9, [r10 + 16]      /* r_addend */
    add r8, rdi
    add r9, rdi
    mov [r8], r9
    add r10, 24
    jmp .Lrelok_ciklo
.Lrelok_fino:
    ret

    /* Saltu al la kopio ĉe `RDX`; la aliaj argumentoj transiras al la dua
       enirpunkto. La celo estas kalkulita el la efektiva diferenco inter la
       trampolino kaj `_komenco`, do ĝi validas ankaŭ se la praŝarĝilo mem
       delokigis la bildon. */
    .global kerna_salto_al_kopio
kerna_salto_al_kopio:
    mov r12, rdi            /* starta informo */
    mov r13, rsi            /* magio */
    mov r14, rdx            /* nova bazo */
    mov r15, rcx            /* origino */
    lea rax, [rip + .Lkopio_eniro]
    lea rcx, [rip + _komenco]
    sub rax, rcx
    add rax, r14
    jmp rax

.Lkopio_eniro:
    lea rsp, [rip + _staka_supro]
    lea rdi, [rip + __bss_komenco]
    lea rcx, [rip + __bss_fino]
    sub rcx, rdi
    xor eax, eax
    rep stosb
    mov rdi, r12
    mov rsi, r13
    mov rdx, r14
    mov rcx, r15
    xor ebp, ebp
    call kerna_eniro_kopio

.Lhalti_kopio:
    cli
    hlt
    jmp .Lhalti_kopio

    .section .note.Xen, "a", @note
    .align 4
    .long 4                 /* namesz: "Xen\0" */
    .long 4                 /* descsz */
    .long 18                /* XEN_ELFNOTE_PHYS32_ENTRY */
    .asciz "Xen"
    .align 4
    /* `_komenco` estas ĉe 1 MiB (la ligilo kontrolas tion per ASSERT). La
       noto ne rajtas uzi 32-bitan simbolan relokon en PIE-bildo. */
    .long 0x100000
    .align 4

    .section .bss, "aw", @nobits
    .align 16
_stako:
    .space 16384
_staka_supro:
"#
);
