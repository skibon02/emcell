use crate::CellType;

pub struct CellDefMeta {
    pub name: &'static str,
    pub cell_type: CellType,

    pub ram_range_start_offs: usize,
    pub ram_range_end_offs: usize,

    pub flash_range_start_offs: usize,
    pub flash_range_end_offs: usize,

    pub struct_sha256: [u8; 32],
}

pub struct DeviceConfigMeta {
    pub initial_stack_ptr: usize,
    pub ram_range_start: usize,
    pub ram_range_end: usize,
    pub flash_range_start: usize,
    pub flash_range_end: usize,
}

pub struct CellDefsMeta<const N: usize> {
    pub device_configuration: DeviceConfigMeta,
    pub cell_defs: [CellDefMeta; N]
}

impl CellDefMeta {

    pub fn absolute_ram_start(&'static self, device_config_meta: &DeviceConfigMeta) -> usize {
        device_config_meta.ram_range_start + self.ram_range_start_offs
    }
    pub fn absolute_ram_end(&'static self, device_config_meta: &DeviceConfigMeta) -> usize {
        device_config_meta.ram_range_start + self.ram_range_end_offs
    }
    pub fn absolute_flash_start(&'static self, device_config_meta: &DeviceConfigMeta) -> usize {
        device_config_meta.flash_range_start + self.flash_range_start_offs
    }
    pub fn absolute_flash_end(&'static self, device_config_meta: &DeviceConfigMeta) -> usize {
        device_config_meta.flash_range_start + self.flash_range_end_offs
    }
}

impl<const N: usize> CellDefsMeta<N> {
    pub fn for_cell(&'static self, cell_name: &str) -> Option<&'static CellDefMeta> {
        self.cell_defs.iter().find(|cell| cell.name == cell_name)
    }
}