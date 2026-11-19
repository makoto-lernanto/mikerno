/* Saluto: eta uzanta bildo por `lanĉi`.
   La bildo estas ligita je la bazo de la uzanta fenestro (4 GiB), kiun la
   kerno mapas per la bito `U`. La stako venas de la kerno, ne de ĉi tiu
   skripto. */
ENTRY(saluto_komenco)

SECTIONS
{
    . = 0x100000000;

    .text : ALIGN(4K)
    {
        *(.text .text.*)
    }

    .rodata : ALIGN(4K)
    {
        *(.rodata .rodata.* .lrodata .lrodata.*)
    }

    .data : ALIGN(4K)
    {
        *(.data .data.* .ldata .ldata.*)
        *(.got .got.plt)
    }

    .bss : ALIGN(4K)
    {
        *(.bss .bss.* .lbss .lbss.*)
        *(COMMON)
    }

    /DISCARD/ :
    {
        *(.eh_frame)
        *(.eh_frame_hdr)
        *(.note .note.*)
        *(.comment)
    }
}
