#![no_std]

#[cfg(feature = "build-rs")]
mod build_rs;
#[cfg(feature = "build-rs")]
pub use build_rs::*;


pub mod meta;

#[cfg(not(feature = "build-rs"))]
pub mod device;

pub enum CellType {
    Primary,
    NonPrimary
}

pub enum HeaderState {
    Uninit,
    Init,
    Dummy
}

pub unsafe trait Cell {
    fn check_signature(&self) -> bool;
}

pub struct CellWrapper<T>
where T: 'static {
    header: &'static T,
    state: HeaderState
}

impl<T> core::ops::Deref for CellWrapper<T>
    where T: Cell + 'static {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.header
    }
}


impl<T> CellWrapper<T>
where T: Cell + 'static {
    pub unsafe fn _new_uninit(h: &'static T) -> Self {
        Self {
            header: unsafe { h },
            state: HeaderState::Uninit
        }
    }

    pub unsafe fn _new_init(h: &'static T) -> Option<Self> {
        if !h.check_signature() {
            return None;
        }

        Some(Self {
            header: unsafe { h },
            state: HeaderState::Init
        })
    }

    pub fn new_dummy(dummy_abi: &'static T) -> Self {
        Self {
            header: dummy_abi,
            state: HeaderState::Dummy
        }
    }
}