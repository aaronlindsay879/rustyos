global start, p4_table, p3_table, p3_table_phys
extern check_multiboot, check_cpuid, check_long_mode, check_huge_pages, set_up_page_tables, enable_paging, long_mode_start, check_lapic, check_x2apic

section .bss
align 4096
p4_table:
	resb 4096
p3_table:
	resb 4096
p3_table_phys:
	resb 4096
stack_bottom:
	resb 4096 * 16
stack_top:


section .rodata
gdt64:  ; set up GDT in 64 bit mode
	dq 0 ; zero entry
.code:  equ $ - gdt64
	dq (1<<43) | (1<<44) | (1<<47) | (1<<53) ; code segment
.pointer:
	dw $ - gdt64 - 1
    dq gdt64

section .text
bits 32
start:
    ; set up stack and save value of ebx for booting
    mov esp, stack_top
    push ebx
    push eax
    push edx

    ; disable cursor
    mov dx, 0x3D4
    mov al, 0xA
    out dx, al

    inc dx
    mov al, 0x20
    out dx, al

    ; restore reg values
    pop edx
    pop eax

    ; check cpu has all the features we require
    call check_multiboot
    call check_cpuid
    call check_long_mode
    call check_huge_pages
    call check_lapic

    ; set up paging
    call set_up_page_tables
    call enable_paging

    ; load the 64-bit GDT
    lgdt [gdt64.pointer]

    ; restore ebx for kernel loader, and perform far jump
    pop ebx
    jmp gdt64.code:long_mode_start
