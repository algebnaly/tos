use crate::{
    mem_utils,
    memolayout::{PCI_BASE, VGA_FRAME_BUFFER, VGA_FRAME_BUFFER_SIZE, VGA_MMIO_BASE},
    println,
    vm::kalloc,
};

pub const VENDOR_SPECIFIC: u8 = 0x09;

#[derive(Debug)]
#[repr(C)]
struct PCIConfigurationSpcaeHeader {
    pub vendor_id: u16,
    pub device_id: u16,
    pub command: u16,
    pub status: u16,
    pub revision_id: u8,
    pub class_code: [u8; 3],
    pub cache_line_size: u8,
    pub master_latency_time: u8,
    pub header_type: u8,
    pub built_in_self_test: u8,
    pub remain_part: [u8; 240],
}

#[repr(C)]
pub struct PCIConfigurationSpcaeHeaderType0 {
    pub __padding: [u8; 16],
    pub base_address_registers: [u8; 24],
    pub cardbus_cis_pointer: u32,
    pub subsystem_vendor_id: u16,
    pub subsystem_id: u16,
    pub expansion_rom_base_address: u32,
    pub capabilities_pointer: u8,
    pub reserved: [u8; 7],
    pub interrupt_line: u8,
    pub interrupt_pin: u8,
    pub min_gnt: u8,
    pub max_lat: u8,
}

#[derive(Debug)]
#[repr(C)]
pub struct VirtioPciCap {
    pub cap_vndr: u8,
    pub cap_next: u8,
    pub cap_len: u8,
    pub cfg_type: u8,
    pub bar: u8,
    pub id: u8,
    pub padding: [u8; 2],
    pub offset: u32,
    pub length: u32,
}

pub fn list_pci(pci_base_addr: usize) {
    for dev_num in 0..(1 << 5) {
        for func_num in 0..(1 << 3) {
            //we list device in bus 0 only
            let config_space_addr = pci_base_addr + (dev_num << (12 + 3)) + (func_num << 12);
            let config_space_header =
                unsafe { &*(config_space_addr as *const PCIConfigurationSpcaeHeader) };
            if config_space_header.vendor_id == 0xffff {
                //ignore device with vendor id 0xffff
                continue;
            }
            println!(
                "bus:dev:func {bus_num}:{dev_num}:{func_num} vendor_id:device_id {:04x}:{:04x}",
                config_space_header.vendor_id,
                config_space_header.device_id,
                bus_num = 0,
            );

            println!(
                "header_type {}, status: {}",
                config_space_header.header_type, config_space_header.status
            );
            if config_space_header.header_type == 0 {
                let type0_header =
                    unsafe { &*(config_space_addr as *const PCIConfigurationSpcaeHeaderType0) };
                println!(
                    "capabilities pointer {:x}",
                    type0_header.capabilities_pointer
                );
                disp_cap_list(
                    config_space_addr,
                    type0_header.capabilities_pointer as usize,
                );
            }
            println!("----------------");
        }
    }
}

pub fn disp_cap_list(config_addr: usize, cap_pointer: usize) {
    let mut cap_ptr = cap_pointer;
    while cap_ptr != 0 {
        // next cap equals 0 means end of the chain
        let addr = config_addr + cap_ptr;
        let cap_vndr = unsafe { *(addr as *const u8) };
        cap_ptr = unsafe { *((addr + 1) as *const u8) } as usize; //next cap
        if cap_vndr != VENDOR_SPECIFIC {
            continue;
        }
        let pci_cap = unsafe { &*(addr as *const VirtioPciCap) };
        println!("{:?}", pci_cap);
    }
}

pub fn find_device(pci_base_addr: usize, vendor_id: u16, device_id: u16) -> Option<usize> {
    for dev_num in 0..(1 << 5) {
        for func_num in 0..(1 << 3) {
            let config_space_addr = pci_base_addr + (dev_num << (12 + 3)) + (func_num << 12);
            let config_space_header =
                unsafe { &*(config_space_addr as *const PCIConfigurationSpcaeHeader) };
            if config_space_header.vendor_id == vendor_id
                && config_space_header.device_id == device_id
            {
                return Some(config_space_addr);
            }
        }
    }
    None
}

pub unsafe fn write_vga(addr: usize) {
    let csh: &mut PCIConfigurationSpcaeHeader = &mut *(addr as *mut PCIConfigurationSpcaeHeader);
    csh.command = csh.command | 0b10;
    println!("Command: {}", csh.command);
    if csh.header_type != 0 {
        return;
    }
    let type0_header: &PCIConfigurationSpcaeHeaderType0 =
        &*((&csh.remain_part as *const u8) as u64 as *const PCIConfigurationSpcaeHeaderType0);
    let bar_0: &mut u32 =
        &mut *(&type0_header.base_address_registers as *const u8 as u64 as *mut u32);
    let bar_2: &mut u32 =
        &mut *((&type0_header.base_address_registers as *const u8 as u64 + 4 * 2) as *mut u32);
    let bar_6: &mut u32 =
        &mut *((&type0_header.base_address_registers as *const u8 as u64 + 4 * 5) as *mut u32);
    let bar_0_val = VGA_FRAME_BUFFER;
    *bar_0 = bar_0_val as u32;
    *bar_2 = VGA_MMIO_BASE as u32;

    println!("{:x}", *bar_0);
    println!("{:x}", bar_0_val);
    let framebuffer: &mut [u8; VGA_FRAME_BUFFER_SIZE] =
        &mut *(bar_0_val as *mut [u8; VGA_FRAME_BUFFER_SIZE]);
    let vga_mmio: &mut [u8; 4096] = &mut *(VGA_MMIO_BASE as *mut [u8; 4096]);
    framebuffer.fill(0xff);
    vga_mmio[0] = 0x0c;
}

pub fn test_write_bar() {
    let config_addr = find_device(PCI_BASE, 0x1af4, 0x1050).expect("can't find pci device");
    let header = unsafe { &*(config_addr as *mut PCIConfigurationSpcaeHeaderType0) };
    // header.base_address_registers
    let my_bar: u32 = 0x9000_0000;

    let addr = &header.base_address_registers[4] as *const u8 as u32 as *mut u32;
    unsafe {
        *addr = my_bar;
    }
}
