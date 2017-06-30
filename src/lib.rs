//! `tinyset` contains a few collections that are optimized to scale
//! in size well for small numbers of elements, while still scaling
//! well in time (and size) for numbers of elements.  We have two set types:
//!
//! 1. `Set` is basically interchangeable with `HashSet`, although it
//!    does require that its elements implement the `Copy` trait,
//!    since otherwise I would have to learn to write correct `unsafe`
//!    code, which would be scary.  It uses FNV hashing when there are
//!    large numbers of elements.
//!
//! 2. `TinySet` is places a stronger requirement on its elements,
//!     which must have trait `HasInvalid`.  This is intended for
//!     elements that are `Copy`, are `Hash`, and have an "invalid"
//!     value.  For the unsigned integer types, we take their maximum
//!     value to mean invalid.  This constraint allows us to save a
//!     bit more space.
//!
//! Both of these set types will do no heap allocation for small sets
//! of small elements.  `TinySet` will store up to 16 bytes of
//! elements before doing any heap allocation, while `Set` stores sets
//! up to size 8 without allocation.  Both sets are similar in speed
//! to `fnv::HashSet`.
//!
//! # Examples
//!
//! ```
//! use david_set::Set;
//! let mut s: Set<usize> = Set::new();
//! s.insert(1);
//! assert!(s.contains(&1));
//! ```
//!
//! ```
//! use david_set::TinySet;
//! let mut s: TinySet<usize> = TinySet::new();
//! s.insert(1);
//! assert!(s.contains(&1));
//! ```

#![deny(missing_docs)]

extern crate fnv;

mod vecset;
pub use vecset::*;

mod copyset;
pub use copyset::*;

mod castset;
pub use castset::*;

#[cfg(test)]
extern crate rand;

#[cfg(test)]
#[macro_use]
extern crate quickcheck;
