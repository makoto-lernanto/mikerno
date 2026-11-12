/* Horizonto: la unua uzanta bildo.
   La nuna fazo uzas unu komunan adresan spacon; la bildo estas metita je
   4 MiB, super la kerna bildo (1 MiB). Kiam la kerno akiras privatajn
   adresajn spacojn, la sama bildo estos mapita en la uzantan fenestron. */
ENTRY(horizonto_start)

SECTIONS
{
    . = 4M;

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
        *(.bss .bss.*)
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
