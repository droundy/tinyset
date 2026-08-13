# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

`tinyset` is a Rust library of size-optimized set collections for integer-like types. The whole point is memory density: small sets of small integers live entirely in a single tagged pointer (no heap allocation), and larger sets pick among several packed heap encodings depending on the value distribution. All operations must preserve hashmap-like time scaling while minimizing bytes per element.

## Commands

- Build: `cargo build`
- Test (default features): `cargo test`
- Run a single test: `cargo test <test_name>` (e.g. `cargo test test_log_2`)
- Test with no default features (uses the built-in time-seeded PRNG instead of `rand`): `cargo test --no-default-features`
- Test the deterministic iteration feature (mutually exclusive with `rand`): `cargo test --no-default-features --features deterministic_iteration`
- Test serde encodings: `cargo test --features serde` and `cargo test --features compactserde`
- Benchmarks (prints timings and per-element storage across set types): `cargo bench`
- Docs: `cargo doc --open`

MSRV is 1.65 (`rust-version` in Cargo.toml). `#![deny(missing_docs)]` is set, so every public item needs a doc comment or the build fails.

## Feature flags (important interactions)

- `rand` (default): randomizes hash/reallocation sizing to mitigate DOS collision attacks.
- `deterministic_iteration`: makes iteration order depend only on the insert/delete sequence. **Conflicts with `rand`** — enable it with `--no-default-features`.
- `serde`: non-compact, stable serialization.
- `compactserde`: serializes the in-memory packed form. Format is **not stable** across versions and a corrupt/malicious input can trigger UB. Treat changes here carefully.

When adding logic, be aware code paths often branch on these features and on `#[cfg(target_pointer_width)]` (32- vs 64-bit storage limits differ).

## Architecture

The public types form a layering, from concrete integer storage up to a generic typed wrapper:

- `src/setu64.rs` (`SetU64`) — the core engine and the most complex file. Holds `u64`s. Small sets pack up to seven elements into one word as a smallest-value-plus-deltas encoding (see the long doc comment at the top of the file for the exact bit budget). Large sets use one of three heap formats chosen by value density: `Internal::Dense` (bitmap), `Internal::Heap` (Robin-Hood-style map from high bits to low-bit bitmaps), and a third table format. Iterators live in `src/setu64/iter.rs`.
- `src/setu32.rs` (`SetU32`) — same idea specialized to `u32` for tighter packing. Iterators in `src/setu32/iter.rs`. `src/setu32b.rs` is a separate/experimental `u32` implementation ("Tiny" small-set encoding).
- `src/setusize.rs` (`SetUsize`) — holds `usize`, delegating to `SetU64` or `SetU32` depending on pointer width.
- `src/set64.rs` (`Set64<T>`, `Fits64`) — the main user-facing generic type. `Fits64` is the trait that maps any `Copy` type of ≤64 bits to/from a `u64`; `Set64<T>` wraps `SetU64` using it.

Cross-cutting pieces:

- `src/sets.rs` — the `generic_set!` macro injecting shared methods (e.g. `is_empty`) into every set type. When adding a method that all sets share, add it here.
- `src/copyset.rs` — the `impl_set_methods!` macro providing `PartialEq`/`Eq`/`Debug`/set-algebra operators (`Sub`, etc.) for each concrete type, plus a `#[cfg(test)]` `CopySet` trait used to run one property-test suite generically across all set implementations.
- `src/rand.rs` — the fallback PRNG used when the `rand` feature is off.

### Behavioral invariants to preserve

- These sets iterate over **values**, not references, and `remove`/`contains` on the concrete types take values (not `&value`) — because elements aren't stored in a referenceable form. Keep this contract when extending APIs.
- The in-memory encoding and the heap-format selection are explicitly **not a stable API**; you may change them, but update the doc comments in `setu64.rs`/`setu32.rs` that describe the bit budgets and format-switch thresholds.

## Testing conventions

- Correctness is driven heavily by property testing with `proptest` and `quickcheck`; failing cases are checked in under `proptest-regressions/`. Don't delete regression entries.
- `tests/` holds regression tests named after issues (e.g. `add-with-overflow-issue-16.rs`, `issue-21.rs`). Reproduce a reported bug as a new `tests/issue-N.rs` before fixing.
- `david_allocator/` is a local tracing-allocator crate used to measure real allocation in benches/tests; it is not published.

## Releasing

Bump `version` in Cargo.toml and record the change in `CHANGELOG.md`.
