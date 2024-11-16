use core::{cell::UnsafeCell, hint::black_box};

use msix::{set_all_msix_interrupt_handler, MSIXCapability};
use virtio::{virtio_pci_device_reset, VirtioPciCommonCfg, DEVICE_STATUS_ACKNOWLEDGE, DEVICE_STATUS_DRIVER, VIRTIO_PCI_CAP_COMMON_CFG};

use crate::{
    memolayout::{PCI_BASE, VGA_FRAME_BUFFER, VGA_FRAME_BUFFER_SIZE, VGA_MMIO_BASE},
    println,
    virtio::find_virtio_device,
};

mod msix;
mod virtio;

pub const VENDOR_SPECIFIC: u8 = 0x09;
pub const MIS_X: u8 = 0x11;
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

#[derive(Debug)]
#[repr(C)]
pub struct CapHeader {
    pub cap_id: u8,
    pub cap_next: u8,
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
    // assume all pci device is in bus 0
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
    let bar_base_addr: UnsafeCell<u32> = (type0_header.base_address_registers[0] as u32).into();
    let bar_0: *mut u32 = bar_base_addr.get();
    let bar_2: *mut u32 = bar_base_addr.get().wrapping_add(2);
    // let bar_6: *mut u32 = bar_base_addr.get().wrapping_add(5);//why 5? not 6?
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
    let header_t = unsafe { &mut *(config_addr as *mut PCIConfigurationSpcaeHeader) };
    let capability_struct_addr: usize = header.capabilities_pointer as usize + config_addr;

    println!("cap_id: {:?}", unsafe {
        *(capability_struct_addr as *const u8)
    });
    println!("next_cap: {:?}", unsafe {
        *((capability_struct_addr + 1) as *const u8)
    });
    println!("msi_mc: {:?}", unsafe {
        *((capability_struct_addr + 2) as *const u8)
    });
    println!("msi_mc: {:?}", unsafe {
        *((capability_struct_addr + 3) as *const u8)
    });
    unsafe {
        let mut msix_enable = *((capability_struct_addr + 2) as *const u16);
        msix_enable |= 1 << 15;
        *((capability_struct_addr + 2) as *mut u16) = msix_enable;
    };
    header_t.command = 6;
    println!("cap_id: {:?}", unsafe {
        *(capability_struct_addr as *const u8)
    });
    println!("next_cap: {:?}", unsafe {
        *((capability_struct_addr + 1) as *const u8)
    });
    println!("msi_mc: {:?}", unsafe {
        *((capability_struct_addr + 2) as *const u8)
    });
    println!("msi_mc: {:?}", unsafe {
        *((capability_struct_addr + 3) as *const u8)
    });

    // header.base_address_registers
}

pub fn test_bar() {
    let config_addr_sound = match find_virtio_device(25) {
        Some(c) => c,
        None => {
            panic!("can't find virtio sound device");
        }
    };

    println!("found virtio sound device: {:#x}", config_addr_sound);
    let header = unsafe { &mut *(config_addr_sound as *mut PCIConfigurationSpcaeHeaderType0) };
    let header_common = unsafe { &mut *(config_addr_sound as *mut PCIConfigurationSpcaeHeader) };
    println!("status: {:?}", header_common.status & 0b10000);
    let header_sound =
        unsafe { &mut *(config_addr_sound as *mut PCIConfigurationSpcaeHeaderType0) };
    traverse_cap_list(config_addr_sound, header.capabilities_pointer as usize);

    let cap_pointer_sound = header_sound.capabilities_pointer as usize;

    enable_device(config_addr_sound as usize);
    let mut next_cap_pointer = cap_pointer_sound;
    loop {
        let cap = unsafe { &*((config_addr_sound + next_cap_pointer) as *const CapHeader) };
        if cap.cap_id == VENDOR_SPECIFIC {
            let pci_cap =
                unsafe { &*((config_addr_sound + next_cap_pointer) as *const VirtioPciCap) };
            if pci_cap.cfg_type == VIRTIO_PCI_CAP_COMMON_CFG {
                let bar = pci_cap.bar as usize;
                println!(
                    "pci cap common cfg bar: {:x?}",
                    get_bar_value(config_addr_sound, bar)
                );
                set_bar_value(config_addr_sound, bar, 0x4002_0000);
                println!(
                    "pci cap common cfg bar: {:x?}",
                    get_bar_value(config_addr_sound, bar)
                );
                let common_cfg = unsafe { &mut *(0x4002_0000 as *mut VirtioPciCommonCfg) };
                common_cfg.device_status = 0; //reset device
                common_cfg.device_status = DEVICE_STATUS_ACKNOWLEDGE;
                common_cfg.device_status = DEVICE_STATUS_DRIVER;
                let features = common_cfg.device_feature;
                println!("features: {:x?}", features);
                // set feature bit here
            }
        }
        if cap.cap_next == 0 {
            break;
        }
        next_cap_pointer = cap.cap_next as usize;
    }

    enable_sound_msix(config_addr_sound, cap_pointer_sound);
    next_cap_pointer = cap_pointer_sound;
    loop {
        let cap = unsafe { &*((config_addr_sound + next_cap_pointer) as *const CapHeader) };
        if cap.cap_id == VENDOR_SPECIFIC {
            let pci_cap =
                unsafe { &*((config_addr_sound + next_cap_pointer) as *const VirtioPciCap) };
            if pci_cap.cfg_type == VIRTIO_PCI_CAP_COMMON_CFG {
                let common_cfg = unsafe { &mut *(0x4002_0000 as *mut VirtioPciCommonCfg) };
                println!("num queues: {:?}", common_cfg.num_queues);
                for i in 0..common_cfg.num_queues {
                    common_cfg.queue_select = i as u16;
                    common_cfg.config_msix_vector = 0;
                    common_cfg.queue_msix_vector = 1;
                    
                    println!("queue enabled: {:?}", common_cfg.queue_enable);
                    common_cfg.queue_enable = 1;
                    println!("queue enabled: {:?}", common_cfg.queue_enable);
                    // loop{}
                }
            }
        }
        if cap.cap_next == 0 {
            break;
        }
        next_cap_pointer = cap.cap_next as usize;
    }

    // enable_msix(config_addr_entropy, cap_pointer_entropy);

    // start_virtio_entropy_config(config_addr_entropy);
    // let bar_base_addr = black_box(&(header.base_address_registers[0]) as *const u8 as usize);
    // traverse_cap_list(config_addr_entropy, cap_pointer_entropy);
    // let bar0 = bar_base_addr as *mut u32;
    // let bar2 = bar0.wrapping_add(2);
}

struct StatusRegister(u16);

impl StatusRegister {
    fn immediate_readiness(&self) -> bool {
        self.0 & 1 == 1
    }
    fn interrupt_status(&self) -> bool {
        (self.0 >> 3) & 1 == 1
    }
    fn capabilities_list(&self) -> bool {
        (self.0 >> 4) & 1 == 1
    }
    fn master_data_parity_error(&self) -> bool {
        (self.0 >> 8) & 1 == 1
    }
    fn signaled_target_abort(&self) -> bool {
        (self.0 >> 11) & 1 == 1
    }
    fn received_target_abort(&self) -> bool {
        (self.0 >> 12) & 1 == 1
    }
    fn received_master_abort(&self) -> bool {
        (self.0 >> 13) & 1 == 1
    }
    fn signaled_system_error(&self) -> bool {
        (self.0 >> 14) & 1 == 1
    }
    fn detected_parity_error(&self) -> bool {
        (self.0 >> 15) & 1 == 1
    }
}

pub fn traverse_cap_list(config_addr: usize, cap_pointer: usize) {
    let mut next_cap_pointer = cap_pointer;
    loop {
        let cap = unsafe { &*((config_addr + next_cap_pointer) as *const VirtioPciCap) };
        println!("cap: {:?}", cap);
        if cap.cap_next == 0 {
            break;
        }
        next_cap_pointer = cap.cap_next as usize;
    }
}

pub fn traverse_express_cap_list(config_addr: usize) {
    let cap = unsafe { &*((config_addr + 0x100) as *const PCIECapability) };
    println!("next_cap_pointer: {:?}", cap);
}

pub fn enable_msix(config_addr: usize, cap_pointer: usize) {
    let mut next_cap_pointer = cap_pointer;
    loop {
        let cap = unsafe { &*((config_addr + next_cap_pointer) as *const CapHeader) };
        if cap.cap_id == MIS_X {
            enable_msix_inner(config_addr, next_cap_pointer);
        }
        if cap.cap_next == 0 {
            break;
        }
        next_cap_pointer = cap.cap_next as usize;
    }
}

fn enable_msix_inner(config_addr: usize, msix_cap_pointer: usize) {
    let msix_cap = unsafe { &mut *((config_addr + msix_cap_pointer) as *mut MSIXCapability) };
    let table_bar = msix_cap.get_table_bir();
    let pba_bar = msix_cap.get_pba_bir();
    println!("msix table bar: {:?}", table_bar);
    println!("msix pba bar: {:?}", pba_bar);

    let table_offset = msix_cap.get_table_offset();
    let pba_offset = msix_cap.get_pba_offset();
    println!("table offset: {:?}", table_offset);
    println!("pba offset: {:?}", pba_offset);

    println!(
        "table bar content{}: ",
        get_bar_value(config_addr, table_bar as usize)
    );
    println!(
        "pba bar content{}: ",
        get_bar_value(config_addr, pba_bar as usize)
    );

    let bar_content = 0x4001_0000;
    let table_addr = bar_content + table_offset;
    let pba_addr = bar_content + pba_offset;

    let table_size = msix_cap.table_size();
    println!("table size: {}", table_size);

    if table_bar == pba_bar {
        set_bar_value(config_addr, table_bar as usize, bar_content);
    } else {
        panic!("not implemented");
    }
    println!(
        "table bar content: {:#x}: ",
        get_bar_value(config_addr, table_bar as usize)
    );
    println!(
        "pba bar content: {:#x}: ",
        get_bar_value(config_addr, pba_bar as usize)
    );
    set_all_msix_interrupt_handler(table_addr as usize, table_size as usize);

    msix_cap.set_enable(true);
}
fn get_bar_value(config_addr: usize, bar: usize) -> u32 {
    let header = unsafe { &mut *(config_addr as *mut PCIConfigurationSpcaeHeaderType0) };
    let bar_base_addr = black_box(&(header.base_address_registers[0]) as *const u8 as usize);
    let bar0 = bar_base_addr as *mut u32;
    unsafe { *bar0.wrapping_add(bar) }
}

fn set_bar_value(config_addr: usize, bar: usize, value: u32) {
    let header = unsafe { &mut *(config_addr as *mut PCIConfigurationSpcaeHeaderType0) };
    let bar_base_addr = black_box(&(header.base_address_registers[0]) as *const u8 as usize);
    let bar0 = bar_base_addr as *mut u32;
    unsafe { *bar0.wrapping_add(bar) = value };
}

pub fn start_virtio_entropy_config(config_addr: usize) {
    let header = unsafe { &mut *(config_addr as *mut PCIConfigurationSpcaeHeaderType0) };
    let header_t = unsafe { &mut *(config_addr as *mut PCIConfigurationSpcaeHeader) };
    let mut next_cap_pointer = header.capabilities_pointer as usize;
    let bar: Option<usize>;
    let offset: Option<usize>;
    loop {
        let cap = unsafe { &*((config_addr + next_cap_pointer) as *const VirtioPciCap) };

        if cap.cap_vndr == VENDOR_SPECIFIC && cap.cfg_type == 0x01 {
            bar = Some(cap.bar as usize);
            offset = Some(cap.offset as usize);
            break;
        }

        if cap.cap_next == 0 {
            println!("did not find common cfg struct");
            return;
        }
        next_cap_pointer = cap.cap_next as usize;
    }

    println!("entropy card cap pointer:");
    let bar_content = 0x4100_0000;
    println!("status: {}", header_t.status);
    println!("bar: {}", bar.unwrap());
    println!("offset: {}", offset.unwrap());
    set_bar_value(config_addr, bar.unwrap(), bar_content);
    println!(
        "bar content: {:#x}: ",
        get_bar_value(config_addr, bar.unwrap())
    );
    println!("status: {}", header_t.status);

    let common_cfg_addr = bar_content as usize + offset.unwrap() as usize;
    println!("common cfg addr: {:#x}", common_cfg_addr);
    let common_cfg = unsafe { &mut *(common_cfg_addr as *mut VirtioPciCommonCfg) };
    println!("common cfg: {:?}", common_cfg);
    virtio_pci_device_reset(common_cfg);
    common_cfg.device_status |= virtio::DEVICE_STATUS_ACKNOWLEDGE;
    println!("common cfg: {:?}", common_cfg);
    common_cfg.device_status |= virtio::DEVICE_STATUS_DRIVER;
    println!("common cfg: {:?}", common_cfg);
    common_cfg.device_status |= virtio::DEVICE_STATUS_FEATURES_OK;
    println!("common cfg: {:?}", common_cfg);
}

pub fn get_bar_region_size(config_addr: usize, bar: usize) -> usize {
    0
}

pub fn enable_device(config_addr: usize) {
    let header_t = unsafe { &mut *(config_addr as *mut PCIConfigurationSpcaeHeader) };
    header_t.command = header_t.command | 0b100; //enable mastering enable bit
    header_t.command = header_t.command | 0b10; //enable mmory space enable bit
    println!("command register: {:x}", header_t.command);
}

#[repr(C)]
#[derive(Debug)]
struct PCIECapability {
    pcie_cap_id: u8,
    next_cap_pointer: u8,
    pcie_capability_register: u16,
    device_capabilities: u32,
    device_control: u16,
    device_status: u16,
    link_capabilities: u32,
    link_control: u16,
    link_status: u16,
    slot_capabilities: u32,
    slot_control: u16,
    slot_status: u16,
    root_control: u16,
    root_capabilities: u16,
    root_status: u32,
    device_capabilities2: u32,
    device_control2: u16,
    device_status2: u16,
    link_capabilities2: u32,
    link_control2: u16,
    link_status2: u16,
    slot_capabilities2: u32,
    slot_control2: u16,
    slot_status2: u16,
}

pub fn enable_sound_msix(config_addr: usize, cap_pointer: usize) {
    let mut next_cap_pointer = cap_pointer;
    loop {
        let cap = unsafe { &*((config_addr + next_cap_pointer) as *const CapHeader) };
        if cap.cap_id == MIS_X {
            println!("found msix cap: {:?}", next_cap_pointer);
            enable_msix_inner(config_addr, next_cap_pointer);
        }
        if cap.cap_next == 0 {
            break;
        }
        next_cap_pointer = cap.cap_next as usize;
    }
}
