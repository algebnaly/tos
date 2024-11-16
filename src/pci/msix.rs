use crate::println;

#[repr(C)]
#[derive(Debug)]
pub struct MSIXCapability {
    pub msix_cap_id: u8,
    pub next_cap_pointer: u8,
    pub message_control: u16,
    pub table_offset_and_bir: u32, //we have to use u32 here because PCI spec says we should use DWORD access
    pub pba_offset_and_bir: u32,   //same as above
}

impl MSIXCapability {
    pub fn get_table_offset(&self) -> u32 {
        self.table_offset_and_bir & !(0b11)
    }
    pub fn get_pba_offset(&self) -> u32 {
        self.pba_offset_and_bir & !(0b11)
    }
    pub fn table_size(&self) -> u16 {
        let n_minus_1 = self.message_control & 0b111_1111_1111;
        n_minus_1 + 1 
    }
    pub fn set_enable(&mut self, enable: bool) {
        if enable {
            self.message_control |= 1 << 15;
        } else {
            self.message_control &= !(1 << 15);
        }
    }
    pub fn get_table_bir(&self) -> u32 {
        self.table_offset_and_bir & 0b11
    }

    pub fn get_pba_bir(&self) -> u32 {
        self.pba_offset_and_bir & 0b11
    }
    pub fn get_table_entry(&self, index: u32) -> u32 {
        panic!("not implemented");
    }
}

#[repr(C)]
pub struct MSIXTableEntry {
    pub message_address: u32,
    pub message_address_high: u32,
    pub message_data: u32,
    pub vector_control: u32,
}

pub struct MSIXPBATableEntry(u64);

pub fn dummy_msix_interrupt_handler() {
    println!("dummy interrupt handler");
}

pub fn set_dummy_msix_interrupt_handler(table_addr : usize, entry_index: usize) {
    let table_addr = table_addr as *mut MSIXTableEntry;
    let entry_addr = table_addr.wrapping_add(entry_index);
    unsafe {
        let table_entry = &mut *entry_addr;
        table_entry.message_address = dummy_msix_interrupt_handler as u32;
        table_entry.message_address_high = 0;
        table_entry.message_data = 0;
        table_entry.vector_control = 0;
    }
}

pub fn set_all_msix_interrupt_handler(table_addr : usize, table_size: usize) {
    for i in 0..table_size {
        set_dummy_msix_interrupt_handler(table_addr, i);
    }
}