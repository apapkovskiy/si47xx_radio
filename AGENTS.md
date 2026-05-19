# AGENTS.md

## Scope
- These instructions apply to the entire `si47xx_radio` repository.

## Project Overview
- Rust firmware + host helper for Si47xx radio control.
- Primary embedded target: `thumbv8m.main-none-eabihf`.
- Default board feature: `nrf5340dk` (see `Cargo.toml`).

## Local Skills

- Added local skill: `.agents/skills/pullrequest/SKILL.md`.
- Purpose: open GitHub PRs with consistent flow (`git fetch origin`, base branch detect `master`/`main`, commit list against base, minimal PR body with Summary + Commits, `gh pr create`).
- Helper script: `.agents/skills/pullrequest/scripts/commits_for_pr.sh` to generate commit-title bullets from `origin/<base>..HEAD`.

## Working Principles
- Prefer the smallest correct change; preserve existing architecture and async/event-driven patterns.
- Keep board-specific behavior behind existing feature flags (`nrf5340dk`, `nrf7002dk`, `host`).
- Do not introduce breaking CLI command changes unless explicitly requested.

## Build, Lint, and Verification
- Use repository-defined commands; do not invent alternatives when these apply.

- Host build:
  - `make build_host`
- Host run:
  - `make run_host`

- Embedded firmware build + artifacts + memory usage:
  - `make build_nrf`

- Embedded clippy (stack-aware):
  - `make clippy_stack`

- Pre-commit checks configured in `.pre-commit-config.yaml`:
  - `cargo fmt -- --check`
  - `cargo clippy --features "nrf7002dk" --target=thumbv8m.main-none-eabihf --no-default-features -- -D warnings -W clippy::large_stack_frames`
  - `cargo clippy --features "nrf5340dk" --target=thumbv8m.main-none-eabihf --no-default-features -- -D warnings -W clippy::large_stack_frames`

- Preferred validation flow for code changes:
  1. `cargo fmt -- --check`
  2. Relevant clippy command(s) for touched target(s)
  3. `make build_host` for host-only changes, or `make build_nrf` for embedded changes

## Flashing / Hardware Notes
- `make flash` uses `nrfutil` + J-Link traits and expects `target/firmware.hex`.
- If changing board/chip assumptions, update docs and build/flash instructions in the same change.

## Code Style Expectations
- Follow rustfmt defaults and keep clippy warning-free under configured flags.
- Avoid unnecessary heap allocations; prefer existing embedded-friendly patterns (`heapless`, static buffers, channels).
- Keep logging concise and useful for UART diagnostics.

## When Updating CLI / Events / Radio Control
- Keep command parsing, event emission, and radio side effects clearly separated:
  - CLI command modules under `src/cli/`
  - Event types in `src/events.rs`
  - Radio hardware behavior in `src/radio.rs` and submodules
- For new user-facing commands, ensure matching `SystemEvent` and (if needed) `SystemNotify` paths are implemented.

## Safety and Non-Goals
- Never commit secrets, keys, probe credentials, or personal machine config.
- Do not run destructive git commands (`reset --hard`, checkout discard) unless explicitly requested.
