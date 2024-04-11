#![no_std]

#[macro_use]
extern crate emcell_macro;

emcell_configuration! {
    device!{
        initial_stack_ptr: 0x2000_6000,

        ram_range_start: 0x2000_0000,
        ram_range_end: 0x2002_0000,

        flash_range_start: 0x0800_0000,
        flash_range_end: 0x0810_0000,
    }

    #[cell(primary)]
    #[ram_region(0x6000, 0xe000)]
    #[flash_region(0x0, 0x8_0000)]
    pub struct Cell1 {
        pub a: u32,
        pub print_some_value: fn(u32),
    }

    #[cell]
    #[ram_region(0x1_8000, 0x2_0000)]
    #[flash_region(0x8_0000, 0x10_0000)]
    pub struct Cell2 {
        pub b: u32,
        pub run_some_code: fn(),
    }
}