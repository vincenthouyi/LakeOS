
PHYS_BASE = 0x80000;
KERNEL_BASE = 0xffff800000080000;
KERNEL_OFFSET = KERNEL_BASE - PHYS_BASE;

ENTRY(_start);

SECTIONS
{
    /* _begin = PHYS_BASE; */
    . = KERNEL_BASE;
    
    .text : AT(ADDR(.text) - KERNEL_OFFSET) {
        *(.text*)
    }
    _text_end = .;

    .rodata : AT(ADDR(.rodata) - KERNEL_OFFSET) {
        *(.rodata*)
    }
    
    . = ALIGN(4K);
    
    .data : AT(ADDR(.data) - KERNEL_OFFSET) {
        *(.data*)
    }
    _data_end = .;
    
    .bss (NOLOAD) : AT(ADDR(.bss)) {
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

    PROVIDE(__KERNEL_END__ = ALIGN(4096));

    /* /DISCARD/ : { *(.comment) *(.gnu*) *(.note*) *(.eh_frame*) *(.debug*)} */
}
