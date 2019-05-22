[![Build Status](https://travis-ci.org/droundy/tinyset.svg?branch=master)](https://travis-ci.org/droundy/tinyset)
[![Build status](https://ci.appveyor.com/api/projects/status/h0rn4amvlwce10pl?svg=true)](https://ci.appveyor.com/project/droundy/tinyset)
[![Crates.io version](https://img.shields.io/crates/v/tinyset.svg)](https://crates.io/crates/tinyset)

[Read the documentation.](https://docs.rs/tinyset)

# tinyset

`tinyset` contains a few collections that are optimized to scale
in size well for small numbers of elements, while still scaling
well in time (and size) for numbers of elements.  We have three set types:

1. `Set` is basically interchangeable with
   `HashSet`, although it does require that its elements implement
   the `Copy` trait, since otherwise I would have to learn to
   write correct `unsafe` code, which would be scary.  It uses FNV
   hashing when there are large numbers of elements.

2. `TinySet` is places a stronger
    requirement on its elements, which must have trait
    `HasInvalid`.  This is intended for elements that are `Copy`,
    are `Hash`, and have an "invalid" value.  For the unsigned
    integer types, we take their maximum value to mean invalid.
    This constraint allows us to save a bit more space.

3. `Set64` is a set for types that are
   64 bits in size or less and are `Copy`, intended for
   essentially integer types.  This is our most efficient type,
   since it can store values in less space than
   `std::mem::size_of::<T>()`, in the common case that they are
   small numbers.  It is also essentially as fast as any of the
   other set types (faster than many), and can avoid heap
   allocations entirely for small sets.

All of these set types will do no heap allocation for small sets of
small elements.  `TinySet` will store up to 16 bytes of elements
before doing any heap allocation, while `Set` stores sets up to size 8
without allocation.  `Set64` will store up to 22 bytes of elements,
and if all your elements are small (e.g. `0..22 as u64` it will store
them in as few bytes as possible.

All these sets are similar in speed to `fnv::HashSet`.  `Set64` is
usually faster than `fnv::HashSet`, sometimes by as much as a factor
of 2.

# Examples

```
use tinyset::Set;
let mut s: Set<usize> = Set::new();
s.insert(1);
assert!(s.contains(&1));
```

```
use tinyset::TinySet;
let mut s: TinySet<usize> = TinySet::new();
s.insert(1);
assert!(s.contains(&1));
```

```
use tinyset::Set64;
let mut s: Set64<usize> = Set64::new();
s.insert(1);
assert!(s.contains(&1));
```

# Hash maps

In addition to the sets that `tinyset` is named for, we export a
couple of space-efficient hash map implentations, which are
closely related to `Set64` described above.  These are

1. `Map64` is a map from types that are
   64 bits in size or less and are `Copy`, intended for
   essentially integer types.  The value can be of any type, and
   the memory use (especially for small or empty maps) is far
   lower than that of a standard `HashMap`.
1. `Map6464` is a map from types
   that are 64 bits in size or less and are `Copy`, to values that
   are also small and `Copy`.  This is an incredibly
   space-efficient data type with no heap storage when you have
   just a few small keys and values.  On a 64-bit system, the size
   of a `Map6464` is 48 bytes, and if your keys and values both
   fit in 8 bits, you can hold 23 items without using the heap.
   If the keys fit in 16 bits and the values in 8 bits, you can
   hold 15 itmes without resorting to the heap, and so on.  You
   can even hold a whopping 4 64-bit keys with 8-bit values
   without resorting to the heap, making this very efficent.

# Benchmarks

To run the benchmark suite, `cd` into `bench` and then run

    cargo run --bin sets --release

This will give you loads of timings and storage requirements for a
wide variety of set types.

You can alternatively run

    cargo run --bin maps --release

This will give you loads of timings and storage requirements for a
variety of map types.

Unfortunately, I don't know an easy way to check the actual memory use
for a hashmap, so the benchmarks don't check heap usage.  (I used to
do this in a fragile way, but cut it.) If you have any suggestions for
tracking heap use in a nice way, please let me know!
