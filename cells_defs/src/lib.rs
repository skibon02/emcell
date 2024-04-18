#![no_std]

#[macro_use]
extern crate emcell_macro;

emcell_configuration! {
    device!{
        initial_stack_ptr: 0x2000_6000,

        ram_range_start: 0x2000_0000,
        ram_range_end: 0x2003_8000, // specified extended range for at32f407 (224KB). But we must ensure that extended memory was unlocked in headboot cell.

        flash_range_start: 0x0800_0000,
        flash_range_end: 0x0810_0000,
    }

    #[cell(primary)]
    #[ram_region(0x6000, 0x6400)]
    #[flash_region(0x0, 0x1000)]
    pub struct Cell1 {
    }

    #[cell]
    #[ram_region(0x6400, 0x2_8000)] // 95KB
    #[flash_region(0x1000, 0x9_0000)]
    pub struct Cell2 {
        #[switch_vectors]
        pub run: fn() -> !,
        pub a: u32,
        pub print_some_value: fn(u32),
    }

    #[cell]
    #[ram_region(0x2_8000, 0x3_4000)]
    #[flash_region(0x9_0000, 0xF_0000)]
    pub struct Cell3 {
        pub b: u32,
        pub run_some_code: fn(),
        pub access_static: fn() -> u32,
    }
}
