#![no_std]

#[cfg(not(feature = "rt-crate-cortex-m-rt"))]
compile_error!("This crate requires any rt-crate-* to be enabled (when using build-rs feature)! *currently only rt-crate-cortex-m-rt is supported*");


#[cfg(feature = "build-rs")]
mod build_rs;

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

pub enum HeaderType {
    Actual,
    Dummy
}

pub unsafe trait Cell {
    const CUR_META: meta::CellDefMeta;
    const CELLS_META: &'static [meta::CellDefMeta];
    const DEVICE_CONFIG: meta::DeviceConfigMeta;
    fn check_signature(&self) -> bool;
    fn static_sha256(&self) -> [u8; 32] {
        Self::CUR_META.struct_sha256
    }
}

/// Safe cell header wrapper.
/// If you create a CellWrapper with new_uninit, you should call ensure_init to handle
pub struct CellWrapper<T>
where T: 'static {
    header: &'static T,
    pub state: HeaderType,
    is_init: AtomicBool,
}

impl<T> core::ops::Deref for CellWrapper<T>
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


impl<T> CellWrapper<T>
where T: Cell + 'static {
    pub const unsafe fn _new_uninit(h: &'static T) -> Self {
        Self {
            header: h,
            state: HeaderType::Actual,
            is_init: AtomicBool::new(false),
        }
    }

    pub unsafe fn _new_init(h: &'static T) -> Option<Self> {
        if !h.check_signature() {
            return None;
        }

        Some(Self {
            header: h,
            state: HeaderType::Actual,
            is_init: AtomicBool::new(true),
        })
    }

    /// If header wrapper was created with new_uninit, this function must be called to potentially initialize other cell's memory.
    pub fn ensure_init(&self) -> Option<()> {
        if !self.header.check_signature() {
            return None;
        }
        self.is_init.store(true, core::sync::atomic::Ordering::Relaxed);
        Some(())
    }

    pub fn new_dummy(dummy_abi: &'static T) -> Self {
        Self {
            header: dummy_abi,
            state: HeaderType::Dummy,
            is_init: AtomicBool::new(true),
        }
    }
}