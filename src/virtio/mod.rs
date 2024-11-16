use crate::mem_utils::memset;
use crate::memolayout::PCI_BASE;
use crate::pci::find_device;
// use crate::println;
use crate::riscv::PGSIZE;
use crate::virtio::virtio_blk::{VirtqAvail, VirtqDesc, VirtqUsed, QUEUE_NUM};
use crate::vm::kalloc;
use virtio_blk::DISK;

pub mod virtio_blk;

pub const MAGIC_VALUE: u32 = 0x74726976;
pub const DEVICE_VERSION: u32 = 0x2; //use force qemu to use new virtio standard

const STATUS_ACKNOWLEDGE: u32 = 1;
const STATUS_DRIVER: u32 = 2;
const STATUS_DRIVER_OK: u32 = 4;
const STATUS_FEATURES_OK: u32 = 8;
const STATUS_DEVICE_NEEDS_RESET: u32 = 64;
const STATUS_FAILED: u32 = 128;

const VIRTIO_F_INDIRECT_DESC: u32 = 1 << 28;
const VIRTIO_F_EVENT_IDX: u32 = 1 << 29;

#[repr(C, align(4096))]
struct MMIODeviceLagacyRegisterLayout {
    magic_value: u32,         //0x000
    version: u32,             //0x004
    device_id: u32,           //0x008
    vendor_id: u32,           //0X00c
    device_features: u32,     //0X010
    device_features_sel: u32, //0x014,
    __padding_01: [u8; 8],    //0x018
    driver_features: u32,     //0x020
    driver_features_sel: u32, //0x024
    __padding_02: [u8; 8],    //0x028
    queue_sel: u32,           //0x030
    queue_num_max: u32,       //0x034
    queue_num: u32,           //0x038
    __padding_03: [u8; 8],    //0x03c
    queue_ready: u32,         //0x044
    __padding_04: [u8; 8],    //0x048
    queue_notify: u32,        //0x050
    __padding_05: [u8; 12],   //0x054
    interrupt_status: u32,    //0x060
    interrupt_ack: u32,       //0x064
    __padding_06: [u8; 8],    //0x068
    status: u32,              //0x070
    __padding_07: [u8; 12],   //0x074
    queue_desc_low: u32,      //0x080
    queue_desc_high: u32,     //0x084
    __padding_08: [u8; 8],    //0x088
    queue_driver_low: u32,    //0x090
    queue_driver_high: u32,   //0x094
    __pading_09: [u8; 8],     //0x098
    queue_device_low: u32,    //0x0a0
    queue_device_high: u32,   //0x0a4
    __padding_10: [u8; 4],    //0x0a8
    shm_sel: u32,             //0x0ac
    shm_len_low: u32,         //0x0b0
    shm_len_high: u32,        //0x0b4
    shm_base_low: u32,        //0x0b8
    shm_base_high: u32,       //0x0bc
    queue_reset: u32,         //0x0c0
    __padding_11: [u8; 56],   //0x0c4
    config_generation: u32,   //0x0fc
    config: [u8; 0x100],      //0x100
}

pub fn check_virtio_device_is_valid(reg_addr: *const u8) -> bool {
    let dev_reg_ref = unsafe { &*(reg_addr as u64 as *const MMIODeviceLagacyRegisterLayout) };
    dev_reg_ref.magic_value == MAGIC_VALUE
        && dev_reg_ref.version == DEVICE_VERSION
        && dev_reg_ref.device_id != 0x0
}

pub fn init_virtio_blk_device(dev_addr: *const u8) {
    if !check_virtio_device_is_valid(dev_addr) {
        panic!("not valid device");
    }
    let mut status;
    let dev_reg_ref = unsafe { &mut *(dev_addr as u64 as *mut MMIODeviceLagacyRegisterLayout) };
    if dev_reg_ref.device_id != virtio_blk::DEVICE_ID
        && dev_reg_ref.vendor_id != virtio_blk::VENDOR_ID
    {
        panic!("this device is not we want!");
    }

    status = 0;
    dev_reg_ref.status = status; //1. reset device
    status |= STATUS_ACKNOWLEDGE;
    dev_reg_ref.status = status; //2. set ACKNOWLEDGE bit
    status |= STATUS_DRIVER;
    dev_reg_ref.status = status; //3. set DRIVER bit

    let mut feature_bits: u32 = dev_reg_ref.device_features; //4. read features bit
    feature_bits &= !virtio_blk::VIRTIO_BLK_F_RO;
    feature_bits &= !virtio_blk::VIRTIO_BLK_F_SCSI;
    feature_bits &= !virtio_blk::VIRTIO_BLK_F_CONFIG_WCE;
    feature_bits &= !virtio_blk::VIRTIO_BLK_F_MQ;
    feature_bits &= !virtio_blk::VIRTIO_F_ANY_LAYOUT;
    feature_bits &= !VIRTIO_F_EVENT_IDX;
    feature_bits &= !VIRTIO_F_INDIRECT_DESC;

    dev_reg_ref.driver_features = feature_bits; //4. set features bit
    status |= STATUS_FEATURES_OK;
    dev_reg_ref.status = status; //5. set FEATURES_OK bit

    status = dev_reg_ref.status; // we have to use explicity write this line, so that compiler can generate 'lw' instruction, rather than 'lbu' instruction.

    if status & STATUS_FEATURES_OK == 0 {
        //6. check FEATURES_OK
        panic!("can't set FEATURES_OK");
    }

    // initialize queue 0
    dev_reg_ref.queue_sel = 0;

    // ensure queue 0 is not in use
    if dev_reg_ref.queue_ready != 0 {
        panic!("virtio disk should not be ready");
    }

    //check maximum queue size
    let max = dev_reg_ref.queue_num_max;
    if max == 0 {
        panic!("virtio disk has no queue 0");
    }
    if max < QUEUE_NUM as u32 {
        panic!("virtio disk max queue too short");
    }

    let disk_ref = &mut DISK.lock();

    disk_ref.desc = kalloc() as *mut VirtqDesc;
    disk_ref.avail = kalloc() as *mut VirtqAvail;
    disk_ref.used = kalloc() as *mut VirtqUsed;

    unsafe {
        memset(disk_ref.desc as *mut u8, 0, PGSIZE);
        memset(disk_ref.avail as *mut u8, 0, PGSIZE);
        memset(disk_ref.used as *mut u8, 0, PGSIZE);
    }

    // set queue size
    dev_reg_ref.queue_num = QUEUE_NUM as u32;

    // write physical addresses
    dev_reg_ref.queue_desc_low = disk_ref.desc as *const u8 as u32;
    dev_reg_ref.queue_desc_high = (disk_ref.desc as *const u8 as u64 >> 32) as u32;
    dev_reg_ref.queue_driver_low = disk_ref.avail as *const u8 as u32;
    dev_reg_ref.queue_driver_high = (disk_ref.avail as *const u8 as u64 >> 32) as u32;
    dev_reg_ref.queue_device_low = disk_ref.used as *const u8 as u32;
    dev_reg_ref.queue_device_high = (disk_ref.used as *const u8 as u64 >> 32) as u32;

    // queue is ready
    dev_reg_ref.queue_ready = 0x1;

    // ALL NUM descriptors start out unused
    for i in 0..QUEUE_NUM {
        disk_ref.free[i] = false;
    }

    status |= STATUS_DRIVER_OK;
    dev_reg_ref.status = status;
    // plic.rs and trap.rs arrange for interrupts from VIRTIO0_IRQ.
}

pub fn find_virtio_device(virtio_device_id: u16) -> Option<usize> {
    find_device(PCI_BASE, 0x1af4, 0x1040 + virtio_device_id)
}
