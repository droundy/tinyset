// Copyright 2017-2018 David Roundy
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//! `tinyset` contains a few collections that are optimized to scale
//! in size well for small numbers of elements, while still scaling
//! well in time (and size) for numbers of elements.  We have three set types:
//!
//! 1. [`Set`](set/struct.Set.html) is basically interchangeable with
//!    `HashSet`, although it does require that its elements implement
//!    the `Copy` trait, since otherwise I would have to learn to
//!    write correct `unsafe` code, which would be scary.  It uses FNV
//!    hashing when there are large numbers of elements.
//!
//! 2. [`TinySet`](tinyset/struct.TinySet.html) is places a stronger
//!     requirement on its elements, which must have trait
//!     `HasInvalid`.  This is intended for elements that are `Copy`,
//!     are `Hash`, and have an "invalid" value.  For the unsigned
//!     integer types, we take their maximum value to mean invalid.
//!     This constraint allows us to save a bit more space.
//!
//! 3. [`Set64`](u64set/struct.Set64.html) is a set for types that are
//!    64 bits in size or less and are `Copy`, intended for
//!    essentially integer types.  This is our most efficient type,
//!    since it can store values in less space than
//!    `std::mem::size_of::<T>()`, in the common case that they are
//!    small numbers.  It is also essentially as fast as any of the
//!    other set types (faster than many), and can avoid heap
//!    allocations entirely for small sets.
//!
//! All of these set types will do no heap allocation for small sets of
//! small elements.  `TinySet` will store up to 16 bytes of elements
//! before doing any heap allocation, while `Set` stores sets up to size 8
//! without allocation.  `Set64` will store up to 22 bytes of elements,
//! and if all your elements are small (e.g. `0..22 as u64` it will store
//! them in as few bytes as possible.
//!
//! All these sets are similar in speed to `fnv::HashSet`.  `Set64` is
//! usually faster than `fnv::HashSet`, sometimes by as much as a factor
//! of 2.
//!
//! # Examples
//!
//! ```
//! use tinyset::Set;
//! let mut s: Set<usize> = Set::new();
//! s.insert(1);
//! assert!(s.contains(&1));
//! ```
//!
//! ```
//! use tinyset::TinySet;
//! let mut s: TinySet<usize> = TinySet::new();
//! s.insert(1);
//! assert!(s.contains(&1));
//! ```
//!
//! ```
//! use tinyset::Set64;
//! let mut s: Set64<usize> = Set64::new();
//! s.insert(1);
//! assert!(s.contains(&1));
//! ```
//!
//! # Hash maps
//!
//! In addition to the sets that `tinyset` is named for, we export a
//! couple of space-efficient hash map implentations, which are
//! closely related to `Set64` described above.  These are
//!
//! 1. [`Map64`](u64set/struct.Map64.html) is a map from types that are
//!    64 bits in size or less and are `Copy`, intended for
//!    essentially integer types.  The value can be of any type, and
//!    the memory use (especially for small or empty maps) is far
//!    lower than that of a standard `HashMap`.
//! 1. [`Map6464`](u64set/struct.Map6464.html) is a map from types
//!    that are 64 bits in size or less and are `Copy`, to values that
//!    are also small and `Copy`.  This is an incredibly
//!    space-efficient data type with no heap storage when you have
//!    just a few small keys and values.  On a 64-bit system, the size
//!    of a `Map6464` is 48 bytes, and if your keys and values both
//!    fit in 8 bits, you can hold 23 items without using the heap.
//!    If the keys fit in 16 bits and the values in 8 bits, you can
//!    hold 15 itmes without resorting to the heap, and so on.  You
//!    can even hold a whopping 4 64-bit keys with 8-bit values
//!    without resorting to the heap, making this very efficent.

#![deny(missing_docs)]

pub mod vecset;
pub use crate::vecset::VecSet;

pub mod set;
pub use crate::set::Set;

pub mod tinyset;
pub use crate::tinyset::*;

pub mod setu32;
pub use setu32::SetU32;

pub mod setu64;
pub use setu64::SetU64;

pub mod u64set;
pub use crate::u64set::{Set64, Map64, Map6464, Fits64};
