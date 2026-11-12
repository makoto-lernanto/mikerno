ENTRY(_komenco)

SECTIONS
{
    . = 1M;
    __kerna_komenco = .;

    .multiboot2_header : ALIGN(8)
    {
        KEEP(*(.multiboot2_header))
    }

    .note.Xen : ALIGN(4)
    {
        KEEP(*(.note.Xen))
    }

    .text : ALIGN(4K)
    {
        *(.text .text.*)
    }

    .rodata : ALIGN(4K)
    {
        *(.rodata .rodata.*)
    }

    .data : ALIGN(4K)
    {
        *(.data .data.*)
    }

    .bss : ALIGN(4K)
    {
        __bss_komenco = .;
        *(.bss .bss.*)
        *(COMMON)
        __bss_fino = .;
    }

    /* Praŝarĝaj paĝtabeloj kaj stako. Ili ne rajtas esti en
       [__bss_komenco, __bss_fino): la longreĝima eniro nuligas tiun aron jam
       uzante ĝuste ĉi tiujn paĝojn. */
    .boot_bss (NOLOAD) : ALIGN(4K)
    {
        *(.boot_bss .boot_bss.*)
    }

    __kerna_fino = ALIGN(4K);

    /DISCARD/ :
    {
        *(.eh_frame)
        *(.eh_frame_hdr)
        *(.note .note.*)
        *(.comment)
    }
}
