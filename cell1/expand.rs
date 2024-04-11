#![feature(prelude_import)]
#![no_std]
#![no_main]
#[prelude_import]
use core::prelude::rust_2021::*;
#[macro_use]
extern crate core;
extern crate compiler_builtins as _;
mod critical_section {
    use core::sync::atomic::AtomicBool;
    use cortex_m::interrupt;
    use cortex_m::interrupt::CriticalSection;
    use cortex_m::register::primask;
    use critical_section::{set_impl, Impl, RawRestoreState};
    use crate::mutex::FullMutex;
    pub(crate) struct SingleCoreCriticalSection;
    #[no_mangle]
    unsafe fn _critical_section_1_0_acquire() -> ::critical_section::RawRestoreState {
        <SingleCoreCriticalSection as ::critical_section::Impl>::acquire()
    }
    #[no_mangle]
    unsafe fn _critical_section_1_0_release(
        restore_state: ::critical_section::RawRestoreState,
    ) {
        <SingleCoreCriticalSection as ::critical_section::Impl>::release(restore_state)
    }
    pub static CRITICAL_SECTION_STATE: FullMutex<CriticalSectionState> = FullMutex::new(
        CriticalSectionState::new(),
    );
    pub static CRITICAL_SECTION_ACTIVE: AtomicBool = AtomicBool::new(false);
    pub struct CriticalSectionState {
        depth: u32,
    }
    impl CriticalSectionState {
        pub const fn new() -> Self {
            Self { depth: 0 }
        }
    }
    unsafe impl Impl for SingleCoreCriticalSection {
        unsafe fn acquire() -> RawRestoreState {
            let was_active = primask::read().is_active();
            interrupt::disable();
            CRITICAL_SECTION_ACTIVE.store(true, core::sync::atomic::Ordering::Relaxed);
            if was_active {}
            was_active
        }
        unsafe fn release(was_active: RawRestoreState) {
            if was_active {
                CRITICAL_SECTION_ACTIVE
                    .store(false, core::sync::atomic::Ordering::Relaxed);
                interrupt::enable();
            }
        }
    }
    #[inline(always)]
    /// Critical section: disable all exceptions (SysTick included)
    pub fn all_interrupt_free<F, R>(f: F) -> R
    where
        F: FnOnce(&CriticalSection) -> R,
    {
        interrupt::free(|cs| {
            CRITICAL_SECTION_ACTIVE.store(true, core::sync::atomic::Ordering::Relaxed);
            let res = f(cs);
            CRITICAL_SECTION_ACTIVE.store(false, core::sync::atomic::Ordering::Relaxed);
            res
        })
    }
}
mod mutex {
    use core::cell::UnsafeCell;
    use core::ops::{Deref, DerefMut};
    use core::sync::atomic::AtomicUsize;
    use cortex_m::interrupt::CriticalSection;
    use cortex_m::peripheral::SCB;
    use defmt::{Debug2Format, unwrap};
    use crate::critical_section::all_interrupt_free;
    /// Make inner data safe to use from all exceptions including SysTick (but not HardFault).
    pub struct FullMutex<T> {
        inner: UnsafeCell<T>,
        state: AtomicUsize,
    }
    impl<T> FullMutex<T> {
        pub const fn new(data: T) -> FullMutex<T> {
            FullMutex {
                inner: UnsafeCell::new(data),
                state: AtomicUsize::new(0xffff_ffff),
            }
        }
        /// Safety: Must be in critical section
        pub unsafe fn lock_unchecked(&self) -> &mut T {
            &mut *self.inner.get()
        }
        /// Safety: Must be in critical section
        pub unsafe fn with_lock_uncheked<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
            let guard = self.lock_unchecked();
            f(&mut *guard)
        }
    }
    impl<T> FullMutex<T>
    where
        T: Copy,
    {}
    pub struct MutexGuard<'a, T> {
        mutex: &'a FullMutex<T>,
        _phantom: core::marker::PhantomData<*mut ()>,
    }
    impl<'a, T> Drop for MutexGuard<'a, T> {
        fn drop(&mut self) {
            self.mutex.state.store(0xffff_ffff, core::sync::atomic::Ordering::Release);
        }
    }
    impl<'a, T> Deref for MutexGuard<'a, T> {
        type Target = T;
        fn deref(&self) -> &Self::Target {
            unsafe { &*(self.mutex.inner.get() as *const T) }
        }
    }
    impl<'a, T> DerefMut for MutexGuard<'a, T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            unsafe { &mut *(self.mutex.inner.get() as *mut T) }
        }
    }
    unsafe impl<T> Sync for FullMutex<T>
    where
        T: Send,
    {}
    unsafe impl<T> Send for FullMutex<T>
    where
        T: Send,
    {}
}
use cells_defs::{Cell1ABI, Cell2ABI};
use defmt::{error, info};
use emcell_macro::{declare_abi_cell1, extern_abi};
extern crate defmt_rtt;
extern crate panic_halt;
extern crate at32f4xx_pac;
#[no_mangle]
#[link_section = ".CELL1_ABI"]
pub static CELL1_ABI: Cell1ABI = Cell1ABI {
    a: 15,
    print_some_value,
};
pub unsafe fn init_memory() {}
extern crate emcell;
extern "Rust" {
    pub static _emcell_Cell2ABI_internal: Cell2ABI;
}
pub type CELL2ABI_wrapper = emcell::CellWrapper<Cell2ABI>;
pub unsafe trait CellWrapperTrait {
    type CellWrapperType;
    fn new() -> Option<Self::CellWrapperType>;
    fn new_uninit() -> Self::CellWrapperType;
}
unsafe impl CellWrapperTrait for CELL2ABI_wrapper {
    type CellWrapperType = CELL2ABI_wrapper;
    fn new() -> Option<Self> {
        let cell = unsafe { &_emcell_Cell2ABI_internal };
        Some(unsafe { emcell::CellWrapper::_new_init(cell) })
    }
    fn new_uninit() -> Self {
        let cell = unsafe { &_emcell_Cell2ABI_internal };
        unsafe { emcell::CellWrapper::_new_uninit(cell) }
    }
}
#[doc(hidden)]
#[export_name = "main"]
pub unsafe extern "C" fn __cortex_m_rt_main_trampoline() {
    __cortex_m_rt_main()
}
fn __cortex_m_rt_main() -> ! {
    match () {
        () => {
            if {
                const CHECK: bool = {
                    const fn check() -> bool {
                        let module_path = "cell1".as_bytes();
                        if if 5usize > module_path.len() {
                            false
                        } else {
                            module_path[0usize] == 99u8 && module_path[1usize] == 101u8
                                && module_path[2usize] == 108u8
                                && module_path[3usize] == 108u8
                                && module_path[4usize] == 49u8
                                && if 5usize == module_path.len() {
                                    true
                                } else {
                                    module_path[5usize] == b':'
                                }
                        } {
                            return true;
                        }
                        false
                    }
                    check()
                };
                CHECK
            } {
                unsafe { defmt::export::acquire() };
                defmt::export::header(
                    &{
                        defmt::export::make_istr({
                            #[link_section = ".defmt.{\"package\":\"cell1\",\"tag\":\"defmt_info\",\"data\":\"Primary cell started!\",\"disambiguator\":\"11376590466960392980\",\"crate_name\":\"cell1\"}"]
                            #[export_name = "{\"package\":\"cell1\",\"tag\":\"defmt_info\",\"data\":\"Primary cell started!\",\"disambiguator\":\"11376590466960392980\",\"crate_name\":\"cell1\"}"]
                            static DEFMT_LOG_STATEMENT: u8 = 0;
                            &DEFMT_LOG_STATEMENT as *const u8 as u16
                        })
                    },
                );
                unsafe { defmt::export::release() }
            }
        }
    };
    if let Some(CELL2_ABI) = CELL2ABI_wrapper::new() {
        match (&(CELL2_ABI.b)) {
            (arg0) => {
                if {
                    const CHECK: bool = {
                        const fn check() -> bool {
                            let module_path = "cell1".as_bytes();
                            if if 5usize > module_path.len() {
                                false
                            } else {
                                module_path[0usize] == 99u8 && module_path[1usize] == 101u8
                                    && module_path[2usize] == 108u8
                                    && module_path[3usize] == 108u8
                                    && module_path[4usize] == 49u8
                                    && if 5usize == module_path.len() {
                                        true
                                    } else {
                                        module_path[5usize] == b':'
                                    }
                            } {
                                return true;
                            }
                            false
                        }
                        check()
                    };
                    CHECK
                } {
                    unsafe { defmt::export::acquire() };
                    defmt::export::header(
                        &{
                            defmt::export::make_istr({
                                #[link_section = ".defmt.{\"package\":\"cell1\",\"tag\":\"defmt_info\",\"data\":\"b from abi2: {}\",\"disambiguator\":\"12604949159135316568\",\"crate_name\":\"cell1\"}"]
                                #[export_name = "{\"package\":\"cell1\",\"tag\":\"defmt_info\",\"data\":\"b from abi2: {}\",\"disambiguator\":\"12604949159135316568\",\"crate_name\":\"cell1\"}"]
                                static DEFMT_LOG_STATEMENT: u8 = 0;
                                &DEFMT_LOG_STATEMENT as *const u8 as u16
                            })
                        },
                    );
                    defmt::export::fmt(arg0);
                    unsafe { defmt::export::release() }
                }
            }
        };
        (CELL2_ABI.run_some_code)();
        match () {
            () => {
                if {
                    const CHECK: bool = {
                        const fn check() -> bool {
                            let module_path = "cell1".as_bytes();
                            if if 5usize > module_path.len() {
                                false
                            } else {
                                module_path[0usize] == 99u8 && module_path[1usize] == 101u8
                                    && module_path[2usize] == 108u8
                                    && module_path[3usize] == 108u8
                                    && module_path[4usize] == 49u8
                                    && if 5usize == module_path.len() {
                                        true
                                    } else {
                                        module_path[5usize] == b':'
                                    }
                            } {
                                return true;
                            }
                            false
                        }
                        check()
                    };
                    CHECK
                } {
                    unsafe { defmt::export::acquire() };
                    defmt::export::header(
                        &{
                            defmt::export::make_istr({
                                #[link_section = ".defmt.{\"package\":\"cell1\",\"tag\":\"defmt_info\",\"data\":\"ok\",\"disambiguator\":\"18197332010060426912\",\"crate_name\":\"cell1\"}"]
                                #[export_name = "{\"package\":\"cell1\",\"tag\":\"defmt_info\",\"data\":\"ok\",\"disambiguator\":\"18197332010060426912\",\"crate_name\":\"cell1\"}"]
                                static DEFMT_LOG_STATEMENT: u8 = 0;
                                &DEFMT_LOG_STATEMENT as *const u8 as u16
                            })
                        },
                    );
                    unsafe { defmt::export::release() }
                }
            }
        };
    } else {
        match () {
            () => {
                if {
                    const CHECK: bool = {
                        const fn check() -> bool {
                            let module_path = "cell1".as_bytes();
                            if if 5usize > module_path.len() {
                                false
                            } else {
                                module_path[0usize] == 99u8 && module_path[1usize] == 101u8
                                    && module_path[2usize] == 108u8
                                    && module_path[3usize] == 108u8
                                    && module_path[4usize] == 49u8
                                    && if 5usize == module_path.len() {
                                        true
                                    } else {
                                        module_path[5usize] == b':'
                                    }
                            } {
                                return true;
                            }
                            false
                        }
                        check()
                    };
                    CHECK
                } {
                    unsafe { defmt::export::acquire() };
                    defmt::export::header(
                        &{
                            defmt::export::make_istr({
                                #[link_section = ".defmt.{\"package\":\"cell1\",\"tag\":\"defmt_error\",\"data\":\"ABI2 signature is not valid!\",\"disambiguator\":\"4188030852262576386\",\"crate_name\":\"cell1\"}"]
                                #[export_name = "{\"package\":\"cell1\",\"tag\":\"defmt_error\",\"data\":\"ABI2 signature is not valid!\",\"disambiguator\":\"4188030852262576386\",\"crate_name\":\"cell1\"}"]
                                static DEFMT_LOG_STATEMENT: u8 = 0;
                                &DEFMT_LOG_STATEMENT as *const u8 as u16
                            })
                        },
                    );
                    unsafe { defmt::export::release() }
                }
            }
        };
    }
    loop {}
}
pub fn print_some_value(v: u32) {
    match (&(v)) {
        (arg0) => {
            if {
                const CHECK: bool = {
                    const fn check() -> bool {
                        let module_path = "cell1".as_bytes();
                        if if 5usize > module_path.len() {
                            false
                        } else {
                            module_path[0usize] == 99u8 && module_path[1usize] == 101u8
                                && module_path[2usize] == 108u8
                                && module_path[3usize] == 108u8
                                && module_path[4usize] == 49u8
                                && if 5usize == module_path.len() {
                                    true
                                } else {
                                    module_path[5usize] == b':'
                                }
                        } {
                            return true;
                        }
                        false
                    }
                    check()
                };
                CHECK
            } {
                unsafe { defmt::export::acquire() };
                defmt::export::header(
                    &{
                        defmt::export::make_istr({
                            #[link_section = ".defmt.{\"package\":\"cell1\",\"tag\":\"defmt_info\",\"data\":\"Someone asked us to print: {}\",\"disambiguator\":\"8125848569838665430\",\"crate_name\":\"cell1\"}"]
                            #[export_name = "{\"package\":\"cell1\",\"tag\":\"defmt_info\",\"data\":\"Someone asked us to print: {}\",\"disambiguator\":\"8125848569838665430\",\"crate_name\":\"cell1\"}"]
                            static DEFMT_LOG_STATEMENT: u8 = 0;
                            &DEFMT_LOG_STATEMENT as *const u8 as u16
                        })
                    },
                );
                defmt::export::fmt(arg0);
                unsafe { defmt::export::release() }
            }
        }
    };
}
