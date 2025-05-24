use core::arch::asm;

use crate::{main, println};
use crate::riscv::*;
use crate::memolayout::{clint_mtimecmp, CLINT_MTIME};

#[no_mangle]
static mut TIMER_SCRATCH: [u64; 5] = [0; 5];

extern "C" {
    fn timervec();
}

#[no_mangle]
extern "C" fn start() {
    // set M Previous Privilege mode to Supervisor, for mret.
    let mut x: u64 = r_mstatus();
    x &= !MSTATUS_MPP_MASK;
    x |= MSTATUS_MPP_S;
    w_mstatus(x);

    //set M Exception Program Counter to main, for mret
    w_mepc(main as u64);
    w_satp(0);
    // delegate all interrupts and exceptions to supervisor mode.
    w_medeleg(0xffff);
    w_mideleg(0xffff);
    w_sie(r_sie() | SIE_SEIE | SIE_STIE | SIE_SSIE);
    // configure Physical Memory Protection to give supervisor mode
    // access to all of physical memory.
    w_pmpaddr0(0x3fffffffffffff);
    w_pmpcfg0(0xf);
    //timerinit();
    let id = r_mhartid();
    w_tp(id);
    unsafe{asm!("mret");}
}


fn timerinit(){
    let id = r_mhartid();
    let interval = 1000000; // cycles; about 1/10th second in qemu.
    let timer_addr: *mut u64 = clint_mtimecmp(id) as *mut u64;
    let mtime_addr: *mut u64 = CLINT_MTIME as *mut u64;
    unsafe {
        *timer_addr = *mtime_addr + interval;
    }
    // prepare information in scratch[] for timervec.
    // scratch[0..2] : space for timervec to save registers.
    // scratch[3] : address of CLINT MTIMECMP register.
    // scratch[4] : desired interval (in cycles) between timer interrupts.
    unsafe{
        TIMER_SCRATCH[3] = clint_mtimecmp(id);
        TIMER_SCRATCH[4] = interval;
        w_mscratch((&TIMER_SCRATCH as *const u64) as u64);
    }
    // set the machine-mode trap handler
    w_mtvec(timervec as u64);
    
    // enable machine-mode interrupts
    w_mstatus(r_mstatus() | MSTATUS_MIE);

    //enable machine-mode timer interrupts
    w_mie(r_mie() | MIE_MTIE);
    // println!("{}: itimer inited", id);
}

