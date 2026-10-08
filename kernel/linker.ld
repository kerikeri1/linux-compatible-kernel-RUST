OUTPUT_FORMAT(elf64-x86-64)
ENTRY(_start)

PHDRS {
    text   PT_LOAD FLAGS((1 << 0) | (1 << 2));
    rodata PT_LOAD FLAGS((1 << 2));
    data   PT_LOAD FLAGS((1 << 1) | (1 << 2));
}

SECTIONS {
    . = 0xffffffff80000000;

    .text   : { *(.text .text.*) } :text
    . = ALIGN(4096);
    .rodata : { *(.rodata .rodata.*) } :rodata
    . = ALIGN(4096);
    .requests : { KEEP(*(.requests_start_marker)) KEEP(*(.requests)) KEEP(*(.requests_end_marker)) } :data
    .data   : { *(.data .data.*) } :data
    .bss    : { *(.bss .bss.*) *(COMMON) } :data

    /DISCARD/ : { *(.eh_frame*) *(.note .note.*) }
}
