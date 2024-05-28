fn main() {
    emcell::build_rs::<cells_defs::Cell1>();

    println!("cargo:rustc-link-arg=-Map=example/cell1/map-at32.map");
}

 