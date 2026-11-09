# Super-sNES

SNES emulator in Rust (crate `ssnes`). CPU is the 65C816 (`src/cpu`), memory bus in `src/memory/bus.rs`.

## Rules

1. Write the simplest code possible, but keep it readable.
2. Commit every feature as soon as it is done (small commits, conventional messages: `feat:`, `fix:`, `test:`, `docs:`).
3. Never add Claude (or any AI) as co-author of a commit. No `Co-Authored-By` lines, no AI attribution in commit messages.
4. Never push, merge into `main`, or open PRs. Work stays on the local feature branch; the human pushes and merges.

## Workflow

- Develop on the feature branch for the area (CPU work: `cpucreation`).
- Run `cargo test` before every commit; it must pass.
- Tests live next to the code as `*_tests.rs` modules (see `src/cpu/addressing_modes_tests.rs`).
