#![no_std]

use core::ops::Deref;

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

impl<T> Deref for CellWrapper<T>
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

pub enum CellType {
    Primary,
    NonPrimary
}
pub struct CellDefMeta {
    pub name: &'static str,
    pub cell_type: CellType,

    pub ram_range_start: usize,
    pub ram_range_end: usize,

    pub flash_range_start: usize,
    pub flash_range_end: usize,
}

pub struct CellDefsMeta<const N: usize> {
    pub cell_defs: [CellDefMeta; N]
}

#[cfg(not(feature = "build-rs"))]
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

#[cfg(all(feature = "build-rs", not(feature = "rt-crate-cortex-m-rt")))]
compile_error!("This crate requires any rt-crate-* to be enabled (when using build-rs feature)! *currently only rt-crate-cortex-m-rt is supported*");


#[cfg(feature = "build-rs")]
extern crate std;

#[cfg(all(feature = "build-rs", feature = "rt-crate-cortex-m-rt"))]
pub fn build_rs<const N: usize>(cells_defs_meta: &'static CellDefsMeta<N>, cur_cell: &'static CellDefMeta) {
    use std::env;
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;
    use std::string::String;

    let out_dir = &PathBuf::from(env::var("OUT_DIR").unwrap());

    let cur_cell_name = cur_cell.name;

    let memory_x_data = match cur_cell_name {
        "Cell1ABI" => String::from(r#"MEMORY
{
  FLASH : ORIGIN = 0x08000000, LENGTH = 510K
  CELL2_FLASH : ORIGIN = 0x08080000, LENGTH = 510K

  /* ABI */
  CELL1_ABI : ORIGIN = 0x0807FC00, LENGTH = 1K
  CELL2_ABI : ORIGIN = 0x080FFC00, LENGTH = 1K

  /* RAM */
  /* <--- leave 24K for the stack */
  RAM : ORIGIN = 0x20006000, LENGTH = 32K
  CELL2_RAM : ORIGIN = 0x2000e000, LENGTH = 40K
}

_stack_start = ORIGIN(RAM);

SECTIONS {
    .CELL1_ABI ORIGIN(CELL1_ABI) : {
        . = ALIGN(4);
        KEEP(*(.emcell.CELL1ABI));
        . = ALIGN(4);
    } > CELL1_ABI

    .CELL2_ABI ORIGIN(CELL2_ABI): {
        _emcell_Cell2ABI_internal = .;
    } > CELL2_ABI
}"#),
    "Cell2ABI" => String::from(r#"MEMORY
{
  CELL1_FLASH : ORIGIN = 0x08000000, LENGTH = 510K
  FLASH : ORIGIN = 0x08080000, LENGTH = 510K

  /* ABI */
  CELL1_ABI : ORIGIN = 0x0807FC00, LENGTH = 1K
  CELL2_ABI : ORIGIN = 0x080FFC00, LENGTH = 1K

  /* RAM */
  /* <--- leave 24K for the stack */
  CELL1_RAM : ORIGIN = 0x20006000, LENGTH = 32K
  RAM : ORIGIN = 0x2000e000, LENGTH = 40K
}

_stack_start = ORIGIN(CELL1_RAM);

SECTIONS {
    .CELL2_ABI ORIGIN(CELL2_ABI) : {
        . = ALIGN(4);
        KEEP(*(.emcell.CELL2ABI));
        . = ALIGN(4);
    } > CELL2_ABI

    .CELL1_ABI ORIGIN(CELL1_ABI): {
        _emcell_Cell1ABI_internal = .;
    } > CELL1_ABI
}"#),
        _ => panic!("Unknown cell name")
    };

    let mut f = File::create(out_dir.join("memory.x")).unwrap();
    f.write_all(memory_x_data.as_bytes()).unwrap();

    std::println!("cargo:rustc-link-search={}", out_dir.display());
}