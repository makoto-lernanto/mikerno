//! arm64-eniro: `Image`-kapo, EL2→EL1, stako, `.bss` kaj la unua Rust-voko.
//!
//! QEMU `virt` ŝargas la bildon ĉe `0x4008_0000` kaj transdonas la DTB-on en
//! `x0`. Se la praŝarĝilo lasis nin ĉe EL2, malgranda ŝelo transiras al EL1
//! antaŭ ol uzi la kernon.

use core::arch::global_asm;

global_asm!(
    r#"
    .section .text.entry, "ax"
    .global _komenco
_komenco:
    b _starto
    .word 0                         /* code1 */
    .quad 0x80000                   /* text_offset */
    .quad __kerna_fino - _komenco   /* image_size */
    .quad 0                         /* flags: 4 KiB, LE, 2 MiB-bazo */
    .quad 0                         /* res2 */
    .quad 0                         /* res3 */
    .quad 0                         /* res4 */
    .word 0x644d5241                /* magic: "ARM\x64" */
    .word 0                         /* res5 */

_starto:
    mov x19, x0                     /* DTB el la praŝarĝilo */

    mrs x9, CurrentEL
    and x9, x9, #0xC
    cmp x9, #0x8                    /* EL2? */
    b.ne .Lel1

    mov x9, #1
    lsl x9, x9, #31                 /* HCR_EL2.RW = 1 */
    msr hcr_el2, x9
    mov x9, #0x3C5                  /* SPSR_EL2: EL1h, D/A/I/F maskitaj */
    msr spsr_el2, x9
    adr x9, .Lel1
    msr elr_el2, x9
    eret

.Lel1:
    mov x9, #3
    lsl x9, x9, #20                 /* CPACR_EL1.FPEN = 11 */
    msr cpacr_el1, x9
    isb

    adrp x9, __stako_supro
    add x9, x9, :lo12:__stako_supro
    mov sp, x9

    adrp x9, __bss_komenco
    add x9, x9, :lo12:__bss_komenco
    adrp x10, __bss_fino
    add x10, x10, :lo12:__bss_fino
.Lbss:
    cmp x9, x10
    b.hs .Lbss_fino
    str xzr, [x9], #8
    b .Lbss
.Lbss_fino:

    mov x0, x19
    bl kerna_eniro_arm64

.Lhalti:
    wfi
    b .Lhalti

    .section .stako, "aw", @nobits
    .align 4
_stako:
    .space 65536
__stako_supro:
"#
);
