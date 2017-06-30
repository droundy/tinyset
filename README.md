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

Both of these set types will do no heap allocation for small sets of
small elements.  `TinySet` will store up to 16 bytes of elements
before doing any heap allocation, while `Set` stores sets up to size 8
without allocation.  Both sets are similar in speed to `fnv::HashSet`.

To run the benchmark suite, `cd` into `bench` and then run

    cargo +nightly run --release

This ought to work, provided you've got the nightly rust installed
with rustup.
