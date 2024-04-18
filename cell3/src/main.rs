#![no_std]
#![no_main]

#![feature(const_refs_to_static)]

use at32f4xx_pac::at32f407::{CRM, gpiob, gpioc, gpioe};
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

pub fn run() -> ! {
    let crm = unsafe { CRM::steal() };
    crm.apb2en().modify(|_, w| w.gpioe().set_bit());
    crm.apb2en().modify(|_, w| w.gpioc().set_bit());
    crm.apb2en().modify(|_, w| w.gpiob().set_bit());

    let gpioe = unsafe { at32f4xx_pac::at32f407::GPIOE::steal() };
    gpioe.cfglr().modify(|_, w| w
        .iomc0().variant(gpioe::cfglr::IOMC0_A::OutputLarge)
        .iofc0().variant(gpioe::cfglr::IOFC0_A::Analog));
    gpioe.cfglr().modify(|_, w| w
        .iomc1().variant(gpioe::cfglr::IOMC0_A::OutputLarge)
        .iofc1().variant(gpioe::cfglr::IOFC0_A::Analog));

    let gpiob = unsafe { at32f4xx_pac::at32f407::GPIOB::steal() };
    gpiob.cfglr().modify(|_, w| w
        .iomc5().variant(gpiob::cfglr::IOMC0_A::OutputLarge)
        .iofc5().variant(gpiob::cfglr::IOFC0_A::Analog));

    let gpioc = unsafe { at32f4xx_pac::at32f407::GPIOC::steal() };
    gpioc.cfghr().modify(|_, w| w
        .iomc15().variant(gpioc::cfghr::IOMC8_A::OutputLarge)
        .iofc15().variant(gpioc::cfghr::IOFC8_A::Analog));

    gpioc.odt().write(|w| w.odt15().high());

    loop {
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
}

pub fn run_some_code() {

    let crm = unsafe { CRM::steal() };
    crm.apb2en().modify(|_, w| w.gpioe().set_bit());
    crm.apb2en().modify(|_, w| w.gpioc().set_bit());
    crm.apb2en().modify(|_, w| w.gpiob().set_bit());

    let gpioe = unsafe { at32f4xx_pac::at32f407::GPIOE::steal() };
    gpioe.cfglr().modify(|_, w| w
        .iomc0().variant(gpioe::cfglr::IOMC0_A::OutputLarge)
        .iofc0().variant(gpioe::cfglr::IOFC0_A::Analog));
    gpioe.cfglr().modify(|_, w| w
        .iomc1().variant(gpioe::cfglr::IOMC0_A::OutputLarge)
        .iofc1().variant(gpioe::cfglr::IOFC0_A::Analog));

    let gpiob = unsafe { at32f4xx_pac::at32f407::GPIOB::steal() };
    gpiob.cfglr().modify(|_, w| w
        .iomc5().variant(gpiob::cfglr::IOMC0_A::OutputLarge)
        .iofc5().variant(gpiob::cfglr::IOFC0_A::Analog));

    let gpioc = unsafe { at32f4xx_pac::at32f407::GPIOC::steal() };
    gpioc.cfghr().modify(|_, w| w
        .iomc15().variant(gpioc::cfghr::IOMC8_A::OutputLarge)
        .iofc15().variant(gpioc::cfghr::IOFC8_A::Analog));

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