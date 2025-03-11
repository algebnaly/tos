    .attribute arch, "rv64gc"
    .section .text.entry
    .global _entry
_entry:
    la sp, STACK0
    li a0, 65536
    la a2, FDT_ADDR
    sd a1, 0(a2)
    csrr a1, mhartid
    addi a1, a1, 1
    mul a0, a0, a1
    add sp, sp, a0 
    call start

# setting stack for every core,
# but we need alloc stack space for it,
# and this is not done,
# we only consider single core for now
