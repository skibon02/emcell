
fn main() {
    emcell::build_rs::<cells_defs::Cell2>();
    
    println!("cargo:rustc-link-arg=-Map=example/cell2/map-at32.map");
}

