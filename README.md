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

## Extra flash regions

By default, each cell places all of its code and data (`.text`, `.rodata`, `.data`
LMA) into the single flash region declared with `#[flash_region(start, end)]`.
Sometimes you want to move a few big, rarely-used functions into a *separate*
flash region that is not contiguous with the main one (e.g. a slower, larger
flash bank), leaving a gap so the regions can be flashed independently.

### Declaring an extra region

Add one or more repeatable `#[extra_flash(...)]` attributes to a cell definition:

```rust
#[cell]
#[ram_region(0x6400, 0xA000)]
#[flash_region(0x0_4000, 0xF_1000)]
#[extra_flash(name = "Slow", section = ".slow_text", 0xE_0000, 0xF_0000)]
pub struct Cell2 {
    // ...
}
```

The attribute takes four values:

- `name` — a stable identifier used to derive the linker memory region name
  (e.g. `"Slow"` produces a `SLOW_FLASH` memory region).
- `section` — the linker input section name that `#[place]`-marked items are
  put into.
- two integer offsets — the flash `start`/`end`, relative to
  `flash_range_start` (the same convention as `#[flash_region]`).

### Placing items into an extra region

Mark items with the `#[place("...")]` attribute macro, passing the section name:

```rust
#[place(".slow_text")]
pub fn rarely_used() {
    // ...
}
```

`#[place(".slow_text")]` expands to `#[link_section = ".slow_text"]`.

### Important: section name must not collide with `.text.*` / `.rodata.*`

`cortex-m-rt`'s `link.x` places code with `*(.text .text.*)` and rodata with
`*(.rodata .rodata.*)`. If an extra region's section name matches `.text.*` or
`.rodata.*` (e.g. `.text.slow`), the linker will greedily pull the item into the
main `FLASH` region and it will never reach the extra region. Use a distinct
prefix such as `.slow_text` (NOT `.text.slow`).
