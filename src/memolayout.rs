use crate::riscv::{MAXVA, PGSIZE};

// You can found those address by dump qemu dtb, and use dtc to get dts file,
// which describe virtual machine's memory layout.
// we manually write this for simplicity.
pub const UART: usize = 0x1000_0000;

pub const FW_CFG: usize = 0x1010_0000;
pub const FW_CFG_SIZE: usize = 0x18;

pub const KERNELBASE: usize = 0x8000_0000;
pub const PHYSTOP: usize = KERNELBASE + 128 * 1024 * 1024;

// core local interruptor (CLINT), which contains the timer.
pub const CLINT: usize = 0x200_0000;
pub const CLINT_MTIME: usize = CLINT + 0xBFF8;

// qemu puts platform-level interrupt controller (PLIC) here.
pub const PLIC: usize = 0x0c000000;

pub const TRAMPOLINE: usize = MAXVA as usize - PGSIZE;
pub const TRAPFRAME: usize = TRAMPOLINE - PGSIZE;
// virtio mmio interface
pub const VIRTIO0: usize = 0x10001000;
pub const VIRTIO0_IRQ: usize = 1;
pub const  UART_IRQ: usize = 10;
extern "C" {
    static end: u8;
    static etext: u8;
    fn kernelvec();
    fn trampoline();
    fn uservec();
    fn userret();
}

pub fn get_kernel_end() -> usize {
    unsafe {
        return (&end as *const u8) as usize;
    }
}

pub fn get_etext() -> usize {
    unsafe {
        return (&etext as *const u8) as usize;
    }
}
pub fn get_kernelvec() -> usize {
    return kernelvec as usize;
}

pub fn get_trampoline() -> usize {
    return trampoline as usize;
}

pub fn get_uservec() -> usize {
    return uservec as usize;
}

pub fn get_userret() -> usize {
    return userret as usize;
}

#[inline]
pub fn clint_mtimecmp(hartid: u64) -> u64 {
    return (CLINT as u64) + 0x4000 + 8 * hartid;
}

#[macro_export]
macro_rules! KSTACK {
    ($p: expr) => {
        TRAMPOLINE - ($p + 1) * 16 * PGSIZE
    };
}

#[inline]
pub fn plic_priority() -> usize {
    PLIC + 0x0
}

#[inline]
pub fn plic_pending() -> usize {
    PLIC + 0x1000
}

#[inline]
pub fn plic_menable(hart: usize) -> usize {
    PLIC + 0x2000 + hart * 0x100
}

#[inline]
pub fn plic_senable(hart: usize) -> usize {
    PLIC + 0x2080 + hart * 0x100
}

pub fn plic_mpriority(hart: usize) -> usize {
    PLIC + 0x200000 + hart * 0x2000
}

pub fn plic_spriority(hart: usize) -> usize {
    PLIC + 0x201000 + hart * 0x2000
}

pub fn plic_mclaim(hart: usize) -> usize {
    PLIC + 0x200004 + hart * 0x2000
}

pub fn plic_sclaim(hart: usize) -> usize {
    PLIC + 0x201004 + hart * 0x2000
}
