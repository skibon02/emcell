#![no_std]
#![no_main]

#![feature(const_refs_to_static)]

use at32f4xx_pac::at32f437::{CRM, gpiob, gpioc, gpioe};
use cortex_m::asm::delay;
use emcell_macro::{define_header, extern_header_backward};
use cells_defs::{Cell3, Cell2};

extern crate panic_halt;
extern crate at32f4xx_pac;

define_header!{
    Cell3 {
        b: 23,
        run_some_code,
        access_static,
    }
}

extern_header_backward!(Cell2Wrapper: Cell2);

pub fn run_some_code() {

    let crm = unsafe { CRM::steal() };
    crm.ahben1().modify(|_, w| w.gpioe().set_bit());
    crm.ahben1().modify(|_, w| w.gpioc().set_bit());
    crm.ahben1().modify(|_, w| w.gpiob().set_bit());

    let gpioe = unsafe { at32f4xx_pac::at32f437::GPIOE::steal() };
    gpioe.cfgr().modify(|_, w| w
        .iomc0().variant(gpioe::cfgr::IOMC0_A::Output));
    gpioe.cfgr().modify(|_, w| w
        .iomc1().variant(gpioe::cfgr::IOMC0_A::Output));

    let gpiob = unsafe { at32f4xx_pac::at32f437::GPIOB::steal() };
    gpiob.cfgr().modify(|_, w| w
        .iomc5().variant(gpiob::cfgr::IOMC0_A::Output));

    let gpioc = unsafe { at32f4xx_pac::at32f437::GPIOC::steal() };
    gpioc.cfgr().modify(|_, w| w
        .iomc15().variant(gpioc::cfgr::IOMC0_A::Output));

    gpioc.odt().write(|w| w.odt15().high());

    delay(1_000_000);
    gpioe.odt().write(|w| w.odt0().set_bit()
        .odt1().set_bit());
    gpiob.odt().write(|w| w.odt5().set_bit());

    delay(1_000_000);
    gpioe.odt().write(|w| w.odt0().clear_bit()
        .odt1().clear_bit());
    gpiob.odt().write(|w| w.odt5().clear_bit());
    
    if let Some(cell2) = Cell2Wrapper::new() {
        (cell2.print_some_value)(cell2.a)
    }
}

pub const FLASH_UNLOCK_KEY1: u32 = 0x4567_0123;
pub fn access_static() -> u32 {
    FLASH_UNLOCK_KEY1
}