#![feature(prelude_import)]
#![no_std]
#[prelude_import]
use core::prelude::rust_2021::*;
#[macro_use]
extern crate core;
extern crate compiler_builtins as _;
#[macro_use]
extern crate emcell_macro;
#[repr(C)]
pub struct Cell1 {
    pub signature: u32,
    pub init: unsafe fn([u8; 32]) -> bool,
    pub a: u32,
    pub print_some_value: fn(u32),
}
#[repr(C)]
pub struct Cell2 {
    pub signature: u32,
    pub init: unsafe fn([u8; 32]) -> bool,
    pub b: u32,
    pub run_some_code: fn(),
    pub access_static: fn() -> u32,
}
pub const CELL_NAMES: [&'static str; 2usize] = ["Cell1", "Cell2"];
pub type PrimaryCell = Cell1;
unsafe impl emcell::Cell for Cell2 {
    fn check_signature(&self) -> bool {
        if self.signature != 0xdeadbeef {
            return false;
        }
        let known_sha256 = [
            204u8,
            124u8,
            183u8,
            146u8,
            72u8,
            34u8,
            134u8,
            179u8,
            231u8,
            3u8,
            70u8,
            51u8,
            109u8,
            169u8,
            130u8,
            214u8,
            71u8,
            162u8,
            245u8,
            17u8,
            134u8,
            240u8,
            241u8,
            100u8,
            50u8,
            7u8,
            123u8,
            232u8,
            11u8,
            196u8,
            75u8,
            204u8,
        ];
        let sha_ok = unsafe { (self.init)(known_sha256) };
        return sha_ok;
    }
}
impl Cell2 {
    pub const static_sha256: [u8; 32] = [
        204u8,
        124u8,
        183u8,
        146u8,
        72u8,
        34u8,
        134u8,
        179u8,
        231u8,
        3u8,
        70u8,
        51u8,
        109u8,
        169u8,
        130u8,
        214u8,
        71u8,
        162u8,
        245u8,
        17u8,
        134u8,
        240u8,
        241u8,
        100u8,
        50u8,
        7u8,
        123u8,
        232u8,
        11u8,
        196u8,
        75u8,
        204u8,
    ];
}
unsafe impl emcell::Cell for Cell1 {
    fn check_signature(&self) -> bool {
        if self.signature != 0xbeefdead {
            return false;
        }
        let known_sha256 = [
            183u8,
            199u8,
            180u8,
            177u8,
            51u8,
            226u8,
            238u8,
            188u8,
            168u8,
            97u8,
            100u8,
            83u8,
            247u8,
            231u8,
            54u8,
            199u8,
            87u8,
            235u8,
            39u8,
            51u8,
            148u8,
            234u8,
            209u8,
            194u8,
            73u8,
            217u8,
            62u8,
            47u8,
            195u8,
            15u8,
            191u8,
            74u8,
        ];
        let sha_ok = unsafe { (self.init)(known_sha256) };
        return sha_ok;
    }
}
impl Cell1 {
    pub const static_sha256: [u8; 32] = [
        183u8,
        199u8,
        180u8,
        177u8,
        51u8,
        226u8,
        238u8,
        188u8,
        168u8,
        97u8,
        100u8,
        83u8,
        247u8,
        231u8,
        54u8,
        199u8,
        87u8,
        235u8,
        39u8,
        51u8,
        148u8,
        234u8,
        209u8,
        194u8,
        73u8,
        217u8,
        62u8,
        47u8,
        195u8,
        15u8,
        191u8,
        74u8,
    ];
}
pub const META: emcell::meta::CellDefsMeta<2usize> = emcell::meta::CellDefsMeta {
    cell_defs: [
        emcell::meta::CellDefMeta {
            name: "Cell1",
            cell_type: emcell::CellType::Primary,
            ram_range_start_offs: 24576usize,
            ram_range_end_offs: 57344usize,
            flash_range_start_offs: 0usize,
            flash_range_end_offs: 524288usize,
            struct_sha256: [
                183u8,
                199u8,
                180u8,
                177u8,
                51u8,
                226u8,
                238u8,
                188u8,
                168u8,
                97u8,
                100u8,
                83u8,
                247u8,
                231u8,
                54u8,
                199u8,
                87u8,
                235u8,
                39u8,
                51u8,
                148u8,
                234u8,
                209u8,
                194u8,
                73u8,
                217u8,
                62u8,
                47u8,
                195u8,
                15u8,
                191u8,
                74u8,
            ],
        },
        emcell::meta::CellDefMeta {
            name: "Cell2",
            cell_type: emcell::CellType::NonPrimary,
            ram_range_start_offs: 196608usize,
            ram_range_end_offs: 229376usize,
            flash_range_start_offs: 524288usize,
            flash_range_end_offs: 1048576usize,
            struct_sha256: [
                204u8,
                124u8,
                183u8,
                146u8,
                72u8,
                34u8,
                134u8,
                179u8,
                231u8,
                3u8,
                70u8,
                51u8,
                109u8,
                169u8,
                130u8,
                214u8,
                71u8,
                162u8,
                245u8,
                17u8,
                134u8,
                240u8,
                241u8,
                100u8,
                50u8,
                7u8,
                123u8,
                232u8,
                11u8,
                196u8,
                75u8,
                204u8,
            ],
        },
    ],
    device_configuration: emcell::meta::DeviceConfigMeta {
        initial_stack_ptr: 536895488usize,
        ram_range_start: 536870912usize,
        ram_range_end: 537100288usize,
        flash_range_start: 134217728usize,
        flash_range_end: 135266304usize,
    },
};
