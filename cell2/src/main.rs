#![no_std]
#![no_main]

use core::ptr;
use emcell_macro::{define_abi, extern_abi};
use cells_defs::{Cell1ABI, Cell2ABI};

extern crate panic_halt;
extern crate at32f4xx_pac;

define_abi!{
    Cell2ABI {
        b: 23,
        run_some_code
    }
}

extern_abi!(CELL1ABI_wrapper: Cell1ABI);

pub fn run_some_code() {
    if let Some(CELL1_ABI) = CELL1ABI_wrapper::new() {
        (CELL1_ABI.print_some_value)(CELL1_ABI.a)
    }
}