#![feature(prelude_import)]
#![no_std]
#[prelude_import]
use core::prelude::rust_2021::*;
#[macro_use]
extern crate core;
extern crate compiler_builtins as _;
#[macro_use]
extern crate emcell_macro;
pub struct Cell1 {
    pub a: u32,
    pub print_some_value: fn(u32),
}
pub struct Cell2 {
    pub b: u32,
    pub run_some_code: fn(),
    pub init_memory: unsafe fn(),
    pub signature: u32,
}
pub const CELL_NAMES: [&'static str; 2usize] = ["Cell1", "Cell2"];
pub type PrimaryCell = Cell1;
unsafe impl emcell::Cell for Cell2 {
    fn check_signature(&self) -> bool {
        let sig_valid = self.signature == 0xdeadbeef;
        if sig_valid {
            unsafe { (self.init_memory)() }
        }
        sig_valid
    }
}
unsafe impl emcell::Cell for Cell1 {
    fn check_signature(&self) -> bool {
        true
    }
}
pub static META: emcell::CellDefsMeta<2usize> = emcell::CellDefsMeta {
    cell_defs: [
        emcell::CellDefMeta {
            name: "Cell1",
            cell_type: emcell::CellType::Primary,
            ram_range_start_offs: 24576usize,
            ram_range_end_offs: 57344usize,
            flash_range_start_offs: 0usize,
            flash_range_end_offs: 524288usize,
        },
        emcell::CellDefMeta {
            name: "Cell2",
            cell_type: emcell::CellType::NonPrimary,
            ram_range_start_offs: 57344usize,
            ram_range_end_offs: 65536usize,
            flash_range_start_offs: 524288usize,
            flash_range_end_offs: 1048576usize,
        },
    ],
    device_configuration: emcell::DeviceConfigMeta {
        initial_stack_ptr: 536895488usize,
        ram_range_start: 536870912usize,
        ram_range_end: 536936448usize,
        flash_range_start: 134217728usize,
        flash_range_end: 135266304usize,
    },
};
