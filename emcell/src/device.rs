
pub unsafe fn init_memory() {
    use core::ptr;

    extern "C" {
        static mut __sbss: u32;
        static mut __ebss: u32;

        static mut __sdata: u32;
        static mut __edata: u32;
        static mut __sidata: u32;
    }
    let count = &__ebss as *const u32 as usize - &__sbss as *const u32 as usize;
    let addr = &mut __sbss as *mut u32 as *mut u8;
    if count > 0 {
        ptr::write_bytes(addr, 0, count);
    }

    let count = &__edata as *const u32 as usize - &__sdata as *const u32 as usize;
    if count > 0 {
        ptr::copy_nonoverlapping(
            &__sidata as *const u32 as *const u8,
            &mut __sdata as *mut u32 as *mut u8,
            count);
    }
}

