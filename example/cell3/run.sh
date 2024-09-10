cargo run --release &&
probe-rs attach --chip STM32F446RETx --probe 1366:0105 ../target/thumbv7em-none-eabihf/release/cell2 --no-location
