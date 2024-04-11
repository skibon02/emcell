#![no_std]

#[macro_use]
extern crate emcell_macro;

emcell_configuration! {
    #[cell(primary)]
    #[ram_region(0x2000_0000, 0x2000_1000)]
    #[flash_region(0x0800_0000, 0x0800_1000)]
    pub struct Cell1ABI {
        pub a: u32,
        pub print_some_value: fn(u32),
    }

    #[cell]
    #[ram_region(0x2000_0000, 0x2000_1000)]
    #[flash_region(0x0800_0000, 0x0800_1000)]
    pub struct Cell2ABI {
        pub b: u32,
        pub run_some_code: fn(),
    }
}