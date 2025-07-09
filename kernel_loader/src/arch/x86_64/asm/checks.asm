global check_multiboot:function
global check_cpuid:function
global check_long_mode:function
global check_huge_pages:function

section .data
bits 32
    no_multiboot_error: db "NO MULTIBOOT", 0
    no_cpuid_error: db "NO CPUID", 0
    no_longmode_error: db "NO LONGMODE", 0
    no_pse_error: db "NO PSE (2MiB pages)", 0
    no_pdpe_error: db "NO PDPE1GB (1GiB pages)", 0

section .text
bits 32

; Prints `ERR: ` and the given error to screen and hangs.
; parameter: address to error string (null terminated in ascii) in eax
error:
    ; first clear screen using 32 bit operations
    push eax

    mov eax, 0x07200720
    mov ecx, 80 * 25 * 2
    mov edi, 0xB8000
    rep stosd

    pop eax

    ; prints "ERR: "
    mov dword [0xb8000], 0x4f524f45
    mov dword [0xb8004], 0x4f3a4f52
    mov dword [0xb8008], 0x4f204f20
    ; eax: pointer within string
    ; ebx: pointer within video memory
    ; cx:
    ;   high byte: colour (always 0x4F)
    ;   low byte:  ascii char from [eax]
    mov ebx, 0xb800a
    mov ch, 0x4F
.loop:
    mov byte cl, [eax]
    test cl, cl
    jz .end
    mov word [ebx], cx
    inc eax
    lea ebx, [ebx + 2]
    jmp .loop
.end:
    hlt

; Checks that eax contains the magic value that indicates
; OS image was booted from a multiboot2 compliant bootloader
check_multiboot:
	cmp eax, 0x36d76289 ; if multiboot then eax will contain this magic value
	jne .no_multiboot
	ret
.no_multiboot:
    mov eax, no_multiboot_error
	jmp error


; Check if CPUID is supported by attempting to flip the ID bit (bit 21)
; in the FLAGS register. If we can flip it, CPUID is available.
check_cpuid:
	; Copy FLAGS in to EAX via stack
	pushfd
	pop eax

	; Copy to ECX as well for comparing later on
	mov ecx, eax

	; Flip the ID bit
	xor eax, 1 << 21

	; Copy EAX to FLAGS via the stack
	push eax
	popfd

	; Copy FLAGS back to EAX (with the flipped bit if CPUID is supported)
	pushfd
	pop eax

	; Restore FLAGS from the old version stored in ECX (i.e. flipping the
	; ID bit back if it was ever flipped).
	push ecx
	popfd

	; Compare EAX and ECX. If they are equal then that means the bit
	; wasn't flipped, and CPUID isn't supported.
	cmp eax, ecx
	je .no_cpuid
	ret
.no_cpuid:
    mov eax, no_cpuid_error
	jmp error

; Checks that long mode is available
check_long_mode:
	mov eax, 0x80000000
	cpuid
	cmp eax, 0x80000001
	jb .no_long_mode

	mov eax, 0x80000001
	cpuid
	test edx, 1 << 29
	jz .no_long_mode
	ret
.no_long_mode:
    mov eax, no_longmode_error
	jmp error

; check that 2MiB and 1GiB pages are supported
check_huge_pages:
	mov eax, 0x80000001
	cpuid

	test edx, 1 << 3 ; pse (2MiB pages)
	jz .no_pse

	test edx, 1 << 26 ; pdpe1gb (1GiB pages)
	jz .no_pdpe

    ret
.no_pse:
    mov eax, no_pse_error
    jmp error
.no_pdpe:
    mov eax, no_pdpe_error
    jmp error
