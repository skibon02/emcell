fn main() {
    let meta = &cells_defs::META;
    let cur_cell_name = "Cell1ABI";
    let cur_cell_meta = cells_defs::meta_for_cell(cur_cell_name).unwrap();
    emcell::build_rs(meta, cur_cell_meta);

    println!("cargo:rustc-link-arg=-Map=map-at32.map");
}

