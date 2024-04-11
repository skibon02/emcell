#![feature(prelude_import)]
#![no_std]
#[prelude_import]
use core::prelude::rust_2021::*;
#[macro_use]
extern crate core;
extern crate compiler_builtins as _;
#[macro_use]
extern crate emcell_macro;
pub struct Cell1ABI {
    pub a: u32,
    pub print_some_value: fn(u32),
}
pub struct Cell2ABI {
    pub b: u32,
    pub run_some_code: fn(),
    pub init_memory: unsafe fn(),
    pub signature: u32,
}
pub const CELL_NAMES: [&'static str; 2usize] = ["Cell1ABI", "Cell2ABI"];
pub type PrimaryCell = Cell1ABI;
pub static META: emcell::CellDefsMeta<2usize> = emcell::CellDefsMeta {
    cell_defs: [
        emcell::CellDefMeta {
            name: "Cell1ABI",
            cell_type: emcell::CellType::Primary,
            ram_range_start: 536870912usize,
            ram_range_end: 536875008usize,
            flash_range_start: 134217728usize,
            flash_range_end: 134221824usize,
        },
        emcell::CellDefMeta {
            name: "Cell2ABI",
            cell_type: emcell::CellType::NonPrimary,
            ram_range_start: 536870912usize,
            ram_range_end: 536875008usize,
            flash_range_start: 134217728usize,
            flash_range_end: 134221824usize,
        },
    ],
};

unsafe impl emcell::Cell for Cell2ABI {
    fn check_signature(&self) -> bool {
        let sig_valid = self.signature == 0xdeadbeef;
        if sig_valid {
            unsafe { (self.init_memory)() }
        }
        sig_valid
    }
}
unsafe impl emcell::Cell for Cell1ABI {
    fn check_signature(&self) -> bool {
        true
    }
}