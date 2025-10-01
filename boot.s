# Multiboot header 

.section .multiboot
.align 4
multiboot_header:
    .long 0x1BADB002                    # Magic number, TODO change
    .long 0x00000000                    # Flags
    .long -(0x1BADB002 + 0x00000000)    # Checksum

.section .text.boot
.code32
.global _start

_start:
    cli
    
    # Set up stack below the page tables
    movl $0x00107000, %esp
    movl %esp, %ebp
    
    # Attempt to save multiboot info
    pushl %ebx
    pushl %eax
    
    # Check for CPUID support
    pushfl
    popl %eax
    movl %eax, %ecx
    xorl $0x00200000, %eax
    pushl %eax
    popfl
    pushfl
    popl %eax
    xorl %ecx, %eax
    je no_long_mode
    
    # Check for long mode support
    movl $0x80000000, %eax
    cpuid
    cmpl $0x80000001, %eax
    jb no_long_mode
    
    movl $0x80000001, %eax
    cpuid
    testl $(1 << 29), %edx
    jz no_long_mode
    
    # Clear page tables, 0x108000-0x10B000 = 12KB
    movl $0x108000, %edi
    movl $0x3000, %ecx
    xorl %eax, %eax
    rep stosl
    
    
    # PML4 at 0x108000
    movl $0x108000, %edi
    movl $0x109003, (%edi)          # PML4[0] -> PDPT (present, writable)
    
    # PDPT at 0x109000 
    movl $0x109000, %edi
    movl $0x10A003, (%edi)          # PDPT[0] -> PD for 0-1GB
    movl $0x10A003, 8(%edi)         # PDPT[1] -> PD for 1-2GB (reuse same PD)
    movl $0x10A003, 16(%edi)        # PDPT[2] -> PD for 2-3GB (reuse same PD)
    movl $0x10A003, 24(%edi)        # PDPT[3] -> PD for 3-4GB (reuse same PD)
    
    # PD at 0x10A000
    movl $0x10A000, %edi
    movl $512, %ecx                 # 512 entries = 1GB coverage
    movl $0x000083, %eax            # Start at 0, 2MB page, present, writable
    
.fill_pd:
    movl %eax, (%edi)               # Write lower 32 bits
    movl $0, 4(%edi)                # Write upper 32 bits (always 0)
    addl $0x200000, %eax            # Next 2MB page
    addl $8, %edi                   # Next PD entry (8 bytes)
    loop .fill_pd
    
    # Enable PAE
    movl %cr4, %eax
    orl $0x20, %eax
    movl %eax, %cr4
    
    # Load PML4 address into CR3
    movl $0x108000, %eax
    movl %eax, %cr3
    
    # Enable long mode in EFER MSR
    movl $0xC0000080, %ecx
    rdmsr
    orl $0x00000100, %eax
    wrmsr
    
    # Enable paging and protected mode
    movl %cr0, %eax
    orl $0x80000001, %eax
    movl %eax, %cr0
    
    # Load GDT
    lgdt (gdt_descriptor)
    
    ljmp $0x08, $long_mode_start

no_long_mode:
    # No 64-bit Error print
    movl $0xb8000, %edi
    movl $0x4f524f45, (%edi)      # 'ER'
    movl $0x4f3a4f52, 4(%edi)     # 'R:'
    movl $0x4f4e4f20, 8(%edi)     # ' N'
    movl $0x4f364f6f, 12(%edi)    # 'o6'
    movl $0x4f2d4f34, 16(%edi)    # '4-'
    movl $0x4f694f62, 20(%edi)    # 'bi'
    movl $0x4f214f74, 24(%edi)    # 't!'
.halt32:
    hlt
    jmp .halt32

.code64
long_mode_start:
    # Set up segment registers
    movw $0x10, %ax
    movw %ax, %ds
    movw %ax, %es
    movw %ax, %fs
    movw %ax, %gs
    movw %ax, %ss
    
    # Set up stack in 64-bit mode below page tables
    movq $0x00107000, %rsp
    
    # Set up IDT
    movq $0x10B000, %rdi
    movq $256, %rcx
    movq $default_handler, %rax
    
    movw %ax, %r8w              # offset_low
    shrq $16, %rax
    movw %ax, %r9w              # offset_mid
    shrq $16, %rax
    movl %eax, %r10d            # offset_high
    
.fill_idt_loop:
    movw %r8w, (%rdi)           # offset_low
    movw $0x08, 2(%rdi)         # segment selector
    movb $0, 4(%rdi)            # IST
    movb $0x8E, 5(%rdi)         # type (interrupt gate, present)
    movw %r9w, 6(%rdi)          # offset_mid
    movl %r10d, 8(%rdi)         # offset_high
    movl $0, 12(%rdi)           # reserved
    
    addq $16, %rdi
    loop .fill_idt_loop
    
    # Load IDT
    lidt (idt_descriptor)
    
    # Clear BSS section
    movq $__bss_start, %rdi
    movq $__bss_end, %rcx
    subq %rdi, %rcx
    xorb %al, %al
    rep stosb
    
    # Restore multiboot info
    popq %rdi  # Multiboot stuff
    popq %rsi  # Multiboot info
    
    # Call kernel
    call kernel_main
    
    # Should never return, but just in case
.halt64:
    cli
    hlt
    jmp .halt64

# Default exception handler
default_handler:
    # Return, kernel should handle its own exceptions
    iretq

# GDT for 64-bit mode
.align 16
gdt_start:
    .quad 0x0000000000000000    # Null descriptor
    .quad 0x00209A0000000000    # Code segment
    .quad 0x0000920000000000    # Data segment
gdt_end:

gdt_descriptor:
    .word gdt_end - gdt_start - 1
    .quad gdt_start

# IDT descriptor
idt_descriptor:
    .word 256 * 16 - 1
    .quad 0x10B000

.section .bss
.align 16
.comm __bss_start, 0
.comm __bss_end, 0