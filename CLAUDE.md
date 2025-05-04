# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build Commands
- Build: `cargo build`
- Run: `cargo run`
- Release build: `cargo build --release`
- Check: `cargo check`
- Lint: `cargo clippy`
- Test: `cargo test` (for all tests)
- Single test: `cargo test test_name` or `cargo test -- --test test_name`

## Code Style Guidelines
- Use Rust 2021 edition conventions
- Naming: `snake_case` for variables/functions, `CamelCase` for types/traits/enums
- Error handling: Use `anyhow::Result` with `?` operator for propagation
- Imports: Group by standard lib, external crates, then internal modules
- State management: Central `App` struct with thread-safe access via `Arc<Mutex<>>`
- Threading: Main thread for events, UI thread for rendering, network thread for async ops
- UI components: Composable widgets in the `widgets/` directory
- Network operations: Async with Tokio runtime, channel-based communication
- Error logging: Use structured logging with `log` macros (`debug`, `info`, `error`)