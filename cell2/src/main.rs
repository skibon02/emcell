#![no_std]
#![no_main]

#![feature(const_refs_to_static)]
#![feature(panic_info_message)]

use core::ptr::read_volatile;
use core::sync::atomic::{compiler_fence, Ordering};
use at32f4xx_pac::at32f407::{CRM, gpiob, gpioc, gpioe};
use at32f4xx_pac::at32f407::gpioa::cfglr::{IOFC0_A, IOMC0_A};
use cortex_m::asm::delay;
use cortex_m::peripheral::SCB;
use defmt::{Debug2Format, error, info, unwrap};
use emcell_macro::{define_header, extern_header_forward};
use cells_defs::{Cell2, Cell3};

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


extern_header_forward!(Cell3Wrapper: Cell3);
#[inline(always)]
pub fn get_cpu_cyc() -> u32 {
    unsafe { &*cortex_m::peripheral::DWT::PTR }.cyccnt.read()
}

#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    error!("panic!: {:?}", Debug2Format(&info.message()));
    err_on();
    loop {}
}

#[defmt::panic_handler]
fn panic_handler_defmt() -> ! {
    error!("defmt panic!");
    err_on();
    loop {}
}

#[cortex_m_rt::exception]
unsafe fn HardFault(ef: &cortex_m_rt::ExceptionFrame) -> ! {
    error!("HardFault!");
    err_on();
    loop {}
}


#[cortex_m_rt::exception]
unsafe fn DefaultHandler(irq: i16) -> ! {
    error!("unknown irq: {}", irq);
    err_on();
    loop {}
}

pub fn memory_access_bench(range: core::ops::Range<usize>, data: &[u32]) {
    let mut ptr = &data[range.start] as *const u32;
    let start = get_cpu_cyc();
    for _ in range.clone() {
        unsafe { ptr.read_volatile() };
        ptr = unsafe { ptr.offset(1) };
    }

    let dur = get_cpu_cyc().wrapping_sub(start);
    info!("bench {}..{}", range.start, range.end);
    info!("Total time: {} cycles, {} cyc/element", dur, dur as f64/range.len() as f64);
}

fn gpio_cfgr() {

    let crm = unsafe {at32f4xx_pac::at32f407::CRM::steal()};
    let gpioe = unsafe {at32f4xx_pac::at32f407::GPIOE::steal()};
    let gpiob = unsafe {at32f4xx_pac::at32f407::GPIOB::steal()};

    crm.apb2en().modify(|_, w| w.gpioe().set_bit());
    crm.apb2en().modify(|_, w| w.gpiob().set_bit());

    gpioe.cfglr().modify(|_, w| w.iofc0().variant(IOFC0_A::Analog)
        .iomc0().variant(IOMC0_A::Output));
    gpiob.cfglr().modify(|_, w| w.iofc5().variant(IOFC0_A::Analog)
        .iomc5().variant(IOMC0_A::Output));
}

fn led_on() {
    let gpioe = unsafe {at32f4xx_pac::at32f407::GPIOE::steal()};
    gpioe.odt().modify(|_, w| w.odt0().set_bit());
}
fn led_off() {
    let gpioe = unsafe {at32f4xx_pac::at32f407::GPIOE::steal()};
    gpioe.odt().modify(|_, w| w.odt0().clear_bit());
}
fn err_on() {
    let gpiob = unsafe {at32f4xx_pac::at32f407::GPIOB::steal()};
    gpiob.odt().modify(|_, w| w.odt5().set_bit());
}
fn err_off() {
    let gpiob = unsafe {at32f4xx_pac::at32f407::GPIOB::steal()};
    gpiob.odt().modify(|_, w| w.odt5().clear_bit());
}

pub fn run() -> ! {

    let mut cp = unwrap!(cortex_m::Peripherals::take());

    cp.DCB.enable_trace();
    cortex_m::peripheral::DWT::unlock();
    cp.DWT.enable_cycle_counter();

    gpio_cfgr();


    if let Some(cell3) = Cell3Wrapper::new() {
        info!("cell2: b from cell3: {}", cell3.b);
        info!("cell2: Accessing static...");
        let v = (cell3.access_static)();
        info!("cell2: static value: 0x{:X}", v);

        loop {
            led_on();
            delay(5_000_000);
            led_off();
            delay(5_000_000);

            (cell3.run_some_code)();
        }
    }
    else {
        error!("CELL3 signature is not valid!");
        err_on();

        loop {
            delay(1_000_000);
        }
    }
}

pub fn print_some_value(v: u32) {
    info!("Someone asked us to print: {}", v);
}
