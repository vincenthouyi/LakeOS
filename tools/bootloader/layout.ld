
PHYS_BASE = 0x80000;

ENTRY(_start);

SECTIONS
{
    
    . = PHYS_BASE;
    .text : {
        *(.text.startup*)
        *(.text*)
    }
    _text_end = .;

    .rodata : {
        *(.rodata*)
    }
    
    . = ALIGN(4K);
    
    .data : {
        *(.data*)
    }
    _data_end = .;
    
    .bss (NOLOAD) : {
        . = ALIGN(32);
        __bss_start = ADDR(.bss);
        *(.bss .bss.*)
        *(COMMON)
        . = ALIGN(8);
        __bss_end = .;
    }

    /* end of the binary */
    PROVIDE(_end = ALIGN(8));

    /* number of bytes in BSS section and complete binary */
    __bss_length = (__bss_end - __bss_start);
    __binary_length = (_end - _start);

    /* /DISCARD/ : { *(.comment) *(.gnu*) *(.note*) *(.eh_frame*) *(.debug*)} */
}
