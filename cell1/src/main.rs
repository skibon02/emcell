#![no_std]
#![no_main]

mod critical_section;
mod mutex;

use defmt::{error, info};
use emcell_macro::{define_primary_abi, extern_abi};
use cells_defs::{Cell1ABI, Cell2ABI};

extern crate panic_halt;
extern crate at32f4xx_pac;
extern crate defmt_rtt;


define_primary_abi!{
    Cell1ABI {
        a: 15,
        print_some_value,
    }
}

extern_abi!(CELL2ABI_wrapper: Cell2ABI);

#[cortex_m_rt::entry]
fn main() -> ! {
    info!("Primary cell started!");
    if let Some(CELL2_ABI) = CELL2ABI_wrapper::new() {
        info!("b from abi2: {}", CELL2_ABI.b);
        (CELL2_ABI.run_some_code)();
        info!("ok");
    }
    else {
        error!("ABI2 signature is not valid!");
    }
    loop {}
}

pub fn print_some_value(v: u32) {
    info!("Someone asked us to print: {}", v);
}
