# Super-sNES

SNES emulator in Rust (crate `ssnes`). CPU is the 65C816 (`src/cpu`), memory bus in `src/memory/bus.rs`.

## Rules

1. Write the simplest code possible, but keep it readable.
2. Commit every feature as soon as it is done (small commits, conventional messages: `feat:`, `fix:`, `test:`, `docs:`).
3. Never add Claude (or any AI) as co-author of a commit. No `Co-Authored-By` lines, no AI attribution in commit messages.
4. Never push, merge into `main`, or open PRs. Work stays on the local feature branch; the human pushes and merges.
5. Stay on the main task you were given. Do not work on features or fixes that were not asked for.

## Workflow

- Develop on the feature branch for the area (CPU work: `cpucreation`).
- Run `cargo test` before every commit; it must pass.
- Tests live next to the code as `*_tests.rs` modules (see `src/cpu/addressing_modes_tests.rs`).

## Code standard

Every module follows these rules. New code must match them. When you change old code, bring the parts you touch up to them.

1. **One concept per file.** The file name names the concept: `status.rs` holds `StatusRegister`, `alu.rs` holds add, subtract, compare and shift.
2. **`mod.rs` only declares modules.** No types, functions or logic in it.
3. **Keep files under about 300 lines.** Past that, split by concept. Test files are exempt.
4. **Plain names.** Spell words out (`instruction`, `wide`, `index_x`). Keep the 65C816 register letters (A, X, Y, S, P, D, DB, PB, PC) because the datasheet uses them.
5. **Explicit imports.** Use `use x::*` only to match on an enum's variants.
6. **Plain control flow.** No closures, traits, generics or wrapper types that only hide code. Add an abstraction once two real callers need it.
7. **Comments say why.** Give each public type and each non-obvious public function a `///` line. Use `//` only for a hardware quirk or a non-obvious reason. Mark a known simplification `// shortcut: <the limit>, <when to upgrade>`.
8. **Tests sit beside the code** as `<name>_tests.rs`, declared in the parent `mod.rs` under `#[cfg(test)]`.
9. **No dead code.** Delete unused functions, fields, files and modules.

### CPU layout (`src/cpu`)

| File | Holds |
|---|---|
| `cpu.rs` | `Cpu`: reset, step, interrupts, register writes |
| `execute.rs` | the instruction `match`: what each instruction does |
| `implied.rs` | instructions that touch only registers and flags |
| `stack.rs` | push and pull instructions, PEA, PEI, PER |
| `alu.rs` | add, subtract, compare, shifts, and the width masks |
| `memory.rs` | fetch, read, write, push and pull through the bus |
| `addressing_modes.rs` | the `AddressingMode` enum and `EffectiveAddress` |
| `resolve.rs` | turns each addressing mode into an address |
| `registers.rs` | `Registers`: A, X, Y, D, DB, PB, PC, S, E |
| `status.rs` | `StatusRegister`: the P flags |
| `instructions.rs` | `Instruction`: one variant per mnemonic |
| `opcodes.rs` | `decode`: opcode byte to instruction, mode and base cycles |
| `cpu_tests.rs`, `addressing_modes_tests.rs` | unit tests for the CPU and the addressing modes |
