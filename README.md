[![Build Status](https://travis-ci.org/droundy/tinyset.svg?branch=master)](https://travis-ci.org/droundy/tinyset)
[![Build status](https://ci.appveyor.com/api/projects/status/h0rn4amvlwce10pl?svg=true)](https://ci.appveyor.com/project/droundy/tinyset)

[Read the documentation.](https://docs.rs/tinyset)

# tinyset

`tinyset` contains a few collections that are optimized to scale
in size well for small numbers of elements, while still scaling
well in time (and size) for numbers of elements.  We have two set types:

1. `Set` is basically interchangeable with `HashSet`, although it
   does require that its elements implement the `Copy` trait,
   since otherwise I would have to learn to write correct `unsafe`
   code, which would be scary.  It uses FNV hashing when there are
   large numbers of elements.

2. `TinySet` is places a stronger requirement on its elements,
    which must have trait `HasInvalid`.  This is intended for
    elements that are `Copy`, are `Hash`, and have an "invalid"
    value.  For the unsigned integer types, we take their maximum
    value to mean invalid.  This constraint allows us to save a
    bit more space.

3. `Set64` is a set for types that are 64 bits in size or less and are
   `Copy`, intended for essentially integer types.  This is our most
   efficient type, since it can store values in less space than
   `std::mem::size_of::<T>()`, in the common case that they are small
   numbers.  It is also essentially as fast as any of the other set
   types (faster than many), and can avoid heap allocations
   entirely for small sets.

All of these set types will do no heap allocation for small sets of
small elements.  `TinySet` will store up to 16 bytes of elements
before doing any heap allocation, while `Set` stores sets up to size 8
without allocation.  `Set64` will store up to 22 bytes of elements,
and if all your elements are small (e.g. `0..22 as u64` it will store
them in as few bytes as possible.

All these sets are similar in speed to `fnv::HashSet`.  `Set64` is
usually faster than `fnv::HashSet`, sometimes by as much as a factor
of 2.

To run the benchmark suite, `cd` into `bench` and then run

    cargo +nightly run --release

This will give you loads of timings and storage requirements for a
wide variety of set types.  You will need nightly rust installed with
rustup.
