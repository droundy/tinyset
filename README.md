[![Build Status](https://github.com/droundy/tinysef/actions/workflows/rust.yml/badge.svg)](https://github.com/droundy/tinyset/actions)
[![Build status](https://ci.appveyor.com/api/projects/status/h0rn4amvlwce10pl?svg=true)](https://ci.appveyor.com/project/droundy/tinyset)
[![Crates.io version](https://img.shields.io/crates/v/tinyset.svg)](https://crates.io/crates/tinyset)

[Read the documentation.](https://docs.rs/tinyset)

# tinyset

`tinyset` contains a few collections that are optimized to scale
in size well for small numbers of elements, while still scaling
well in time (and size) for numbers of elements.  We now have
just a few types that you might care for.

1. [`Set64`] is a set for types that are 64 bits in size or less
and are `Copy`, intended for essentially integer types.  This is
our most efficient type, since it can store small sets with just
the size of one pointer, with no heap storage.

2. [`SetU64`] just holds `u64` items, and is the internal storage
of [`Set64`].

3. [`SetU32`] just holds `u32` items, and uses a bit less memory
than [`SetU64`].

4. [`SetUsize`] holds `usize` items, and uses either [SetU64] or
[SetU32] internally.

All of these set types will do no heap allocation for small sets of
small elements.  `TinySet` will store up to 16 bytes of elements
before doing any heap allocation, while `Set` stores sets up to size 8
without allocation.  `Set64` will store up to 22 bytes of elements,
and if all your elements are small (e.g. `0..22 as u64` it will store
them in as few bytes as possible.

These sets all differ from the standard sets in that they iterate
over items rather than references to items, because they do not
store values directly in a way that can be referenced.  All of the
type-specific sets further differ in that `remove` and `contains`
accept values rather than references.
# Benchmarks

To run the benchmark suite, run

    cargo bench

This will give you loads of timings and storage requirements for a
wide variety of set types.
