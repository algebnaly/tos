pub const DEVICE_STATUS_ACKNOWLEDGE: u8 = 0x01;
pub const DEVICE_STATUS_DRIVER: u8 = 0x02;
pub const DEVICE_STATUS_FAILED: u8 = 0x80;// 128
pub const DEVICE_STATUS_FEATURES_OK: u8 = 0x08;
pub const DEVICE_STATUS_DRIVER_OK: u8 = 0x04;
pub const DEVICE_STATUS_NEEDS_RESET: u8 = 0x40; // 64


#[derive(Debug)]
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

