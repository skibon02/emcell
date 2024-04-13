extern crate std;

use crate::meta;
use crate::meta::{CellDefMeta, CellDefsMeta, DeviceConfigMeta};

const HEADER_SIZE: usize = 1 * 1024;

pub struct PartitionedFlashRegion {
    pub start_flash: usize,
    pub end_flash: usize,
    pub start_header: usize,
    pub end_header: usize,
}

impl PartitionedFlashRegion {
    pub fn from(cell: &'static CellDefMeta, device_config_meta: &DeviceConfigMeta) -> Self {
        let start_flash = cell.absolute_flash_start(device_config_meta);
        let end_flash = cell.absolute_flash_end(device_config_meta) - HEADER_SIZE;

        let start_header = end_flash;
        let end_header = end_flash + HEADER_SIZE;

        Self {
            start_flash,
            end_flash,
            start_header,
            end_header,
        }
    }
}
#[cfg(feature = "rt-crate-cortex-m-rt")]
pub fn build_rs<const N: usize>(meta: &'static CellDefsMeta<N>, cur_cell: &'static CellDefMeta) {
    use std::env;
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;
    use std::string::String;

    let out_dir = &PathBuf::from(env::var("OUT_DIR").unwrap());

    let cur_cell_name = cur_cell.name;
    let other_cells_names: std::vec::Vec<&str> = meta.cell_defs.iter().map(|cell| cell.name).filter(|name| *name != cur_cell_name).collect();

    let cur_partitioned_flash_region = PartitionedFlashRegion::from(cur_cell, &meta.device_configuration);
    let mut memory_definition = String::from("# THIS SCRIPT WAS GENERATED AUTOMATICALLY BY emcell LIBRARY!\nMEMORY {\n")
        // this cell flash definition
        + &std::format!("  FLASH : ORIGIN = 0x{:X}, LENGTH = {}\n",
                        cur_partitioned_flash_region.start_flash,
                        cur_partitioned_flash_region.end_flash - cur_partitioned_flash_region.start_flash)
        + &std::format!("  CUR_HEADER : ORIGIN = 0x{:X}, LENGTH = {}\n",
                        cur_partitioned_flash_region.start_header,
                        cur_partitioned_flash_region.end_header - cur_partitioned_flash_region.start_header)
        + &std::format!("  RAM : ORIGIN = 0x{:X}, LENGTH = {}\n\n",
                        cur_cell.absolute_ram_start(&meta.device_configuration),
                        cur_cell.absolute_ram_end(&meta.device_configuration) - cur_cell.absolute_ram_start(&meta.device_configuration));

    for cell_name in &other_cells_names {
        let cell_meta = meta.for_cell(cell_name).unwrap();
        let partitioned_flash_region = PartitionedFlashRegion::from(cell_meta, &meta.device_configuration);
        memory_definition += &(String::from(std::format!("  {}_FLASH : ORIGIN = 0x{:X}, LENGTH = {}\n",
                                                         cell_name,
                                                         partitioned_flash_region.start_flash,
                                                         partitioned_flash_region.end_flash - partitioned_flash_region.start_flash))
            + &std::format!("  {}_HEADER : ORIGIN = 0x{:X}, LENGTH = {}\n",
                            cell_name,
                            partitioned_flash_region.start_header,
                            partitioned_flash_region.end_header - partitioned_flash_region.start_header)
            + &std::format!("  {}_RAM : ORIGIN = 0x{:X}, LENGTH = {}\n\n",
                            cell_name,
                            cell_meta.absolute_ram_start(&meta.device_configuration),
                            cell_meta.absolute_ram_end(&meta.device_configuration) - cell_meta.absolute_ram_start(&meta.device_configuration)));
    }

    memory_definition += "}\n\n";

    memory_definition += std::format!("_stack_start = 0x{:X};\n\n", meta.device_configuration.initial_stack_ptr).as_str();

    memory_definition += "SECTIONS {\n";
    // this cell header
    memory_definition += &(String::from("    .CUR_HEADER ORIGIN(CUR_HEADER) : {\n")
        + "        . = ALIGN(4);\n"
        + "        KEEP(*(.emcell.cur_header))\n"
        + "        . = ALIGN(4);\n"
        + "    } > CUR_HEADER\n");

    // other cells headers
    for cell_name in &other_cells_names {
        memory_definition += &(String::from("    .") + cell_name + "_HEADER ORIGIN(" + cell_name + "_HEADER) : {\n"
            + &std::format!("        _emcell_{}_internal = .;\n", cell_name)
            + "    } > " + cell_name + "_HEADER\n");
    }

    memory_definition += "}\n";



    let mut f = File::create(out_dir.join("memory.x")).unwrap();
    f.write_all(memory_definition.as_bytes()).unwrap();

    std::println!("cargo:rustc-link-search={}", out_dir.display());
}