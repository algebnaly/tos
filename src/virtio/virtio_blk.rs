use core::mem::size_of;
use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering::Relaxed;

use super::MMIODeviceLagacyRegisterLayout;
use crate::memolayout::{self, VIRTIO0};
use crate::println;
use crate::riscv::PGSIZE;
use lazy_static::lazy_static;
use spin::Mutex;

use super::{VIRTIO_F_EVENT_IDX, VIRTIO_F_INDIRECT_DESC};

lazy_static! {
    pub static ref DISK: Mutex<Disk> = Mutex::new(Disk {
        desc: 0 as *mut VirtqDesc,
        avail: 0 as *mut VirtqAvail,
        used: 0 as *mut VirtqUsed,
        free: [true; QUEUE_NUM],
        used_idx: 0,
        info: [DiskInfo {
            b: 0 as *mut DiskBuffer,
            status: 0
        }; QUEUE_NUM],
        ops: [VirtqBlkReq {
            type_filed: 0,
            reserved: 0,
            sector: 0,
        }; QUEUE_NUM],
    });
}
pub static mut is_finish_rw: AtomicBool = AtomicBool::new(true);

pub const BSIZE: usize = 1024;

pub const DEVICE_ID: u32 = 0x2;
pub const VENDOR_ID: u32 = 0x554d4551;

pub const QUEUE_NUM: usize = 8;

pub const DISK_PAGES_LEN: usize = 2 * PGSIZE;

pub const VIRTIO_BLK_T_IN: u32 = 0; //read the disk
pub const VIRTIO_BLK_T_OUT: u32 = 1; //write the disk

pub const VIRTIO_BLK_F_BARRIER: u32 = 1 << 0;
pub const VIRTIO_BLK_F_SIZE_MAX: u32 = 1 << 1;
pub const VIRTIO_BLK_F_SEG_MAX: u32 = 1 << 2;
pub const VIRTIO_BLK_F_GEOMETRY: u32 = 1 << 4;
pub const VIRTIO_BLK_F_RO: u32 = 1 << 5;
pub const VIRTIO_BLK_F_BLK_SIZE: u32 = 1 << 6;
pub const VIRTIO_BLK_F_SCSI: u32 = 1 << 7;
pub const VIRTIO_BLK_F_FLUSH: u32 = 1 << 9;
pub const VIRTIO_BLK_F_TOPOLOGY: u32 = 1 << 10;
pub const VIRTIO_BLK_F_CONFIG_WCE: u32 = 1 << 11;
pub const VIRTIO_BLK_F_MQ: u32 = 1 << 12;
pub const VIRTIO_BLK_F_DISCARD: u32 = 1 << 13;
pub const VIRTIO_BLK_F_WRITE_ZEROES: u32 = 1 << 14;
pub const VIRTIO_BLK_F_LIFETIME: u32 = 1 << 15;
pub const VIRTIO_BLK_F_SECURE_ERASE: u32 = 1 << 16;
pub const VIRTIO_F_NOTIFY_ON_EMPTY: u32 = 1 << 24;
pub const VIRTIO_F_ANY_LAYOUT: u32 = 1 << 27;
pub const VIRTIO_UNUSED: u32 = 1 << 30;
pub fn list_feature(feature_bits: u32) {
    if feature_bits & VIRTIO_BLK_F_BARRIER != 0 {
        //        println!("VIRTIO_BLK_F_BARRIER");
    }
    if feature_bits & VIRTIO_BLK_F_BLK_SIZE != 0 {
        //        println!("VIRTIO_BLK_F_BLK_SIZE");
    }
    if feature_bits & VIRTIO_BLK_F_CONFIG_WCE != 0 {
        //        println!("VIRTIO_BLK_F_CONFIG_WCE");
    }
    if feature_bits & VIRTIO_BLK_F_DISCARD != 0 {
        //        println!("VIRTIO_BLK_F_DISCARD");
    }
    if feature_bits & VIRTIO_BLK_F_FLUSH != 0 {
        //        println!("VIRTIO_BLK_F_FLUSH");
    }
    if feature_bits & VIRTIO_BLK_F_GEOMETRY != 0 {
        //        println!("VIRTIO_BLK_F_GEOMETRY");
    }
    if feature_bits & VIRTIO_BLK_F_LIFETIME != 0 {
        //        println!("VIRTIO_BLK_F_LIFETIME");
    }
    if feature_bits & VIRTIO_BLK_F_MQ != 0 {
        //        println!("VIRTIO_BLK_F_MQ");
    }
    if feature_bits & VIRTIO_BLK_F_RO != 0 {
        //        println!("VIRTIO_BLK_F_RO");
    }
    if feature_bits & VIRTIO_BLK_F_SCSI != 0 {
        //        println!("VIRTIO_BLK_F_SCSI");
    }
    if feature_bits & VIRTIO_BLK_F_SECURE_ERASE != 0 {
        //        println!("VIRTIO_BLK_F_SECURE_ERASE");
    }
    if feature_bits & VIRTIO_BLK_F_SEG_MAX != 0 {
        //        println!("VIRTIO_BLK_F_SEG_MAX");
    }
    if feature_bits & VIRTIO_BLK_F_SIZE_MAX != 0 {
        //        println!("VIRTIO_BLK_F_SIZE_MAX");
    }
    if feature_bits & VIRTIO_BLK_F_TOPOLOGY != 0 {
        //        println!("VIRTIO_BLK_F_TOPOLOGY");
    }
    if feature_bits & VIRTIO_BLK_F_WRITE_ZEROES != 0 {
        //        println!("VIRTIO_BLK_F_WRITE_ZEROES");
    }
    if feature_bits & VIRTIO_F_ANY_LAYOUT != 0 {
        //        println!("VIRTIO_F_ANY_LAYOUT");
    }
    if feature_bits & VIRTIO_F_NOTIFY_ON_EMPTY != 0 {
        //        println!("VIRTIO_F_NOTIFY_ON_EMPTY");
    }
    if feature_bits & VIRTIO_F_EVENT_IDX != 0 {
        //        println!("VIRTIO_F_EVENT_IDX");
    }
    if feature_bits & VIRTIO_F_INDIRECT_DESC != 0 {
        //        println!("VIRTIO_F_INDIRECT_DESC");
    }
    if feature_bits & VIRTIO_UNUSED != 0 {
        //        println!("VIRTIO_UNUSED");
    }
}

#[repr(C)]
pub struct Disk {
    pub desc: *mut VirtqDesc,
    pub avail: *mut VirtqAvail,
    pub used: *mut VirtqUsed,
    pub free: [bool; QUEUE_NUM],
    pub used_idx: u16,
    pub info: [DiskInfo; QUEUE_NUM],
    pub ops: [VirtqBlkReq; QUEUE_NUM],
}

unsafe impl Send for Disk {}

#[repr(C)]
pub struct VirtqDesc {
    pub addr: u64,
    pub len: u32,
    pub flags: u16,
    pub next: u16,
}

#[repr(C)]
pub struct VirtqAvail {
    pub flags: u16,
    pub idx: u16,
    pub ring: [u16; QUEUE_NUM],
    pub unused: u16,
}

#[repr(C)]
pub struct VirtqUsedElement {
    pub id: u32,
    pub len: u32,
}

#[repr(C)]
pub struct VirtqUsed {
    pub flags: u16,
    pub idx: u16,
    pub ring: [VirtqUsedElement; QUEUE_NUM],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct VirtqBlkReq {
    pub type_filed: u32,
    pub reserved: u32,
    pub sector: u64,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DiskInfo {
    pub b: *mut DiskBuffer,
    pub status: u8,
}

pub struct DiskBuffer {
    valid: bool,
    disk: bool, // does disk "own" buf?
    // some filed omit
    data: [u8; BSIZE],
}

pub fn virtio_disk_intr() {
    // let _disk = DISK.lock();
    // the device won't raise another interrupt until we tell it
    // we've seen this interrupt, which the following line does.
    // this may race with the device writing new entries to
    // the "used" ring, in which case we may process the new
    // completion entries in this interrupt, and have nothing to do
    // in the next interrupt, which is harmless.
    let _dev_reg_ref = unsafe { &mut *(VIRTIO0 as u64 as *mut MMIODeviceLagacyRegisterLayout) };

    unsafe {
        is_finish_rw.store(true, Relaxed);
    }
}

pub fn virtio_disk_rw(data: [u8; BSIZE], write: bool) {
    let sector = 0;
    let mut disk_ref = DISK.lock();

    // format the three descriptors.
    // qemu's virtio-blk.c reads them.
    let buf0 = &mut disk_ref.ops[0];
    let buf0_addr = buf0 as *const VirtqBlkReq as u64;
    if write {
        buf0.type_filed = VIRTIO_BLK_T_OUT; // write the disk
    } else {
        buf0.type_filed = VIRTIO_BLK_T_IN; // read the disk
    }

    // the spec's Section 5.2 says that legacy block operations use
    // three descriptors: one for type/reserved/sector, one for the
    // data, one for a 1-byte status result.

    buf0.reserved = 0;
    buf0.sector = sector;
    let desc_array = unsafe { &mut *(disk_ref.desc as *mut [VirtqDesc; QUEUE_NUM]) };
    desc_array[0].addr = buf0_addr;
    desc_array[0].len = size_of::<VirtqBlkReq>() as u32;
    desc_array[0].flags = 1;
    desc_array[0].next = 1;

    desc_array[1].addr = &data as *const [u8; BSIZE] as u64;
    desc_array[1].len = BSIZE as u32;
    if write {
        desc_array[1].flags = 0;
    }
    desc_array[1].flags |= 1;
    desc_array[1].next = 2;

    disk_ref.info[0].status = 0xff;
    desc_array[2].addr = &disk_ref.info[0].status as *const u8 as u64;
    desc_array[2].len = 1;
    desc_array[2].flags = 2;
    desc_array[2].next = 0;

    let mut avail_ref = unsafe { &mut *disk_ref.avail };
    avail_ref.ring[avail_ref.idx as usize % QUEUE_NUM] = 0;
    avail_ref.idx += 1;
    let dev_reg_ref =
        unsafe { &mut *(memolayout::VIRTIO0 as u64 as *mut MMIODeviceLagacyRegisterLayout) };
    unsafe {
        is_finish_rw.store(false, Relaxed);
    }
    dev_reg_ref.queue_notify = 0; // start device r/w operation
    unsafe { while !is_finish_rw.load(Relaxed) {} }
}
