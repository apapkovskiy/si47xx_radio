build_host:
	@echo "Building host application..."
	cargo build --features="host"

run_host:
	@echo "Running host application..."
	cargo run --features="host"

build_nrf:
	@echo "Building nRF firmware..."
	cargo objcopy --target=thumbv8m.main-none-eabihf --features="nrf5340dk" -- -O ihex target/firmware.hex

flash:
	@echo "Flashing nRF firmware..."
	nrfutil device program --traits=jlink --firmware target/firmware.hex --core=application
	nrfutil device reset --traits=jlink
