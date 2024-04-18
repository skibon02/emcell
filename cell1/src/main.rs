#![no_std]
#![no_main]

#![feature(const_refs_to_static)]

use core::slice::from_raw_parts;
use at32f4xx_pac::at32f407::{CRM, GPIOC, gpioc, gpioe};
use emcell_macro::{define_primary_header, extern_header_forward};
use cells_defs::{Cell1, Cell2};
use cortex_m::asm::delay;
use cortex_m::peripheral::SCB;

extern crate panic_halt;
extern crate at32f4xx_pac;

define_primary_header!{
    Cell1 {
    }
}

extern_header_forward!(Cell2Wrapper: Cell2);

const EOPB0_ADDR: *mut u8 = 0x1fff_f810 as *mut u8;

fn is_extended_memory() -> bool {
    (unsafe {EOPB0_ADDR.read_volatile()} & 0b1) == 0
}

#[cortex_m_rt::entry]
unsafe fn main() -> ! {

    if !is_extended_memory() {
        loop {
            delay(1_000_000);
        }
    }

    if let Some(cell2) = Cell2Wrapper::new() {
        cell2.switch_vectors_and_run()

    }
    else {
        loop {
            delay(1_000_000);
        }
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
