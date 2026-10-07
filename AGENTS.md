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
RUSTFLAGS="-C llvm-args=-align-all-functions=6" \
  cargo bench --bench versions --no-default-features --features deterministic_iteration
                                        # current sets vs the previous release (accurate)
cargo bench --bench libraries           # tinyset vs HashSet, roaring, id-set (rough)
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

## Benchmarks

`benches/versions.rs` and `benches/libraries.rs` use the attribute API of `scaling`, a git dependency on its `laps` branch (MSRV 1.81, so benchmarks need a newer toolchain than the crate's MSRV; that branch is an open pull request, so expect its API to move). Set fixtures come from a small pool of pre-built sets shared via `Rc`, because the harness makes and clones one input per iteration per candidate. The `collect_*` and `insert_all_*` candidates return the set they built, and a `#[scaling::metrics(.., allocation)]` function per set type (metrics pair by return type) reports its net bytes, peak bytes and allocation count (`scaling::Allocator` is the global allocator). Each cell is a single run on one random input.

The two files split the comparisons by how accurate they need to be, and each has a custom `main` (`scaling::main!()` can't take a `Config`) that sets the precision, which `TINYSET_BENCH_PRECISION` (a fraction, such as `0.05`) overrides. With no way to filter from the command line, running one bench is how to run a subset.

- `libraries.rs` compares tinyset (`setu32`, `setu64`, `set64`) with `std32`/`std64`, `roaring` and `idset`, with `std64` as the baseline and every other candidate marked `uninteresting` (measured only to a 20% rough error). Most of these differ by several times. Build it normally.
- `versions.rs` is for checking a change against the previous release. It has one group per operation, density and type (`contains_sparse_setu64`, ...) with two candidates, the previous release's type (`old_*`, the baseline) and the current one, so each comparison is the direct old-versus-new pair, at 1% relative precision. The `old_` types come from `tinyset_old`, a renamed dev-dependency on `tinyset =0.5.4`; bump that pin when a release is made. Don't compare against numbers from an earlier run: `scaling` deliberately has no way to save results.

Build `versions` as in the commands above (it refuses to compile otherwise). Two things make its old-versus-current gaps meaningless without it:

1. **Function alignment.** Identical source compiled as a separate crate, or even as another module of the same crate, differs by up to 75% on short functions, purely from where the compiler puts function entry points. `-C llvm-args=-align-all-functions=6` removes nearly all of it: 138 of 432 identical-source cells were significant without it and 13 with it, and the same-crate control went from 24 to 0. After it, gaps below about 10% between the old and current types are still unresolved (the separate-crate floor: a smoke run of identical code showed 24 of 432 cells changed, the largest 12.4%), so don't read changes smaller than that. Block alignment on top of it (`-align-all-nofallthru-blocks`) made it worse.
2. **Deterministic table layout.** tinyset picks a random capacity when a table grows, so two types holding the same elements can sit in differently sized tables. `deterministic_iteration` makes the choice depend only on the history of inserts and removes, which the old and current types share; `tinyset_old` is built with it too. Without it the same elements give different layouts in the two types.

The construction cells (`collect`, `insert_all`) are the most reliable: they report net bytes, peak bytes and allocation count, which are exact.

Run benchmarks on a quiet machine, one at a time, and keep a memory cap on `scaling` runs (`systemd-run --user --scope -p MemoryMax=4G -p MemorySwapMax=0`). For comparisons that must be rigorous, reserve a CPU and run the binary with `quiet-bench run <binary> --bench`. On a quiet CPU 2 `versions` took about 2 minutes at 5% precision and `libraries` about 1 minute; `versions` is now set to 1%, which takes longer.

## Tests

Unit tests live inline in each module as `#[cfg(test)]` blocks, and many are proptest-based. Proptest failure cases are saved in `proptest-regressions/`; commit new regression files alongside fixes. `tests/` contains regression tests for specific GitHub issues.

## Releases

Use the `release` skill (`.claude/skills/`). It covers the CHANGELOG entry format, the version bump, tagging (bare `X.Y.Z`, no `v` prefix), and `cargo publish`.
