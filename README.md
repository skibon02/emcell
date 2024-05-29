# emcell

**emcell** (**EM**bedded **CELL**) - is a library, which make it very easy
to keep several binaries on a single microcontroller. You just need to create
a separate crate with *cells* definitions and create simple `build.rs`, specifying which *cell* 
is a current crate for.

*Cell* is an abstract word for a binary or library, which have a specific region of 
FLASH and RAM memory assigned. You can keep several *cells* on a single microcontroller,
and even define header for each of them to cross-call different functions.

*Emcell* also allow you to define a special function with signature `fn() -> !`, which will
perform vector table switch. For example, you can define function `run() -> !` in your main code,
and run it from bootloader cell.

## Usage

Example of `lib.rs` for cells definitions crate:

```rust
#![no_std]

#[macro_use]
extern crate emcell_macro;

emcell_configuration! {
    device!{
        initial_stack_ptr: 0x2000_6000,

        ram_range_start: 0x2000_0000,
        ram_range_end: 0x2001_8000, // 96Kb RAM

        flash_range_start: 0x0800_0000,
        flash_range_end: 0x0810_0000, // 1Mb flash
    }

    #[cell(primary)]
    #[ram_region(0x6000, 0x6400)]
    #[flash_region(0x0, 0x4000)]
    pub struct Cell1 {
    }

    #[cell]
    #[ram_region(0x6400, 0xA000)]
    #[flash_region(0x0_4000, 0xF_1000)]
    pub struct Cell2 {
        #[switch_vectors]
        pub run: fn() -> !,
        pub a: u32,
        pub print_some_value: fn(u32),
    }

    #[cell]
    #[ram_region(0xA000, 0x1_0000)]
    #[flash_region(0xF_1000, 0x10_0000)]
    pub struct Cell3 {
        pub b: u32,
        pub run_some_code: fn(),
        pub access_static: fn() -> u32,
    }
}
```

In this example you define 3 cells: `Cell1`, `Cell2` and `Cell3`. Each of them have
separate crate and can call functions from other crates using safe wrapper, which is created automatically
 with macro.

Example of `main.rs` for cell1 crate:

```rust
#![no_std]
#![no_main]

#![feature(const_refs_to_static)]
use emcell_macro::{define_primary_header, extern_header_forward};
use cells_defs::{Cell1, Cell2};
use cortex_m::asm::delay;

extern crate panic_halt;

define_primary_header!{
    Cell1 {
    }
}

extern_header_forward!(Cell2Wrapper: Cell2);

#[cortex_m_rt::entry]
unsafe fn main() -> ! {
    gpio_cfgr();
    led_on();

    if let Some(cell2) = Cell2Wrapper::new() {
        cell2.switch_vectors_and_run() // execute run() -> ! for cell2
    }
    else {
        loop {
            delay(1_000_000);
        }
    }
}
```

build.rs:
```rust
fn main() {
    emcell::build_rs::<cells_defs::Cell1>();
}
```
`Cell2Wrapper::new()` is created automatically and perform additional checks to ensure, that header for cell2 
was not modified (by comparing hash) and is compatible with current crate.

## Nightly toolchain
Currently, emcell requires nightly because of `const_refs_to_static` feature. 
You can use `rustup override set nightly` to set nightly for the current directory.