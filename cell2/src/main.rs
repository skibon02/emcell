#![no_std]
#![no_main]

use emcell_macro::{define_header, extern_header};
use cells_defs::{Cell1, Cell2};

extern crate panic_halt;
extern crate at32f4xx_pac;

define_header!{
    Cell2 {
        b: 23,
        run_some_code
    }
}

extern_header!(CELL1ABI_wrapper: Cell1);

pub fn run_some_code() {
    if let Some(CELL1_ABI) = CELL1ABI_wrapper::new() {
        (CELL1_ABI.print_some_value)(CELL1_ABI.a)
    }
}