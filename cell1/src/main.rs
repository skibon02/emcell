#![no_std]
#![no_main]

mod critical_section;
mod mutex;

use core::slice::from_raw_parts;
use at32f4xx_pac::at32f407::{CRM, GPIOC, gpioc};
use defmt::{error, info, warn};
use emcell_macro::{define_primary_header, extern_header};
use cells_defs::{Cell1, Cell2};
use cortex_m::asm::delay;
use cortex_m::peripheral::SCB;

extern crate panic_halt;
extern crate at32f4xx_pac;
extern crate defmt_rtt;

define_primary_header!{
    Cell1 {
        a: 15,
        print_some_value,
    }
}

extern_header!(CELL2_wrapper: Cell2);

const EOPB0_ADDR: *mut u8 = 0x1fff_f810 as *mut u8;

fn is_extended_memory() -> bool {
    (unsafe {EOPB0_ADDR.read_volatile()} & 0b1) == 0
}

#[cortex_m_rt::entry]
unsafe fn main() -> ! {
    let crm = CRM::steal();
    crm.apb2en().modify(|_, w| w.gpioc().set_bit());

    let gpioc = GPIOC::steal();
    gpioc.cfglr().modify(|_, w| w.iofc0().variant(gpioc::cfglr::IOFC0_A::PullDownPullUp)
        .iomc0().variant(gpioc::cfglr::IOMC0_A::Input));
    gpioc.odt().write(|w| w.odt0().high());
    
    while !gpioc.idt().read().idt0().bit() {}
    info!("Primary cell started!");

    if !is_extended_memory() {
        warn!("Extended memory is not unlocked!");
        loop {
            let bit = gpioc.idt().read().idt0().bit();
            if !bit {
                SCB::sys_reset();
            }

            delay(1_000_000);
        }

    }


    if let Some(cell2) = CELL2_wrapper::new() {
        info!("cell1: b from cell2: {}", cell2.b);
        (cell2.run_some_code)();
        info!("cell1: Accessing static...");
        let v = (cell2.access_static)();
        info!("cell1: static value: 0x{:X}", v);
    }
    else {
        error!("CELL2 signature is not valid!");
    }
    loop {
        let bit = gpioc.idt().read().idt0().bit();
        if !bit {
            SCB::sys_reset();
        }

        delay(1_000_000);
    }
}

pub const FLASH_UNLOCK_KEY1: u32 = 0x4567_0123;
pub const FLASH_UNLOCK_KEY2: u32 = 0xCDEF_89AB;

#[cortex_m_rt::pre_init]
unsafe fn pre_init() {
    //unlock extended memory on at32f4xx
    let flash = at32f4xx_pac::at32f407::FLASH::steal();

    if is_extended_memory() {
    }
    else {
        flash.unlock().write(|w| w.ukval().variant(FLASH_UNLOCK_KEY1));
        flash.unlock().write(|w| w.ukval().variant(FLASH_UNLOCK_KEY2));

        while flash.sts().read().obf().bit_is_set() {}

        flash.usd_unlock().write(|w| w.usd_ukval().variant(FLASH_UNLOCK_KEY1));
        flash.usd_unlock().write(|w| w.usd_ukval().variant(FLASH_UNLOCK_KEY2));


        let success = flash.ctrl().read().usdulks().bit_is_set();
        if success {
            //read full USD f800 - f830
            let mut usd_data: [u8; 48] = [0xff; 48];
            let mut usd_loc = 0x1fff_f800 as *const u8;
            for i in 0..48 {
                usd_data[i] = usd_loc.read_volatile();
                usd_loc = usd_loc.offset(1);
            }
            //erase operation
            flash.ctrl().modify(|_, w| w.usders().set_bit()
                .erstr().set_bit());
            while flash.sts().read().obf().bit_is_set() {}
            flash.ctrl().modify(|_, w| w.usders().clear_bit());

            //set FAP to correct value
            usd_data[0] = 0xa5;
            usd_data[1] = 0x5a;

            usd_data[16] = 0xFE;
            usd_data[17] = 0x01;

            //program operation
            let mut usd_loc = 0x1fff_f800 as *mut u16;
            let usd_data_u16: &[u16] = from_raw_parts(usd_data.as_ptr() as *const u16, 24);

            for word in usd_data_u16 {
                flash.ctrl().modify(|_, w| w.usdprgm().set_bit());
                usd_loc.write_volatile(*word);
                usd_loc = usd_loc.offset(1);

                while flash.sts().read().obf().bit_is_set() {}
                let is_error = flash.sts().read().prgmerr().bit_is_set();
                let is_done = flash.sts().read().odf().bit_is_set();
                if !is_done || is_error {
                    loop {}
                }

                flash.ctrl().modify(|_, w| w.usdprgm().clear_bit());
            }
            SCB::sys_reset();
        }
    }
}

pub fn print_some_value(v: u32) {
    info!("Someone asked us to print: {}", v);
}
