pub const DEVICE_STATUS_ACKNOWLEDGE: u8 = 0x01;
pub const DEVICE_STATUS_DRIVER: u8 = 0x02;
pub const DEVICE_STATUS_FAILED: u8 = 0x80; // 128
pub const DEVICE_STATUS_FEATURES_OK: u8 = 0x08;
pub const DEVICE_STATUS_DRIVER_OK: u8 = 0x04;
pub const DEVICE_STATUS_NEEDS_RESET: u8 = 0x40; // 64

pub const VIRTIO_PCI_CAP_COMMON_CFG: u8 = 0x01;
pub const VIRTIO_PCI_CAP_NOTIFY_CFG: u8 = 0x02;
pub const VIRTIO_PCI_CAP_ISR_CFG: u8 = 0x03;
pub const VIRTIO_PCI_CAP_DEVICE_CFG: u8 = 0x04;
pub const VIRTIO_PCI_CAP_PCI_CFG: u8 = 0x05;
pub const VIRTIO_PCI_CAP_SHARED_CFG: u8 = 0x08;
pub const VIRTIO_PCI_CAP_VENDOR_CFG: u8 = 0x09;

pub const VIRTIO_PCI: u8 = 0x01;

#[derive(Debug)]
#[repr(C)]
pub struct VirtioPciCommonCfg {
    pub device_feature_select: u32,
    pub device_feature: u32,
    pub driver_feature_select: u32,
    pub driver_feature: u32,
    pub config_msix_vector: u16,
    pub num_queues: u16,
    pub device_status: u8,
    pub config_generation: u8,
    pub queue_select: u16,
    pub queue_size: u16,
    pub queue_msix_vector: u16,
    pub queue_enable: u16,
    pub queue_notify_off: u16,
    pub queue_desc: u64,
    pub queue_driver: u64,
    pub queue_device: u64,
    pub queue_notify_data: u16,
    pub queue_reset: u16,
}

pub fn virtio_pci_device_reset(device: &mut VirtioPciCommonCfg) {
    device.device_status = 0;
}

pub const VIRTQ_DESC_F_NEXT: u16 = 1; // this marks a buffer as continuing via the next field
pub const VIRTQ_DESC_F_WRITE: u16 = 2; // this marks a buffer as device write only
pub const VIRTQ_DESC_F_INDIRECT: u16 = 4; // this means the buffer contains a list of buffer descriptors
#[derive(Debug)]
#[repr(C)]
pub struct VirtioDesc {
    pub addr: u64,
    pub len: u32,
    pub flags: u16, // values defined above
    pub next: u16,
}
