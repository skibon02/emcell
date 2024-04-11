#![no_std]
#![no_main]

mod critical_section;
mod mutex;

use core::slice::from_raw_parts;
use defmt::{error, info};
use emcell_macro::{define_primary_header, extern_header};
use cells_defs::{Cell1, Cell2};

extern crate panic_halt;
extern crate at32f4xx_pac;
extern crate defmt_rtt;


define_primary_header!{
    Cell1 {
        a: 15,
        print_some_value,
    }
}

extern_header!(CELL2ABI_wrapper: Cell2);

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

#[cortex_m_rt::pre_init]
unsafe fn pre_init() {
    //unlock extended memory on at32f4xx
    let flash = at32f4xx_pac::at32f407::FLASH::steal();

    let EOPB0_addr = 0x1fff_f810 as *mut u16;
    let data = EOPB0_addr.read_volatile();
    if data & 0b1 == 0 {
        // info!("Extended memory is already unlocked!");
    }
    else {
        flash.unlock().write(|w| w.ukval().variant(0x4567_0123));
        flash.unlock().write(|w| w.ukval().variant(0xCDEF_89AB));

        flash.usd_unlock().write(|w| w.usd_ukval().variant(0x4567_0123));
        flash.usd_unlock().write(|w| w.usd_ukval().variant(0xCDEF_89AB));


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
            while flash.sts().read().obf().bit_is_set() {}
            flash.ctrl().modify(|_, w| w.usders().set_bit()
                .erstr().set_bit());
            while flash.sts().read().obf().bit_is_set() {}

            let usd = flash.usd().read();
            usd_data[0] = usd.fap().bit() as u8;
            usd_data[1] = !usd_data[0];

            let ssb: u8 = usd.n_wdt_ato_en().bit() as u8 | ((usd.n_depslp_rst().bit() as u8) << 1) |
                ((usd.n_stdby_rst().bit() as u8) << 2) | ((usd.btopt().bit() as u8) << 3);
            usd_data[2] = ssb;
            usd_data[3] = !ssb;

            usd_data[4] = usd.user_d0().bits();
            usd_data[5] = !usd_data[4];
            
            usd_data[6] = usd.user_d1().bits();
            usd_data[7] = !usd_data[6];
            
            let flash_epps = flash.epps().read();
            
            usd_data[8] = flash_epps.epps().bits()

            //modify eopb0 word
            usd_data[16] = 0x00;
            usd_data[17] = 0xFF;

            //program operation
            // let mut usd_loc = 0x1fff_f800 as *mut u16;
            // let usd_data_u16: &[u16] = from_raw_parts(usd_data.as_ptr() as *const u16, 24);
            // for word in usd_data_u16 {
            //     flash.ctrl().modify(|_, w| w.usdprgm().set_bit());
            //     usd_loc.write_volatile(*word);
            //     usd_loc = usd_loc.offset(1);
            //
            //     while flash.sts().read().obf().bit_is_set() {}
            //     let is_error = flash.sts().read().prgmerr().bit_is_set();
            //     let is_done = flash.sts().read().odf().bit_is_set();
            //     // if is_done && !is_error {
            //     // }
            //     flash.ctrl().modify(|_, w| w.usdprgm().clear_bit());
            // }

            //wait for the operation to finish
            while flash.sts().read().obf().bit_is_set() {}
            // //lock the flash
            flash.ctrl().modify(|_, w| w.usdulks().clear_bit());
            flash.ctrl().modify(|_, w| w.oplk().set_bit());
        }
    }
}

pub fn print_some_value(v: u32) {
    info!("Someone asked us to print: {}", v);
}
