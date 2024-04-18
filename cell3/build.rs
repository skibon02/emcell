
fn main() {
    emcell::build_rs::<cells_defs::Cell3>();
    
    println!("cargo:rustc-link-arg=-Map=cell3/map-at32.map");
}

