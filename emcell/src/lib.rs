#![no_std]

use core::ops::Deref;

pub enum HeaderState {
    Uninit,
    Init,
    Dummy
}

pub unsafe trait Cell {
    fn check_signature(&self) -> bool;
}

pub struct CellWrapper<T>
where T: 'static {
    header: &'static T,
    state: HeaderState
}

impl<T> Deref for CellWrapper<T>
    where T: Cell + 'static {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.header
    }
}


impl<T> CellWrapper<T>
where T: Cell + 'static {
    pub unsafe fn _new_uninit(h: &'static T) -> Self {
        Self {
            header: unsafe { h },
            state: HeaderState::Uninit
        }
    }

    pub unsafe fn _new_init(h: &'static T) -> Option<Self> {
        if !h.check_signature() {
            return None;
        }

        Some(Self {
            header: unsafe { h },
            state: HeaderState::Init
        })
    }

    pub fn new_dummy(dummy_abi: &'static T) -> Self {
        Self {
            header: dummy_abi,
            state: HeaderState::Dummy
        }
    }
}

pub enum CellType {
    Primary,
    NonPrimary
}
pub struct CellDefMeta {
    pub name: &'static str,
    pub cell_type: CellType,

    pub ram_range_start_offs: usize,
    pub ram_range_end_offs: usize,

    pub flash_range_start_offs: usize,
    pub flash_range_end_offs: usize,
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

impl<const N: usize> CellDefsMeta<N> {
    pub fn for_cell(&'static self, cell_name: &str) -> Option<&'static CellDefMeta> {
        self.cell_defs.iter().find(|cell| cell.name == cell_name)
    }
    pub fn absolute_ram_start(&'static self, cell_name: &str) -> usize {
        let cell = self.for_cell(cell_name).unwrap();
        self.device_configuration.ram_range_start + cell.ram_range_start_offs
    }
    pub fn absolute_ram_end(&'static self, cell_name: &str) -> usize {
        let cell = self.for_cell(cell_name).unwrap();
        self.device_configuration.ram_range_start + cell.ram_range_end_offs
    }
    pub fn absolute_flash_start(&'static self, cell_name: &str) -> usize {
        let cell = self.for_cell(cell_name).unwrap();
        self.device_configuration.flash_range_start + cell.flash_range_start_offs
    }
    pub fn absolute_flash_end(&'static self, cell_name: &str) -> usize {
        let cell = self.for_cell(cell_name).unwrap();
        self.device_configuration.flash_range_start + cell.flash_range_end_offs
    }
}

#[cfg(not(feature = "build-rs"))]
pub unsafe fn init_memory() {
    use core::ptr;

    extern "C" {
        static mut __sbss: u32;
        static mut __ebss: u32;

        static mut __sdata: u32;
        static mut __edata: u32;
        static mut __sidata: u32;
    }
    let count = &__ebss as *const u32 as usize - &__sbss as *const u32 as usize;
    let addr = &mut __sbss as *mut u32 as *mut u8;
    if count > 0 {
        ptr::write_bytes(addr, 0, count);
    }

    let count = &__edata as *const u32 as usize - &__sdata as *const u32 as usize;
    if count > 0 {
        ptr::copy_nonoverlapping(
            &__sidata as *const u32 as *const u8,
            &mut __sdata as *mut u32 as *mut u8,
            count);
    }
}

#[cfg(all(feature = "build-rs", not(feature = "rt-crate-cortex-m-rt")))]
compile_error!("This crate requires any rt-crate-* to be enabled (when using build-rs feature)! *currently only rt-crate-cortex-m-rt is supported*");


#[cfg(feature = "build-rs")]
extern crate std;

const HEADER_SIZE: usize = 1 * 1024;

pub struct PartitionedFlashRegion {
    pub start_flash: usize,
    pub end_flash: usize,
    pub start_header: usize,
    pub end_header: usize,
}

impl PartitionedFlashRegion {
    pub fn from<const N: usize>(meta: &'static CellDefsMeta<N>, cell_name: &str) -> Self {
        let cell = meta.for_cell(cell_name).unwrap();
        let start_flash = meta.absolute_flash_start(cell_name);
        let end_flash = meta.absolute_flash_end(cell_name) - HEADER_SIZE;

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

#[cfg(all(feature = "build-rs", feature = "rt-crate-cortex-m-rt"))]
pub fn build_rs<const N: usize>(meta: &'static CellDefsMeta<N>, cur_cell: &'static CellDefMeta) {
    use std::env;
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;
    use std::string::String;

    let out_dir = &PathBuf::from(env::var("OUT_DIR").unwrap());

    let cur_cell_name = cur_cell.name;
    let other_cells_names: std::vec::Vec<&str> = meta.cell_defs.iter().map(|cell| cell.name).filter(|name| *name != cur_cell_name).collect();

    let cur_partitioned_flash_region = PartitionedFlashRegion::from(meta, cur_cell_name);
    let mut memory_definition = String::from("# THIS SCRIPT WAS GENERATED AUTOMATICALLY BY emcell LIBRARY!\nMEMORY {\n")
        // this cell flash definition
    + &std::format!("  FLASH : ORIGIN = 0x{:X}, LENGTH = {}\n",
            cur_partitioned_flash_region.start_flash,
            cur_partitioned_flash_region.end_flash - cur_partitioned_flash_region.start_flash)
    + &std::format!("  CUR_HEADER : ORIGIN = 0x{:X}, LENGTH = {}\n",
            cur_partitioned_flash_region.start_header,
            cur_partitioned_flash_region.end_header - cur_partitioned_flash_region.start_header)
    + &std::format!("  RAM : ORIGIN = 0x{:X}, LENGTH = {}\n\n",
            meta.absolute_ram_start(cur_cell_name),
            meta.absolute_ram_end(cur_cell_name) - meta.absolute_ram_start(cur_cell_name));

    for cell_name in &other_cells_names {
        let partitioned_flash_region = PartitionedFlashRegion::from(meta, cell_name);
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
            meta.absolute_ram_start(cell_name),
            meta.absolute_ram_end(cell_name) - meta.absolute_ram_start(cell_name)));
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