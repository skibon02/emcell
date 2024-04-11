use core::sync::atomic::AtomicBool;
use cortex_m::interrupt;
use cortex_m::interrupt::CriticalSection;
use cortex_m::register::primask;
use critical_section::{set_impl, Impl, RawRestoreState};
use crate::mutex::FullMutex;


pub(crate) struct SingleCoreCriticalSection;
set_impl!(SingleCoreCriticalSection);


pub static CRITICAL_SECTION_STATE: FullMutex<CriticalSectionState> = FullMutex::new(CriticalSectionState::new());
pub static CRITICAL_SECTION_ACTIVE: AtomicBool = AtomicBool::new(false);

pub struct CriticalSectionState {
    depth: u32,
    // addresses: BTreeMap<usize, u32>,
}

impl CriticalSectionState {
    pub const fn new() -> Self {
        Self {
            depth: 0,
            // addresses: BTreeMap::new()
        }
    }
}

unsafe impl Impl for SingleCoreCriticalSection {
    unsafe fn acquire() -> RawRestoreState {
        let was_active = primask::read().is_active();
        interrupt::disable();
        CRITICAL_SECTION_ACTIVE.store(true, core::sync::atomic::Ordering::Relaxed);
        if was_active {
            // metrics_critical_section_start();
        }
        was_active
    }

    unsafe fn release(was_active: RawRestoreState) {
        // Only re-enable interrupts if they were enabled before the critical section.
        if was_active {
            // metrics_critical_section_end();
            CRITICAL_SECTION_ACTIVE.store(false, core::sync::atomic::Ordering::Relaxed);
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
        // unsafe { metrics_critical_section_start(); }

        let res = f(cs);

        // unsafe { metrics_critical_section_end(); }
        CRITICAL_SECTION_ACTIVE.store(false, core::sync::atomic::Ordering::Relaxed);
        res
    })
    // cortex_m::interrupt::free(|cs| {
    //     f(cs)
    // })
}