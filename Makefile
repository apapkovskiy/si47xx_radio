build_host:
	@echo "Building host application..."
	cargo build --features="host" --no-default-features

run_host:
	@echo "Running host application..."
	cargo run --features="host" --no-default-features

build_nrf:
	@echo "Building nRF firmware..."
	cargo objcopy --target=thumbv8m.main-none-eabihf --features="nrf5340dk" -- -O ihex target/firmware.hex

clippy_stack:
	@echo "Running clippy to check for large stack frames"
	cargo clippy --target=thumbv8m.main-none-eabihf --features="nrf5340dk" -- -D warnings -W clippy::large_stack_frames

flash:
	@echo "Flashing nRF firmware..."
	nrfutil device program --traits=jlink --firmware target/firmware.hex --core=application
	nrfutil device reset --traits=jlink
