use crate::CellType;

#[derive(Copy, Clone)]
pub struct CellDefMeta {
    pub name: &'static str,
    pub cell_type: CellType,

    pub ram_range_start_offs: usize,
    pub ram_range_end_offs: usize,

    pub flash_range_start_offs: usize,
    pub flash_range_end_offs: usize,

    pub struct_sha256: [u8; 32],
}

#[derive(Copy, Clone)]
pub struct DeviceConfigMeta {
    pub initial_stack_ptr: usize,
    pub ram_range_start: usize,
    pub ram_range_end: usize,
    pub flash_range_start: usize,
    pub flash_range_end: usize,
}


#[derive(Copy, Clone)]
pub struct CellDefsMeta<const N: usize> {
    pub device_configuration: DeviceConfigMeta,
    pub cell_defs: [CellDefMeta; N]
}

impl<const N: usize> CellDefsMeta<N> {
    pub fn for_cell(&'static self, cell_name: &str) -> Option<&'static CellDefMeta> {
        self.cell_defs.iter().find(|cell| cell.name == cell_name)
    }
}

impl CellDefMeta {

    pub const fn absolute_ram_start(&self, device_config_meta: &DeviceConfigMeta) -> usize {
        device_config_meta.ram_range_start + self.ram_range_start_offs
    }
    pub const fn absolute_ram_end(&self, device_config_meta: &DeviceConfigMeta) -> usize {
        device_config_meta.ram_range_start + self.ram_range_end_offs
    }
    pub const fn absolute_flash_start(&self, device_config_meta: &DeviceConfigMeta) -> usize {
        device_config_meta.flash_range_start + self.flash_range_start_offs
    }
    pub const fn absolute_flash_end(&self, device_config_meta: &DeviceConfigMeta) -> usize {
        device_config_meta.flash_range_start + self.flash_range_end_offs
    }
}