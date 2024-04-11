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

    // pub fn lock_int_free(&self, _cs: &cortex_m::interrupt::CriticalSection) -> MutexGuard<T> {
    //     let vec = SCB::vect_active();
    //     let vec = ExceptionHandler::from(vec);
    //     let vec_num = vec.get_exception_number();
    //
    //     //check locked state, panic if was locked from different exception handler
    //     let prev_state = self.state.load(core::sync::atomic::Ordering::Acquire);
    //     if prev_state != 0xffff_ffff
    //         && prev_state != vec_num as usize {
    //         let prev_state_cortex_m_vec = unwrap!(get_exception_from_number(prev_state as u8));
    //         let prev_state_vec = ExceptionHandler::from(prev_state_cortex_m_vec);
    //         defmt::panic!(
    //             "FullMutex was recurrently locked from different exception handler! previous context: {:?}, current context: {:?} Data: {:?}",
    //             prev_state_vec,
    //             vec,
    //             Debug2Format(&self.inner)
    //         );
    //     }
    //
    //     self.state
    //         .store(vec_num as usize, core::sync::atomic::Ordering::Release);
    //     MutexGuard { mutex: self, _phantom: core::marker::PhantomData }
    // }

    // pub fn lock_int_free_legacy(&self, _cs: critical_section::CriticalSection) -> MutexGuard<T> {
    //     unsafe { self.lock_int_free(&CriticalSection::new()) }
    // }

    /// Safety: Must be in critical section
    pub unsafe fn lock_unchecked(&self) -> &mut T {
        &mut *self.inner.get()
    }

    // pub fn with_lock<R>(
    //     &self,
    //     cs: &cortex_m::interrupt::CriticalSection,
    //     f: impl FnOnce(&mut T) -> R,
    // ) -> R {
    //     let mut guard = self.lock_int_free(cs);
    //     f(&mut *guard)
    // }

    /// Safety: Must be in critical section
    pub unsafe fn with_lock_uncheked<R>(
        &self,
        f: impl FnOnce(&mut T) -> R,
    ) -> R {
        let guard = self.lock_unchecked();
        f(&mut *guard)
    }
}

impl<T> FullMutex<T>
    where
        T: Copy,
{
    // pub fn locked_int_free_copy(&self) -> T {
    //     all_interrupt_free(|cs| {
    //         let res = self.lock_int_free(cs);
    //         *res
    //     })
    // }
}


pub struct MutexGuard<'a, T> {
    mutex: &'a FullMutex<T>,
    _phantom: core::marker::PhantomData<*mut ()>,
}

impl<'a, T> Drop for MutexGuard<'a, T> {
    fn drop(&mut self) {
        self.mutex
            .state
            .store(0xffff_ffff, core::sync::atomic::Ordering::Release);
    }
}

impl<'a, T> Deref for MutexGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        // Safety: the MutexGuard represents exclusive access to the contents
        // of the mutex, so it's OK to get it.
        unsafe { &*(self.mutex.inner.get() as *const T) }
    }
}

impl<'a, T> DerefMut for MutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // Safety: the MutexGuard represents exclusive access to the contents
        // of the mutex, so it's OK to get it.
        unsafe { &mut *(self.mutex.inner.get() as *mut T) }
    }
}


// Make it sync because Mutex guarantees that only one thread can access the resource at a time
unsafe impl<T> Sync for FullMutex<T> where T: Send {}
unsafe impl<T> Send for FullMutex<T> where T: Send {}
