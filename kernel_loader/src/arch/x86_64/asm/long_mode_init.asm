global long_mode_start:function

extern loader_main

section .text
bits 64
long_mode_start:
        ; can clear screen with 64 bit operations now
        mov rax, 0x0720072007200720
        mov rcx, 80 * 25 * 2
        mov rdi, 0xB8000
        rep stosq

	    mov rdi, rbx
    	call loader_main

        cli
.hang   hlt
	    jmp .hang