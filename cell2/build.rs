use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    emcell::build_rs::<cells_defs::Cell2>();
    
    println!("cargo:rustc-link-arg=-Map=cell2/map-at32.map");
}

