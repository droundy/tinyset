## What this is

`tinyset` is a single Rust library crate of size-optimized integer sets (`SetU64`, `SetU32`, `SetUsize`, `Set64<T: Fits64>`). Small sets fit in one tagged pointer with no heap allocation; larger ones use compact heap formats. The code is heavily `unsafe` (raw pointers, manual allocation, pointer tagging), so correctness changes should be checked under Miri.

## Commands

```bash
cargo build
cargo test                              # default features (includes `rand`)
cargo test --no-default-features        # CI also runs this; exercises the built-in PRNG in src/rand.rs
cargo test check_sets                   # run tests matching a name
cargo test --lib setu32::               # run one module's tests
cargo check --features compactserde     # serde / compactserde are not exercised by default
cargo check --target wasm32-unknown-unknown --no-default-features
cargo +nightly miri test check_sets     # CI runs Miri on check_sets and check_specific_sets only
cargo bench                             # timing + memory comparisons vs HashSet, BTreeSet, roaring, id-set
```

MSRV is set by `rust-version` in `Cargo.toml` (1.65). Don't use newer language/std features without bumping it. CI (`.github/workflows/rust.yml`) checks the MSRV toolchain on Linux, wasm, Windows, and macOS; keep its version lists in sync with `rust-version`.

## Features

- `rand` (default): randomizes hash-table reallocation to mitigate collision DoS attacks.
- Without `rand`, `src/rand.rs` falls back to a SplitMix64 PRNG seeded from the system time.
- `deterministic_iteration`: can't be combined with `rand` (enforced by `compile_error!`), so it needs `--no-default-features`. Iteration order then depends only on the history of inserts and removes.
- `serde`: serializes sets as plain element lists.
- `compactserde`: serializes the raw in-memory representation. The format is unstable and loading corrupt input can cause undefined behavior.

## Architecture

- **`src/setu64.rs`**: the core implementation; the doc comment on `SetU64` describes the design in detail. `SetU64(*mut S)` is a tagged pointer:
  - null: empty set.
  - low 3 bits nonzero: a "stack" set (`Tiny`). The low 3 bits hold the element count (up to 7), and the remaining bits hold the smallest element followed by deltas, split according to `BITSPLITS`.
  - otherwise: a pointer to a heap `S` (header `Sbeginning { sz, cap, bits }` followed by a `u64` array). The `bits` field selects the heap format: `Dense` bitmap (`bits == 64`), `Heap` Robin Hood map from high bits to bitmaps (`0 < bits < 64`), or `Big` plain Robin Hood set (`bits == 0` or `> 64`).
  - Everything dispatches through `internal()` / `internal_mut()`, which decode the pointer into the `Internal` enum.
- **`src/setu32.rs`**: a near-copy of `setu64.rs` adapted to `u32`. It uses a 2-bit tag (`& 3`) instead of 3 bits. Bug fixes in one file usually need mirroring in the other; several past bugs came from the two drifting apart (e.g. mismatched tag checks).
- **`src/setusize.rs`**: a newtype over `SetU64` or `SetU32`, chosen by `target_pointer_width`.
- **`src/set64.rs`**: `Set64<T>` wraps `SetU64` and converts via the `Fits64` trait (`to_u64` / unsafe `from_u64`). Implementations for integer types and `char` come from the `define_fits!` / `define_ifits!` macros.
- **`src/copyset.rs`**: the `impl_set_methods!` macro, which supplies `PartialEq`, `Debug`, and set operators (`-`, `|`, `&`, …) for each concrete set type. It also defines the test-only `CopySet` trait, which lets the proptest suites compare every set type against `HashSet`/`BTreeSet`.
- **`src/sets.rs`**: the `generic_set!` macro for methods shared by all types (e.g. `is_empty`).
- `#![deny(missing_docs)]` is on, so every public item needs a doc comment.

**Not compiled:** `src/setu32b.rs` and `david_allocator/` aren't referenced by `lib.rs` or `Cargo.toml`. They're historical leftovers; don't edit them expecting any effect.

## Tests

Unit tests live inline in each module as `#[cfg(test)]` blocks, and many are proptest-based. Proptest failure cases are saved in `proptest-regressions/`; commit new regression files alongside fixes. `tests/` contains regression tests for specific GitHub issues.

## Releases

Use the `release` skill (`.claude/skills/`). It covers the CHANGELOG entry format, the version bump, tagging (bare `X.Y.Z`, no `v` prefix), and `cargo publish`.
