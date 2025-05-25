/*
this file was inspired by osdev wiki and https://github.com/CityAceE/qemu-ramfb-riscv64-driver
*/

use crate::{
    fw_cfg::{
        fw_cfg_dma_transfer, fw_cfw_find_file, test_fw_cfg, test_fw_cfg_dma,
        FW_CFG_DMA_CONTROL_SELECT, FW_CFG_DMA_CONTROL_WRITE,
    },
    println,
};

macro_rules! fourcc_code {
    ($a: expr,$b: expr,$c: expr, $d: expr) => {
        ($a as u32) | ($b as u32) << 8 | ($c as u32) << 16 | ($d as u32) << 24
    };
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
struct RAMFBCfg {
    addr: u64,
    fourcc: u32,
    flags: u32,
    width: u32,
    height: u32,
    stride: u32,
}

pub const RAMFB_FW_CFW_NAME: &str = "etc/ramfb";

pub const RAM_FB_WIDTH: usize = 1280;
pub const RAM_FB_HEIGHT: usize = 720;
pub const RAM_FB_BPP: usize = 4;
pub const RAM_FB_STRIDE: usize = RAM_FB_BPP * RAM_FB_WIDTH;
pub const FOURCC_ARGB8888: u32 = fourcc_code!('A', 'R', '2', '4'); /* [31:0] A:R:G:B 8:8:8:8 little endian */

pub static mut RAMFB: [u8; RAM_FB_HEIGHT * RAM_FB_STRIDE] = [0; RAM_FB_HEIGHT * RAM_FB_STRIDE];
pub static mut RAMFB_OK: bool = false;
pub fn setup_ramfb() {
    let ramfbcfg_be = RAMFBCfg {
        addr: (&raw const RAMFB as u64).to_be(),
        fourcc: FOURCC_ARGB8888.to_be(),
        flags: (0 as u32).to_be(),
        width: (RAM_FB_WIDTH as u32).to_be(),
        height: (RAM_FB_HEIGHT as u32).to_be(),
        stride: (RAM_FB_STRIDE as u32).to_be(),
    };

    let ramfb_cfg_file = match fw_cfw_find_file(RAMFB_FW_CFW_NAME) {
        Some(f) => f,
        None => {
            return;
        }
    };
    let select = ramfb_cfg_file.select;
    let control = FW_CFG_DMA_CONTROL_WRITE | FW_CFG_DMA_CONTROL_SELECT | ((select as u32) << 16);
    let length = size_of::<RAMFBCfg>() as u32;
    if fw_cfg_dma_transfer(
        control,
        length,
        (&raw const ramfbcfg_be) as u64,
    )
    .is_err()
    {
        return;
    }
    unsafe {
        RAMFB_OK = true;
    }
}

pub fn ramfb_clear(color: u32){
    for x in 0..RAM_FB_WIDTH*RAM_FB_HEIGHT{
        let color_bytes = color.to_le_bytes();
        unsafe{
            RAMFB[x * RAM_FB_BPP + 0] = color_bytes[0];
            RAMFB[x * RAM_FB_BPP + 1] = color_bytes[1];
            RAMFB[x * RAM_FB_BPP + 2] = color_bytes[2];
            RAMFB[x * RAM_FB_BPP + 3] = color_bytes[3];
        }
    }
}