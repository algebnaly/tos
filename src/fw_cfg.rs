use core::mem::MaybeUninit;

use alloc::vec::Vec;

use crate::{
    memolayout::{FW_CFG, FW_CFG_DATA_OFFSET, FW_CFG_DMA_OFFSET, FW_CFG_SELECTOR_OFFSET},
    println,
};

pub const FW_CFG_SIGNATURE: u16 = 0x0000;
pub const FW_CFG_FW_CFG_ID: u16 = 0x0001;
pub const FW_CFG_FILE_DIR: u16 = 0x0019;

pub const FW_CFG_SIGNATURE_VAL: &[u8; 4] = b"QEMU";
pub const FWCFG_NAME_LEN: usize = 56;

pub const FW_CFG_DMA_CONTROL_ERROR: u32 = 0x1 << 0;
pub const FW_CFG_DMA_CONTROL_READ: u32 = 0x1 << 1;
pub const FW_CFG_DMA_CONTROL_SKIP: u32 = 0x1 << 2;
pub const FW_CFG_DMA_CONTROL_SELECT: u32 = 0x1 << 3;
pub const FW_CFG_DMA_CONTROL_WRITE: u32 = 0x1 << 4;

struct FWCfgFiles {
    /* the entire file directory fw_cfg item */
    count: u32,        /* number of entries, in big-endian format */
    f: *mut FWCfgFile, /* array of file entries, see below */
}

pub struct FWCfgFile {
    /* an individual file entry, 64 bytes total */
    pub size: u32,   /* size of referenced fw_cfg item, big-endian */
    pub select: u16, /* selector key of fw_cfg item, big-endian */
    pub reserved: u16,
    pub name: [u8; FWCFG_NAME_LEN], /* fw_cfg item name, NUL-terminated ascii */
}

#[repr(C, packed)]
pub struct FWCfgDmaAccess {
    pub control: u32,
    pub length: u32,
    pub address: u64,
}

pub fn fw_cfw_find_file(name: &str) -> Option<FWCfgFile> {
    let selector_addr = FW_CFG + FW_CFG_SELECTOR_OFFSET;
    let data_addr = FW_CFG + FW_CFG_DATA_OFFSET;

    let selector_reg = selector_addr as *mut u16;
    unsafe {
        *selector_reg = FW_CFG_FILE_DIR.to_be();
    }
    let count = unsafe { (*(data_addr as *mut u32)).to_be() };
    for _ in 0..count {
        // read FWCfgFile here
        let size: u32 = unsafe { (*(data_addr as *mut u32)).to_be() };
        let select: u16 = unsafe { (*(data_addr as *mut u16)).to_be() };
        let _reserved: u16 = unsafe { (*(data_addr as *mut u16)).to_be() };
        let mut name_bytes: [u8; FWCFG_NAME_LEN] = [0; FWCFG_NAME_LEN];
        for i in 0..FWCFG_NAME_LEN {
            name_bytes[i] = unsafe { *(data_addr as *mut u8) };
        }
        let null_pos = name_bytes
            .iter()
            .position(|&x| x == 0)
            .unwrap_or(name_bytes.len());
        let name_str = core::str::from_utf8(&name_bytes[..null_pos]).unwrap();

        if name_str == name {
            return Some(FWCfgFile {
                size,
                select,
                reserved: _reserved,
                name: name_bytes,
            });
        }
    }
    None
}

// this function was from osdev wiki
pub fn fw_cfg_dma_transfer(control: u32, length: u32, address: u64) -> Result<(), ()> {
    let dma_address_register_addr = FW_CFG + FW_CFG_DMA_OFFSET;
    let fw_cfg_dma_access_be = FWCfgDmaAccess {
        control: control.to_be(),
        length: length.to_be(),
        address: address.to_be(),
    };
    unsafe {
        (dma_address_register_addr as *mut u64)
            .write_volatile((&raw const fw_cfg_dma_access_be as u64).to_be());
        while (fw_cfg_dma_access_be.control & !FW_CFG_DMA_CONTROL_ERROR) != 0 {}
        if (fw_cfg_dma_access_be.control & FW_CFG_DMA_CONTROL_ERROR) != 0 {
            return Err(());
        }
    }
    Ok(())
}

pub fn test_fw_cfg() -> bool {
    let selector_addr = FW_CFG + FW_CFG_SELECTOR_OFFSET;
    let data_addr = FW_CFG + FW_CFG_DATA_OFFSET;

    let selector_reg = selector_addr as *mut u16;
    let data_reg = data_addr as *mut u8;
    unsafe {
        *selector_reg = FW_CFG_SIGNATURE.to_be();
    }
    // now we read data from data register
    for i in 0..4 {
        let d = unsafe { *data_reg };
        if d != FW_CFG_SIGNATURE_VAL[i] {
            return false;
        }
    }
    true
}

pub fn test_fw_cfg_dma() -> bool {
    let selector_addr = FW_CFG + FW_CFG_SELECTOR_OFFSET;
    let data_addr = FW_CFG + FW_CFG_DATA_OFFSET;
    unsafe {
        *(selector_addr as *mut u16) = FW_CFG_FW_CFG_ID.to_be();
    }
    let revision: u32 = unsafe { *(data_addr as *mut u32) };
    return (revision & 0b01) != 0;
}

pub fn test_iter_fwcfg() {
    let selector_addr = FW_CFG + FW_CFG_SELECTOR_OFFSET;
    let data_addr = FW_CFG + FW_CFG_DATA_OFFSET;

    let selector_reg = selector_addr as *mut u16;
    unsafe {
        *selector_reg = FW_CFG_FILE_DIR.to_be();
    }
    let count = unsafe { (*(data_addr as *mut u32)).to_be() };
    let files: Vec<FWCfgFile> = Vec::new();
    for i in 0..count {
        // read FWCfgFile here
        let size: u32 = unsafe { (*(data_addr as *mut u32)).to_be() };
        let select: u16 = unsafe { (*(data_addr as *mut u16)).to_be() };
        let reserved: u16 = unsafe { (*(data_addr as *mut u16)).to_be() };
        let mut name: [u8; FWCFG_NAME_LEN] = [0; FWCFG_NAME_LEN];
        for i in 0..FWCFG_NAME_LEN {
            name[i] = unsafe { *(data_addr as *mut u8) };
        }
        let null_pos = name.iter().position(|&x| x == 0).unwrap_or(name.len());
        let name_str = core::str::from_utf8(&name[..null_pos]).unwrap();
        println!("filename: {}", name_str);
    }
}
