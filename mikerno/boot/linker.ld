ENTRY(_komenco)

SECTIONS
{
    . = 1M;
    __kerna_komenco = .;

    /* Praŝarĝa ŝelko: 32-bita eniro, GDT kaj la unuaj paĝtabeloj. La Xen-PVH
       noto enhavas la literalon 0x100000, do `_komenco` devas resti ĉe la
       unua bajto de la bildo. */
    .boot_text : ALIGN(16)
    {
        KEEP(*(.boot_text))
    }

    .note.Xen : ALIGN(4)
    {
        KEEP(*(.note.Xen))
    }

    /* Relokoj de la pozicio-sendependa bildo. La KASLR-ŝelko legas ilin el la
       liga kopio (1 MiB) antaŭ ol salti en la relokitan bildon. */
    .rela.dyn : ALIGN(8)
    {
        __rela_komenco = .;
        *(.rela.dyn)
        *(.rela.*)
        __rela_fino = .;
    }

    .dynamic : ALIGN(8)
    {
        *(.dynamic)
    }
    .dynsym : ALIGN(8)
    {
        *(.dynsym)
    }
    .dynstr : ALIGN(8)
    {
        *(.dynstr)
    }
    .hash : ALIGN(8)
    {
        *(.hash)
    }
    .gnu.hash : ALIGN(8)
    {
        *(.gnu.hash)
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
        . = ALIGN(4K);
        __bss_fino = .;
        /* Ene de la sekcio, do la simbolo ricevas la saman RELATIVE-relokon
           kiel la ceteraj limoj kaj sekvas la hazardan bazadreson. */
        __kerna_fino = .;
    }

    /DISCARD/ :
    {
        *(.eh_frame)
        *(.eh_frame_hdr)
        *(.note .note.*)
        *(.comment)
    }
}

/* La PVH-noto kaj la unua paĝo de la ŝelko devas koincidi kun la liga bazo. */
ASSERT(_komenco == 1M, "la enirpunkto devas esti ĉe 1 MiB por la PVH-noto")
