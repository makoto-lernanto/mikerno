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

    /* Sekciaj limoj por W^X: la identeca mapado donas al `.text` nur
       legadon kaj plenumon, al `.rodata` nur legadon, kaj al la cetera bildo
       legadon kaj skribadon kun NX. Ĉiu limo estas paĝ-aliniiĝinta, por ke
       la mapoj ne dividu unu paĝon inter du permesoj. */
    . = ALIGN(4K);
    __kerna_kodo_komenco = .;
    .text : ALIGN(4K)
    {
        /* `code-model=large` metas multajn funkciojn en `.ltext.*`; sen ĉi tiu
           aranĝo ili fariĝus neordigitaj sekcioj antaŭ `.text` kaj ricevus la
           permesojn de nurlega antaŭkodo anstataŭ RX. */
        *(.text .text.* .ltext .ltext.*)
    }
    . = ALIGN(4K);
    __kerna_kodo_fino = .;

    __kerna_rodata_komenco = .;
    /* `code-model=large` metas datumojn en `.lrodata`/`.ldata`/`.lbss`; ili
       devas resti ene de la bildo, alie `__kerna_fino` ne kovrus ilin. */
    .rodata : ALIGN(4K)
    {
        *(.rodata .rodata.* .lrodata .lrodata.*)
    }
    . = ALIGN(4K);
    __kerna_rodata_fino = .;

    __kerna_datumoj_komenco = .;
    .data : ALIGN(4K)
    {
        *(.data .data.* .ldata .ldata.*)
        *(.got .got.plt)
    }

    .bss : ALIGN(4K)
    {
        __bss_komenco = .;
        *(.bss .bss.* .lbss .lbss.*)
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
