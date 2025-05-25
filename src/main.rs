#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![allow(dead_code, non_upper_case_globals)]

mod mem_utils;
mod memolayout;
mod params;
mod plic;
mod proc;
mod riscv;
mod spin_lock;
mod start;
mod syscall;
mod trap;
mod uart;
mod utils;
mod virtio;
mod vm;
mod fw_cfg;
mod ramfb;


use core::{arch::global_asm, panic::PanicInfo};
use fw_cfg::{test_fw_cfg, test_iter_fwcfg};
use linked_list_allocator::LockedHeap;
use plic::plicinithart;
use ramfb::{ramfb_clear, setup_ramfb, RAMFB_OK};
use riscv::intr_on;
extern crate alloc;

use crate::plic::plicinit;

global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("trampoline.asm"));
global_asm!(include_str!("kernelvec.asm"));
global_asm!(include_str!("switch.asm"));

#[no_mangle]
static STACK0: StackWrapper = StackWrapper([0; 65536]);

#[repr(align(65536))]
struct StackWrapper([u8; 65536]);

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

#[no_mangle]
pub extern "C" fn main() -> ! {
    let heap_start = crate::memolayout::get_kernel_end();
    let heap_end = crate::memolayout::PHYSTOP;
    let heap_size = heap_end - heap_start;
    unsafe {
        ALLOCATOR.lock().init(heap_start, heap_size);
    }
    uart::console_init();
    setup_ramfb();
    ramfb_clear(0xf0_ff_ff_00);
    plicinit();
    plicinithart();
    vm::kvminit();
    vm::kvminithart();
    proc::procinit();
    trap::trapinithart();
    proc::userinit();
    intr_on();
    // virtio_disk_rw([0x75; 1024], true);
    proc::scheduler();
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("{}", _info);
    loop {}
}

#[alloc_error_handler]
fn alloc_error_handler(layout: core::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout);
}
