#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(const_maybe_uninit_zeroed)]
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

use core::{arch::global_asm, panic::PanicInfo};
use linked_list_allocator::LockedHeap;
use plic::plicinithart;
use riscv::intr_on;
use virtio::virtio_blk::virtio_disk_rw;

use crate::plic::plicinit;

extern crate alloc;

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
    // virtio::init_virtio_blk_device(memolayout::VIRTIO0 as *const u8);
    
    uart::console_init();
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
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout);
}
