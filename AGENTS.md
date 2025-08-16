# Repository Guidelines

## Project Structure & Module Organization
- `Cargo.toml`: Rust crate metadata and dependencies (`serde`, `serde_json`).
- `src/main.rs`: Single-binary CLI (`otl`) that parses Sidekick Plus `.OTL` files and prints JSON, text, or canonical dumps.
- `watch-otl.sh`: Dev helper to watch `.OTL` files, show zero-context diffs of `--canon`, and archive snapshots to `.otl-archive/`.
- `target/`: Build artifacts (ignored by Git). Temporary `.otl-watch/` and `.otl-archive/` may appear next to watched files.
- `README.md`: Background and roadmap. No separate library modules yet.

## Build, Test, and Development Commands
- Makefile (preferred):
  - Build: `make build` (use `RELEASE=0` for debug). Binary path: `make binpath`.
  - Run: `make run ARGS='--canon path/to/file.OTL'`
  - JSON: `make json FILE=path/to/file.OTL`
  - Canon diff: `make diff PREV=prev.OTL CURR=curr.OTL CURSOR=1`
  - Watch: `make watch TARGET=<file|dir> ARGS='--validate'` (needs `inotifywait`, `diff`, `awk`).
  - Hygiene: `make check` (fmt + clippy + test). See all: `make help`.
- Cargo (alternative): `cargo build --release`, `cargo run -- --canon file.OTL`, `cargo fmt --all`, `cargo clippy -- -D warnings`, `cargo test`.

## Coding Style & Naming Conventions
- Rust 2021 edition; idiomatic Rust.
- Format with `rustfmt` before pushing; keep warnings at zero with `clippy`.
- Naming: types `CamelCase`, functions/vars `snake_case`, modules `lowercase`.
- Keep `main.rs` focused on CLI/IO; prefer small pure helpers for parsing/rendering.

## Testing Guidelines
- Current status: no tests checked in. Add unit tests under `#[cfg(test)]` in `src/main.rs` or create `tests/` for integration.
- Suggested: golden tests for `--canon` output and JSON structure. Run with `cargo test`.
- Keep tests deterministic (normalize CRLF to LF as the app does).

## Commit & Pull Request Guidelines
- Commits: concise, imperative present. Examples: "improve --canon output", "recognize A_HASKIDS", "add --text option". Limit subject to ~72 chars; add a body if rationale is non-obvious.
- PRs: include a clear description, sample input and before/after snippets (e.g., `--canon` diff), flags used (`--validate`, `--show-cursor`, etc.), and any linked issues.
- CI hygiene: run `cargo fmt`, `cargo clippy`, and `cargo test` locally before opening.

## Security & Configuration Tips
- The tool reads untrusted `.OTL`; bounds checks exist, but avoid committing sample binaries with sensitive data.
- `watch-otl.sh` env: `OTL_ARCHDIR` to set archive location; `ALWAYS_ARCHIVE=1` to archive even when canonical is unchanged.
