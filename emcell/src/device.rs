pub unsafe fn init() {
    init_memory();
}

pub unsafe fn init_primary() {
}

use core::ptr::{addr_of, addr_of_mut};

/// Initialize BSS and DATA sections
/// Should not be called directly! instead, use `init` function from the generated header
#[cfg(feature = "rt-crate-cortex-m-rt")]
pub unsafe fn init_memory() {
    use core::ptr;

    extern "C" {
        static mut __sbss: u32;
        static mut __ebss: u32;

        static mut __sdata: u32;
        static mut __edata: u32;
        static mut __sidata: u32;
    }
    let count = addr_of!(__ebss) as usize - addr_of!(__sbss) as usize;
    let addr = addr_of_mut!(__sbss) as *mut u8;
    if count > 0 {
        ptr::write_bytes(addr, 0, count);
    }

    let count = addr_of!(__edata) as usize - addr_of!(__sdata) as usize;
    if count > 0 {
        ptr::copy_nonoverlapping(
            addr_of!(__sidata) as *const u8,
            addr_of_mut!(__sdata) as *mut u8,
            count);
    }
}