#![no_std]
#![no_main]

#![feature(const_refs_to_static)]

use at32f4xx_pac::at32f437::gpioa::cfgr::IOMC0_A;

use emcell_macro::{define_primary_header, extern_header_forward};
use cells_defs::{Cell1, Cell2};
use cortex_m::asm::delay;

extern crate panic_halt;
extern crate at32f4xx_pac;

define_primary_header!{
    Cell1 {
    }
}

extern_header_forward!(Cell2Wrapper: Cell2);


fn gpio_cfgr() {

    let crm = unsafe {at32f4xx_pac::at32f437::CRM::steal()};
    let gpioe = unsafe {at32f4xx_pac::at32f437::GPIOE::steal()};

    crm.ahben1().modify(|_, w| w.gpioe().set_bit());

    gpioe.cfgr().modify(|_, w| w.iomc0().variant(IOMC0_A::Output));
}

fn led_on() {
    let gpioe = unsafe {at32f4xx_pac::at32f437::GPIOE::steal()};
    gpioe.odt().modify(|_, w| w.odt0().set_bit());
}
fn led_off() {
    let gpioe = unsafe {at32f4xx_pac::at32f437::GPIOE::steal()};
    gpioe.odt().modify(|_, w| w.odt0().clear_bit());
}

#[cortex_m_rt::entry]
unsafe fn main() -> ! {
    gpio_cfgr();
    led_on();

    if let Some(cell2) = Cell2Wrapper::new() {
        cell2.switch_vectors_and_run()

    }
    else {
        loop {
            delay(1_000_000);
        }
    }
}