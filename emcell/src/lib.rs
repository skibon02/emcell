#![no_std]

#[cfg(not(feature = "rt-crate-cortex-m-rt"))]
compile_error!("This crate requires any rt-crate-* to be enabled (when using build-rs feature)! *currently only rt-crate-cortex-m-rt is supported*");


#[cfg(feature = "build-rs")]
mod build_rs;

use core::marker::PhantomData;
#[cfg(feature = "build-rs")]
pub use build_rs::*;

use core::sync::atomic::AtomicBool;

pub mod meta;

#[cfg(not(feature = "build-rs"))]
pub mod device;

#[derive(Copy, Clone)]
pub enum CellType {
    Primary,
    NonPrimary
}

#[derive(PartialEq, Copy, Clone)]
pub enum HeaderType {
    Actual,
    Dummy
}

pub unsafe trait WithSignature {
    const VALID_SIGNATURE: u32;
}

pub unsafe trait Cell: WithSignature {
    const CUR_META: meta::CellDefMeta;
    const CELLS_META: &'static [meta::CellDefMeta];
    const DEVICE_CONFIG: meta::DeviceConfigMeta;
    fn check_signature(&self, init_memory: bool) -> bool;
    fn static_sha256(&self) -> [u8; 32] {
        Self::CUR_META.struct_sha256
    }
}

/// Safe cell header wrapper.
/// If you create a CellWrapper with new_uninit, you should call ensure_init to handle
pub struct CellWrapper<T, K>
where T: 'static {
    header: &'static T,
    header_type: HeaderType,
    is_init: AtomicBool,
    _phantom: PhantomData<K>
}

pub struct Forward;
pub struct Backward;

impl<T> core::ops::Deref for CellWrapper<T, Forward>
    where T: Cell + 'static {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        if !self.is_init.load(core::sync::atomic::Ordering::Relaxed) {
            //attempt to initialize
            if self.ensure_init().is_some() {
                self.is_init.store(true, core::sync::atomic::Ordering::Relaxed);
                return self.header;
            }
            panic!("CellWrapper initialization failed!");
        }
        self.header
    }
}


impl<T> core::ops::Deref for CellWrapper<T, Backward>
    where T: Cell + 'static {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        if !self.is_init.load(core::sync::atomic::Ordering::Relaxed) {
            //attempt to initialize
            if self.ensure_init().is_some() {
                self.is_init.store(true, core::sync::atomic::Ordering::Relaxed);
                return self.header;
            }
            panic!("CellWrapper initialization failed!");
        }
        self.header
    }
}


impl<T, K> CellWrapper<T, K>
    where T: Cell + 'static {
    pub const unsafe fn _new_uninit(h: &'static T) -> Self {
        Self {
            header: h,
            header_type: HeaderType::Actual,
            is_init: AtomicBool::new(false),
            _phantom: PhantomData
        }
    }

    pub const fn new_dummy(dummy_header: &'static T) -> Self {
        Self {
            header: dummy_header,
            header_type: HeaderType::Dummy,
            is_init: AtomicBool::new(true),
            _phantom: PhantomData
        }
    }
}


impl<T> CellWrapper<T, Forward>
    where T: Cell + 'static {
    pub unsafe fn _new_init(h: &'static T) -> Option<Self> {
        if !h.check_signature(true) {
            return None;
        }

        Some(Self {
            header: h,
            header_type: HeaderType::Actual,
            is_init: AtomicBool::new(true),
            _phantom: PhantomData
        })
    }

    /// If header wrapper was created with new_uninit, this function must be called to potentially initialize other cell's memory.
    pub fn ensure_init(&self) -> Option<()> {
        if self.is_init.load(core::sync::atomic::Ordering::Relaxed) {
            return Some(());
        }
        //init
        if !self.header.check_signature(true) {
            return None;
        }
        self.is_init.store(true, core::sync::atomic::Ordering::Relaxed);
        Some(())
    }

    pub fn is_dummy(&self) -> bool {
        self.header_type == HeaderType::Dummy
    }
}



impl<T> CellWrapper<T, Backward>
    where T: Cell + 'static {
    pub unsafe fn _new_init(h: &'static T) -> Option<Self> {
        if !h.check_signature(false) {
            return None;
        }

        Some(Self {
            header: h,
            header_type: HeaderType::Actual,
            is_init: AtomicBool::new(true),
            _phantom: PhantomData
        })
    }

    /// If header wrapper was created with new_uninit, this function must be called to potentially initialize other cell's memory.
    pub fn ensure_init(&self) -> Option<()> {
        if self.is_init.load(core::sync::atomic::Ordering::Relaxed) {
            return Some(());
        }
        //init
        if !self.header.check_signature(false) {
            return None;
        }
        self.is_init.store(true, core::sync::atomic::Ordering::Relaxed);
        Some(())
    }

    pub fn is_dummy(&self) -> bool {
        self.header_type == HeaderType::Dummy
    }
}