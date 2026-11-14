/* arm64-bildo: plata `Image` ĉe 0x4008_0000 (RAM-bazo 0x4000_0000 +
   text_offset 0x80000, kiel en la arm64-bootprotokolo). */
ENTRY(_komenco)

SECTIONS
{
    . = 0x40080000;
    __kerna_komenco = .;

    /* La unuaj 64 bajtoj estas la `Image`-kapo; `_komenco` mem estas la
       unua instrukcio (`b _starto`). */
    .text : ALIGN(16)
    {
        KEEP(*(.text.entry))
        *(.text .text.*)
    }
    . = ALIGN(4K);

    .rodata : ALIGN(4K)
    {
        *(.rodata .rodata.*)
    }
    . = ALIGN(4K);

    .data : ALIGN(4K)
    {
        *(.data .data.*)
        *(.got .got.plt)
    }

    .bss : ALIGN(4K)
    {
        __bss_komenco = .;
        *(.bss .bss.*)
        *(COMMON)
        . = ALIGN(4K);
        __bss_fino = .;
    }

    __kerna_fino = .;

    /DISCARD/ :
    {
        *(.eh_frame)
        *(.eh_frame_hdr)
        *(.note .note.*)
        *(.comment)
        *(.ARM.attributes)
    }
}

ASSERT(_komenco == 0x40080000, "la Image-kapo devas esti ĉe 0x40080000")
