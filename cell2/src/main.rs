#![no_std]
#![no_main]

#![feature(const_refs_to_static)]

use at32f4xx_pac::at32f407::{CRM, gpiob, gpioc, gpioe};
use cortex_m::asm::delay;
use defmt::{error, info};
use emcell_macro::{define_header, extern_header_forward};
use cells_defs::{Cell2, Cell3};

extern crate panic_halt;
extern crate at32f4xx_pac;
extern crate defmt_rtt;

define_header!{
    Cell2 {
        a: 15,
        print_some_value,
        run,
        _emcell_internal_switch_vectors: emcell::device::switch_vectors,
    }
}

// extern_header_backward!(Cell1Wrapper: Cell1);
extern_header_forward!(Cell3Wrapper: Cell3);

pub fn run() -> ! {

    let cell3_start_ptr = Cell3::get_cell_start_flash_addr();
    let cell3_end_ptr = Cell3::get_cell_end_flash_addr();
    info!("cell2: Cell3 start: 0x{:X}, end: 0x{:X}", cell3_start_ptr as u32, cell3_end_ptr as u32);

    if let Some(cell3) = Cell3Wrapper::new() {
        info!("cell2: b from cell3: {}", cell3.b);
        info!("cell2: Accessing static...");
        let v = (cell3.access_static)();
        info!("cell2: static value: 0x{:X}", v);

        loop {
            delay(1_000_000);
            (cell3.run_some_code)();
        }
    }
    else {
        error!("CELL2 signature is not valid!");

        loop {
            delay(1_000_000);
        }
    }
}

pub fn print_some_value(v: u32) {
    info!("Someone asked us to print: {}", v);
}
