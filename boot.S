.section .text
.code32

# Multiboot header
.align 4
multiboot_header:
    .long 0x1BADB002           # magic
    .long 0x00                 # flags
    .long -(0x1BADB002 + 0x00) # checksum

.global _start
.type _start, @function
_start:
    cli
    mov $stack_top, %esp
    call kernel_main
hang:
    hlt
    jmp hang

.section .bss
.align 16
stack_bottom:
    .skip 16384 # 16 KiB
stack_top:

# Damn vscode retards not bothering including highlights.