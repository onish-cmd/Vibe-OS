.section .text._start
.global _start

_start:
    mv s1, a1
    mv s0, a0
    bnez s0, park

    la sp, __stack_top

    la t0, __bss_start
    la t1, __bss_end
bss_clear:
    bge t0, t1, launch
    sd zero, 0(t0)
    addi t0, t0, 8
    j bss_clear

launch:
    mv a0, s0
    mv a1, s1
    
    tail kmain

park:
    wfi
    j park

.section .bss.stack
.align 16
stack_base:
    .space 4096 * 4
__stack_top:
