build_host:
	@echo "Building host application..."
	cargo build --features="host" --no-default-features

run_host:
	@echo "Running host application..."
	cargo run --features="host" --no-default-features

build_nrf:
	@echo "Building nRF firmware..."
	cargo objcopy --target=thumbv8m.main-none-eabihf --features="nrf5340dk" -- -O ihex target/firmware.hex
	@echo "Flash and RAM usage:"
	@cargo bloat --target=thumbv8m.main-none-eabihf --features="nrf5340dk" --bin si47x_radio --embedded --split-std --crates
	@cargo size --target=thumbv8m.main-none-eabihf --features="nrf5340dk" --bin si47x_radio -- -A
	@cargo size --target=thumbv8m.main-none-eabihf --features="nrf5340dk" --bin si47x_radio | awk 'NR==2 {sumFlash=$$1} {sumRam=$$2+$$3} END {printf "Flash Usage: %.2d, %.2f%%, RAM Usage: %.2d, %.2f%%\n", sumFlash, (sumFlash/1048576)*100, sumRam, (sumRam/262144)*100}'

clippy_stack:
	@echo "Running clippy to check for large stack frames"
	cargo clippy --target=thumbv8m.main-none-eabihf --features="nrf5340dk" -- -D warnings -W clippy::large_stack_frames

flash:
	@echo "Flashing nRF firmware..."
	nrfutil device program --traits=jlink --firmware target/firmware.hex --core=application
	nrfutil device reset --traits=jlink
