cargo run --release &&
probe-rs attach --chip AT32F437VMT7 --probe 1366:0105 ../target/thumbv7em-none-eabihf/release/cell2 --no-location
