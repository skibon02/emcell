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
