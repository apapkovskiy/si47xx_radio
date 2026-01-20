# si47x_radio

Firmware for controlling Si47xx FM/AM radio tuners from an nRF5340 using the Embassy async ecosystem. It provides a small UART CLI and status logging while driving the Si47xx over I2C.

## Hardware targets
- MCU: Nordic nRF5340 (`nRF5340_xxAA` by default in `.cargo/config.toml`)
- UART0 at 115200 8N1 for CLI: `P0_22` (TX) and `P0_20` (RX)
- I2C (TWIM1): `P1_14` (SCL) and `P1_13` (SDA)
- Si47xx reset pin: `P1_03`
- Status LED: `P0_28`

Adjust pin mappings in `src/main.rs` if your board is wired differently.

## Build and flash
1. Install the target and probe support:
   ```bash
   rustup target add thumbv8m.main-none-eabihf
   cargo install probe-rs-tools  # if probe-rs is not installed
   ```
2. Connect your probe and board, then run:
   ```bash
   cargo run --release
   ```
   The runner in `.cargo/config.toml` uses `probe-rs run --chip nRF5340_xxAA`. Change the chip value if you use another device.

## Using the CLI
Open a serial terminal on UART0 at 115200 baud. Commands available:
- `status` — print basic system status.
- `mode fm|am|off` — switch radio mode or power down.
- `volume up|down|set <0-100>` — adjust audio level.
- `tune up|down|frequency <MHz>` — seek up or set a specific frequency (down is currently a placeholder).

CLI echoes feedback and emits events handled in `src/main.rs` by the async Embassy tasks.

## Logging
Logs are written over the same UART via the `log` facade. You will see initialization messages, tune results, and event traces alongside CLI output.

## Licensing
Dual-licensed under MIT and Apache-2.0. You may use either license at your option.
- See `LICENSE-MIT` for the MIT license text.
- See `LICENSE-APACHE` for the Apache 2.0 license text.
