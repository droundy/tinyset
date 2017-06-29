//! david-set contains a few collections that are optimized to scale
//! in size well for small numbers of elements, while still scaling
//! well in time (and size) for numbers of elements.  We have two set types:
//!
//! 1. `Set` is basically interchangeable with `HashSet`, although it
//!    does require that its elements implement the `Copy` trait,
//!    since otherwise I would have to learn to write correct `unsafe`
//!    code, which would be scary.
//!
//! 2. `CastSet` is places a stronger requirement on its elements,
//!     which must have trait `Cast`.  This is intended for elements
//!     that are `Copy`, can be cheaply converted to `usize`, and are
//!     sufficiently evenly distributed that they do not require real
//!     hashing.  Basically, this is suitable if you want to store a
//!     set of indices into an array.  All the basic integer types
//!     should satisfy trait `Cast`.  Oh, and this set also requires
//!     that one value of your type is "invalid".  For the unsigned
//!     integer types, we take their maximum value to mean invalid.
//!     This constraint allows us to save a bit more space.
//!
//! Both of these set types will do no heap allocation for small sets
//! of small elements.  `CastSet` will store up to 16 bytes of
//! elements before doing any heap allocation, while `Set` stores sets
//! up to size 8 without allocation.  Both sets are typically faster
//! than `HashSet` by a factor of around two, although for sets with
//! more than 8 elements `Set` is in fact identical to `HashSet` in
//! performance.
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
//! use david_set::CastSet;
//! let mut s: CastSet<usize> = CastSet::new();
//! s.insert(1);
//! assert!(s.contains(&1));
//! ```

#![deny(missing_docs)]

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
